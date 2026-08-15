// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Xunit;

namespace Pamba.Tests;

/// <summary>
/// Integration tests for <see cref="MvuRuntime{TState, TMsg, TCmd, TSub}"/>
/// exercising the full dispatch loop via the <see cref="MvuRuntimeBuilder"/>.
/// Uses a synchronous dispatcher so messages process immediately.
/// </summary>
public sealed class MvuRuntimeTests
{
  private sealed record TestState(int Count, bool SubActive);

  private abstract record TestMsg
  {
    internal sealed record Increment : TestMsg;
    internal sealed record ActivateSub : TestMsg;
    internal sealed record DeactivateSub : TestMsg;
    internal sealed record SetValue(int Value) : TestMsg;
    internal sealed record CommandErrored(string Detail) : TestMsg;
    internal sealed record RuntimeErrored(string Detail) : TestMsg;
    internal sealed record ValidationRejected : TestMsg;
  }

  private abstract record TestCmd
  {
    internal sealed record Save(int Value) : TestCmd;
  }

  private sealed record TestSub(SubscriptionKey Key) : ISubscription<TestMsg>;

  private sealed class TrackingAsyncDisposable : IAsyncDisposable
  {
    public bool IsDisposed { get; private set; }
    public ValueTask DisposeAsync() { IsDisposed = true; return ValueTask.CompletedTask; }
  }

  private static MvuProgram<TestState, TestMsg, TestCmd, TestSub> CreateProgram() =>
      CreateProgram(state => new ValidationResult<TestState, TestMsg>.Valid(state));

  private static MvuProgram<TestState, TestMsg, TestCmd, TestSub> CreateProgram(
      Func<TestState, ValidationResult<TestState, TestMsg>> validate)
  {
    return new MvuProgram<TestState, TestMsg, TestCmd, TestSub>
    {
      Init = () => (new TestState(0, false), []),
      Update = (msg, state) => msg switch
      {
        TestMsg.Increment => (new TestState(state.Count + 1, state.SubActive), [new TestCmd.Save(state.Count + 1)]),
        TestMsg.ActivateSub => (state with { SubActive = true }, []),
        TestMsg.DeactivateSub => (state with { SubActive = false }, []),
        TestMsg.SetValue v => (new TestState(v.Value, state.SubActive), []),
        TestMsg.CommandErrored => (state, []),
        TestMsg.RuntimeErrored => (state, []),
        TestMsg.ValidationRejected => (state, []),
        _ => (state, [])
      },
      Subscriptions = state => state.SubActive
          ? [new TestSub(SubscriptionKey.From("ticker"))]
          : [],
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = validate
    };
  }

  private static MvuRuntime<TestState, TestMsg, TestCmd, TestSub> StartRuntime(
      MvuProgram<TestState, TestMsg, TestCmd, TestSub> program,
      CommandExecutor<TestCmd, TestMsg> executor,
      SubscriptionStarter<TestSub, TestMsg> starter,
      Action<TestState>? onInit = null,
      Action<TestState, TestState>? onStateChanged = null)
  {
    Func<Action, bool> dispatcher = action => { action(); return true; };

    IRuntimeNeedsDispatcher<TestState, TestMsg, TestCmd, TestSub> withSubs = MvuRuntimeBuilder
        .Create(program)
        .WithCommandExecutor(executor)
        .WithSubscriptionStarter(starter);

    if (onInit is not null && onStateChanged is not null)
    {
      return withSubs.WithDispatcher(dispatcher, onInit, onStateChanged).Start();
    }

    if (onStateChanged is not null)
    {
      return withSubs.WithDispatcher(dispatcher, onStateChanged).Start();
    }

    return withSubs.WithDispatcher(dispatcher).Start();
  }

  private static ValueTask<CommandResult<TestMsg>> NoOpExecutor(TestCmd cmd, Dispatch<TestMsg> dispatch, CancellationToken ct) =>
      ValueTask.FromResult(CommandResult<TestMsg>.Ok);

  private static IAsyncDisposable NoOpStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError) =>
      new TrackingAsyncDisposable();

  [Fact]
  public void Init_sets_state_and_executes_startup_commands()
  {
    List<TestCmd> executedCmds = [];
    ValueTask<CommandResult<TestMsg>> TrackingExecutor(TestCmd cmd, Dispatch<TestMsg> dispatch, CancellationToken ct)
    {
      executedCmds.Add(cmd);
      return ValueTask.FromResult(CommandResult<TestMsg>.Ok);
    }

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(42, false), [new TestCmd.Save(42)]),
      Update = (_, state) => (state, []),
      Subscriptions = _ => [],
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(program, TrackingExecutor, NoOpStarter);

    Assert.Equal(42, runtime.State.Count);
    Assert.Single(executedCmds);
    Assert.Equal(new TestCmd.Save(42), executedCmds[0]);
  }

  [Fact]
  public void Dispatch_processes_message_and_updates_state()
  {
    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), NoOpExecutor, NoOpStarter);

    runtime.Dispatch(new TestMsg.Increment());

    Assert.Equal(1, runtime.State.Count);
  }

  [Fact]
  public void Dispatch_executes_returned_commands()
  {
    List<TestCmd> executedCmds = [];
    ValueTask<CommandResult<TestMsg>> TrackingExecutor(TestCmd cmd, Dispatch<TestMsg> dispatch, CancellationToken ct)
    {
      executedCmds.Add(cmd);
      return ValueTask.FromResult(CommandResult<TestMsg>.Ok);
    }

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), TrackingExecutor, NoOpStarter);

    runtime.Dispatch(new TestMsg.Increment());

    Assert.Single(executedCmds);
    Assert.Equal(new TestCmd.Save(1), executedCmds[0]);
  }

  [Fact]
  public void Dispatch_invokes_state_change_callback_with_old_and_new_state()
  {
    List<(TestState Old, TestState New)> callbacks = [];

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime = StartRuntime(
        CreateProgram(),
        NoOpExecutor,
        NoOpStarter,
        onStateChanged: (oldState, newState) => callbacks.Add((oldState, newState)));

    runtime.Dispatch(new TestMsg.Increment());

    Assert.Single(callbacks);
    Assert.Equal(0, callbacks[0].Old.Count);
    Assert.Equal(1, callbacks[0].New.Count);
  }

  [Fact]
  public void Validate_is_called_on_init_and_every_transition()
  {
    int validateCallCount = 0;
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = CreateProgram(state =>
    {
      validateCallCount++;
      return new ValidationResult<TestState, TestMsg>.Valid(state);
    });

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(program, NoOpExecutor, NoOpStarter);

    Assert.Equal(1, validateCallCount);

    runtime.Dispatch(new TestMsg.Increment());

    Assert.Equal(2, validateCallCount);
  }

  [Fact]
  public void Validate_returns_Invalid_reverts_state_and_dispatches_corrective_message()
  {
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = CreateProgram(
        state => state.Count < 0
            ? new ValidationResult<TestState, TestMsg>.Invalid(new TestMsg.ValidationRejected())
            : new ValidationResult<TestState, TestMsg>.Valid(state));

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(program, NoOpExecutor, NoOpStarter);

    // SetValue(-1) triggers validation rejection; state should revert to 0
    runtime.Dispatch(new TestMsg.SetValue(-1));

    Assert.Equal(0, runtime.State.Count);
  }

  [Fact]
  public void Dispatch_starts_subscriptions_when_state_activates_them()
  {
    List<string> startedKeys = [];
    IAsyncDisposable TrackingStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError)
    {
      startedKeys.Add(sub.Key.Value);
      return new TrackingAsyncDisposable();
    }

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), NoOpExecutor, TrackingStarter);

    Assert.Empty(startedKeys);

    runtime.Dispatch(new TestMsg.ActivateSub());

    Assert.Single(startedKeys);
    Assert.Equal("ticker", startedKeys[0]);
  }

  [Fact]
  public void Dispatch_cancels_subscriptions_when_state_deactivates_them()
  {
    Dictionary<string, TrackingAsyncDisposable> handles = [];
    IAsyncDisposable TrackingStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError)
    {
      TrackingAsyncDisposable handle = new();
      handles[sub.Key.Value] = handle;
      return handle;
    }

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), NoOpExecutor, TrackingStarter);

    runtime.Dispatch(new TestMsg.ActivateSub());
    Assert.False(handles["ticker"].IsDisposed);

    runtime.Dispatch(new TestMsg.DeactivateSub());
    Assert.True(handles["ticker"].IsDisposed);
  }

  [Fact]
  public void Command_executor_can_dispatch_messages_back_into_loop()
  {
    ValueTask<CommandResult<TestMsg>> DispatchingExecutor(TestCmd cmd, Dispatch<TestMsg> dispatch, CancellationToken ct)
    {
      if (cmd is TestCmd.Save s)
      {
        dispatch(new TestMsg.SetValue(s.Value * 10));
      }

      return ValueTask.FromResult(CommandResult<TestMsg>.Ok);
    }

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), DispatchingExecutor, NoOpStarter);

    runtime.Dispatch(new TestMsg.Increment());

    // Increment: Count=0 -> Count=1, produces Save(1)
    // Save(1) dispatches SetValue(10)
    // SetValue(10): Count -> 10
    Assert.Equal(10, runtime.State.Count);
  }

  [Fact]
  public void Dispatch_after_dispose_is_noop()
  {
    MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), NoOpExecutor, NoOpStarter);

    runtime.Dispatch(new TestMsg.Increment());
    Assert.Equal(1, runtime.State.Count);

    runtime.Dispose();

    runtime.Dispatch(new TestMsg.Increment());
    Assert.Equal(1, runtime.State.Count);
  }

  [Fact]
  public void Dispose_cancels_all_active_subscriptions()
  {
    Dictionary<string, TrackingAsyncDisposable> handles = [];
    IAsyncDisposable TrackingStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError)
    {
      TrackingAsyncDisposable handle = new();
      handles[sub.Key.Value] = handle;
      return handle;
    }

    MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), NoOpExecutor, TrackingStarter);

    runtime.Dispatch(new TestMsg.ActivateSub());
    Assert.False(handles["ticker"].IsDisposed);

    runtime.Dispose();
    Assert.True(handles["ticker"].IsDisposed);
  }

  [Fact]
  public void Command_error_result_dispatches_error_message()
  {
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, false), [new TestCmd.Save(0)]),
      Update = (msg, state) => msg switch
      {
        TestMsg.CommandErrored => (new TestState(-1, state.SubActive), []),
        _ => (state, [])
      },
      Subscriptions = _ => [],
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime = StartRuntime(
        program,
        (_, _, _) => ValueTask.FromResult(CommandResult<TestMsg>.Error(new TestMsg.CommandErrored("DB failed"))),
        NoOpStarter);

    Assert.Equal(-1, runtime.State.Count);
  }

  [Fact]
  public void SubscriptionStarter_exception_routes_runtime_error_via_OnRuntimeError()
  {
    List<string> runtimeErrors = [];

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, true), []),
      Update = (msg, state) => msg switch
      {
        TestMsg.RuntimeErrored e => (state with { SubActive = false }, []),
        _ => (state, [])
      },
      Subscriptions = state => state.SubActive
          ? [new TestSub(SubscriptionKey.From("ticker"))]
          : [],
      OnRuntimeError = err => { runtimeErrors.Add(err.ToString()); return new TestMsg.RuntimeErrored(err.ToString()); },
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    IAsyncDisposable ThrowingStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError) =>
        throw new InvalidOperationException("Timer init failed");

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(program, NoOpExecutor, ThrowingStarter);

    Assert.Single(runtimeErrors);
    Assert.False(runtime.State.SubActive);
  }

  [Fact]
  public void Dispatch_skips_subscriptions_and_projection_when_state_unchanged()
  {
    List<(TestState Old, TestState New)> callbacks = [];
    int subscriptionsCalled = 0;

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, false), []),
      Update = (_, state) => (state, []),
      Subscriptions = state =>
      {
        subscriptionsCalled++;
        return [];
      },
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime = StartRuntime(
        program,
        NoOpExecutor,
        NoOpStarter,
        onStateChanged: (old, @new) => callbacks.Add((old, @new)));

    int subscriptionsAfterInit = subscriptionsCalled;

    runtime.Dispatch(new TestMsg.Increment());

    Assert.Empty(callbacks);
    Assert.Equal(subscriptionsAfterInit, subscriptionsCalled);
  }

  [Fact]
  public void OnInit_callback_fires_with_initial_state()
  {
    TestState? capturedInit = null;

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime = StartRuntime(
        CreateProgram(),
        NoOpExecutor,
        NoOpStarter,
        onInit: state => capturedInit = state,
        onStateChanged: (_, _) => { });

    Assert.NotNull(capturedInit);
    Assert.Equal(0, capturedInit.Count);
    Assert.False(capturedInit.SubActive);
  }

  [Fact]
  public void Validate_rejection_corrective_message_effects_are_preserved()
  {
    // C1/C2 regression: corrective message that changes state must not be overwritten
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, false), []),
      Update = (msg, state) => msg switch
      {
        TestMsg.SetValue v => (new TestState(v.Value, state.SubActive), []),
        TestMsg.ValidationRejected => (new TestState(99, state.SubActive), []),
        _ => (state, [])
      },
      Subscriptions = _ => [],
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = state => state.Count < 0
          ? new ValidationResult<TestState, TestMsg>.Invalid(new TestMsg.ValidationRejected())
          : new ValidationResult<TestState, TestMsg>.Valid(state)
    };

    using var runtime = StartRuntime(program, NoOpExecutor, NoOpStarter);

    // SetValue(-1) triggers validation rejection; corrective handler sets Count=99
    runtime.Dispatch(new TestMsg.SetValue(-1));

    Assert.Equal(99, runtime.State.Count);
  }

  [Fact]
  public void Dispatch_rejected_does_not_corrupt_state()
  {
    // C3 regression: when enqueue returns false, state must not be mutated
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = CreateProgram();
    List<string> runtimeErrors = [];

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> programWithTracking = new()
    {
      Init = program.Init,
      Update = program.Update,
      Subscriptions = program.Subscriptions,
      OnRuntimeError = err =>
      {
        runtimeErrors.Add(err.ToString());
        return new TestMsg.RuntimeErrored(err.ToString());
      },
      Validate = program.Validate
    };

    // Use a dispatcher that can be switched to rejecting mode
    bool rejectAll = false;
    Func<Action, bool> rejectingDispatcher = action =>
    {
      if (rejectAll)
      {
        return false;
      }

      action();
      return true;
    };

    using var runtime = MvuRuntimeBuilder
        .Create(programWithTracking)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithDispatcher(rejectingDispatcher)
        .Start();

    // Switch to rejecting mode after startup
    rejectAll = true;
    runtime.Dispatch(new TestMsg.Increment());

    // OnRuntimeError should have been called with DispatchRejected
    Assert.Single(runtimeErrors);
  }

  [Fact]
  public void Subscription_parameter_change_restarts_subscription()
  {
    // C4 regression: same key, different data must restart subscription
    Dictionary<string, List<int>> startedValues = [];

    // Use a sub type that carries data beyond the key
    MvuProgram<TestState, TestMsg, TestCmd, TestSubWithData> programWithData = new()
    {
      Init = () => (new TestState(0, false), []),
      Update = (msg, state) => msg switch
      {
        TestMsg.Increment => (new TestState(state.Count + 1, true), []),
        _ => (state, [])
      },
      Subscriptions = state => state.SubActive
          ? [new TestSubWithData(SubscriptionKey.From("ticker"), state.Count)]
          : [],
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    Func<Action, bool> dispatcher = action => { action(); return true; };

    using var runtime = MvuRuntimeBuilder
        .Create(programWithData)
        .WithCommandExecutor(
            (TestCmd cmd, Dispatch<TestMsg> dispatch, CancellationToken ct) => ValueTask.FromResult(CommandResult<TestMsg>.Ok))
        .WithSubscriptionStarter(
            (TestSubWithData sub, Dispatch<TestMsg> dispatch, Action<Exception> onError) =>
            {
              if (!startedValues.TryGetValue(sub.Key.Value, out var list))
              {
                list = [];
                startedValues[sub.Key.Value] = list;
              }

              list.Add(sub.Interval);
              return new TrackingAsyncDisposable();
            })
        .WithDispatcher(dispatcher)
        .Start();

    // First Increment: Count=1, starts subscription with Interval=1
    runtime.Dispatch(new TestMsg.Increment());
    Assert.Single(startedValues["ticker"]);
    Assert.Equal(1, startedValues["ticker"][0]);

    // Second Increment: Count=2, same key but different Interval=2 -> restart
    runtime.Dispatch(new TestMsg.Increment());
    Assert.Equal(2, startedValues["ticker"].Count);
    Assert.Equal(2, startedValues["ticker"][1]);
  }

  [Fact]
  public void CommandExecutor_unexpected_throw_routes_via_OnRuntimeError_as_CommandExecutorFailed()
  {
    List<PambaError> runtimeErrors = [];

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, false), [new TestCmd.Save(0)]),
      Update = (msg, state) => msg switch
      {
        TestMsg.RuntimeErrored => (new TestState(-1, state.SubActive), []),
        _ => (state, [])
      },
      Subscriptions = _ => [],
      OnRuntimeError = err =>
      {
        runtimeErrors.Add(err);
        return new TestMsg.RuntimeErrored(err.ToString());
      },
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime = StartRuntime(
        program,
        (_, _, _) => ValueTask.FromException<CommandResult<TestMsg>>(new InvalidOperationException("Executor bug")),
        NoOpStarter);

    // Unexpected throw from executor routed via OnRuntimeError as CommandExecutorFailed
    Assert.Single(runtimeErrors);
    Assert.IsType<PambaError.CommandExecutorFailed>(runtimeErrors[0]);
    Assert.Equal(-1, runtime.State.Count);
  }

  [Fact]
  public void OnRuntimeError_throwing_traces_error_but_runtime_survives()
  {
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, true), []),
      Update = (msg, state) => msg switch
      {
        _ => (state with { SubActive = false }, [])
      },
      Subscriptions = state => state.SubActive
          ? [new TestSub(SubscriptionKey.From("ticker"))]
          : [],
      OnRuntimeError = _ => throw new InvalidOperationException("Runtime handler bug"),
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    IAsyncDisposable ThrowingStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError) =>
        throw new InvalidOperationException("Starter failed");

    // Suppress trace listeners so Trace.TraceError does not surface in the test host
    using var _ = new SuppressTraceOutput();

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(program, NoOpExecutor, ThrowingStarter);

    Assert.Equal(0, runtime.State.Count);
  }

  /// <summary>
  /// Temporarily removes trace listeners to suppress Trace.TraceError
  /// output during a test, restoring the original listeners on dispose.
  /// </summary>
  private sealed class SuppressTraceOutput : IDisposable
  {
    private readonly System.Diagnostics.TraceListener[] _original;

    public SuppressTraceOutput()
    {
      _original = new System.Diagnostics.TraceListener[System.Diagnostics.Trace.Listeners.Count];
      System.Diagnostics.Trace.Listeners.CopyTo(_original, 0);
      System.Diagnostics.Trace.Listeners.Clear();
    }

    public void Dispose()
    {
      System.Diagnostics.Trace.Listeners.Clear();
      System.Diagnostics.Trace.Listeners.AddRange(_original);
    }
  }

  [Fact]
  public void Projection_exception_routes_ProjectionFailed_via_OnRuntimeError()
  {
    List<PambaError> runtimeErrors = [];

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, false), []),
      Update = (msg, state) => msg switch
      {
        TestMsg.Increment => (new TestState(state.Count + 1, state.SubActive), []),
        TestMsg.RuntimeErrored => (state, []),
        _ => (state, [])
      },
      Subscriptions = _ => [],
      OnRuntimeError = err =>
      {
        runtimeErrors.Add(err);
        return new TestMsg.RuntimeErrored(err.ToString());
      },
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime = StartRuntime(
        program,
        NoOpExecutor,
        NoOpStarter,
        onStateChanged: (_, _) => throw new InvalidOperationException("Projection bug"));

    runtime.Dispatch(new TestMsg.Increment());

    // State transition should have completed despite projection failure
    Assert.Equal(1, runtime.State.Count);
    Assert.Single(runtimeErrors);
    Assert.IsType<PambaError.ProjectionFailed>(runtimeErrors[0]);
  }

  [Fact]
  public void DispatchAll_processes_all_messages_and_produces_single_projection()
  {
    List<(TestState Old, TestState New)> projections = [];

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime = StartRuntime(
        CreateProgram(),
        NoOpExecutor,
        NoOpStarter,
        onStateChanged: (old, @new) => projections.Add((old, @new)));

    runtime.DispatchAll(new TestMsg.Increment(), new TestMsg.Increment(), new TestMsg.Increment());

    // State flows through all three messages
    Assert.Equal(3, runtime.State.Count);
    // Projection fires exactly once for the whole batch, not once per message
    Assert.Single(projections);
    Assert.Equal(0, projections[0].Old.Count);
    Assert.Equal(3, projections[0].New.Count);
  }

  [Fact]
  public void DispatchAll_with_validation_rejection_dispatches_corrective_messages()
  {
    // If a message in the batch produces an invalid state, a corrective message is
    // dispatched after the batch completes (not inside the batch loop).
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, false), []),
      Update = (msg, state) => msg switch
      {
        TestMsg.SetValue v => (new TestState(v.Value, state.SubActive), []),
        TestMsg.ValidationRejected => (new TestState(99, state.SubActive), []),
        _ => (state, [])
      },
      Subscriptions = _ => [],
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = state => state.Count < 0
          ? new ValidationResult<TestState, TestMsg>.Invalid(new TestMsg.ValidationRejected())
          : new ValidationResult<TestState, TestMsg>.Valid(state)
    };

    using var runtime = StartRuntime(program, NoOpExecutor, NoOpStarter);

    // Batch: SetValue(1), SetValue(-1) [rejected], SetValue(2)
    // After batch: corrective ValidationRejected dispatched → Count=99
    runtime.DispatchAll(new TestMsg.SetValue(1), new TestMsg.SetValue(-1), new TestMsg.SetValue(2));

    // SetValue(2) is the last accepted state from the batch; then corrective sets Count=99
    Assert.Equal(99, runtime.State.Count);
  }

  [Fact]
  public void DispatchAll_empty_span_is_noop()
  {
    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), NoOpExecutor, NoOpStarter);

    runtime.DispatchAll(); // empty params is empty span

    Assert.Equal(0, runtime.State.Count);
  }

  [Fact]
  public void Subscription_onError_routes_SubscriptionFaulted_carrying_key_and_cause()
  {
    List<PambaError> runtimeErrors = [];
    Action<Exception>? capturedOnError = null;

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, true), []),
      Update = (_, state) => (state, []),
      Subscriptions = state => state.SubActive
          ? [new TestSub(SubscriptionKey.From("ticker"))]
          : [],
      OnRuntimeError = err =>
      {
        runtimeErrors.Add(err);
        return new TestMsg.RuntimeErrored(err.ToString());
      },
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    IAsyncDisposable CapturingStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError)
    {
      capturedOnError = onError;
      return new TrackingAsyncDisposable();
    }

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(program, NoOpExecutor, CapturingStarter);

    Assert.NotNull(capturedOnError);
    InvalidOperationException tickFailure = new("tick handler threw");
    capturedOnError(tickFailure);

    PambaError.SubscriptionFaulted faulted =
        Assert.IsType<PambaError.SubscriptionFaulted>(Assert.Single(runtimeErrors));
    Assert.Equal("ticker", faulted.Key.Value);
    Assert.Same(tickFailure, faulted.Cause);
  }

  [Fact]
  public void Duplicate_subscription_keys_route_DuplicateSubscriptionKey_and_start_only_the_first()
  {
    List<PambaError> runtimeErrors = [];
    List<int> startedIntervals = [];

    MvuProgram<TestState, TestMsg, TestCmd, TestSubWithData> program = new()
    {
      Init = () => (new TestState(0, true), []),
      Update = (_, state) => (state, []),
      Subscriptions = _ =>
      [
        new TestSubWithData(SubscriptionKey.From("ticker"), 1),
        new TestSubWithData(SubscriptionKey.From("ticker"), 2)
      ],
      OnRuntimeError = err =>
      {
        runtimeErrors.Add(err);
        return new TestMsg.RuntimeErrored(err.ToString());
      },
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    using var runtime = MvuRuntimeBuilder
        .Create(program)
        .WithCommandExecutor(
            (TestCmd cmd, Dispatch<TestMsg> dispatch, CancellationToken ct) =>
                ValueTask.FromResult(CommandResult<TestMsg>.Ok))
        .WithSubscriptionStarter(
            (TestSubWithData sub, Dispatch<TestMsg> dispatch, Action<Exception> onError) =>
            {
              startedIntervals.Add(sub.Interval);
              return new TrackingAsyncDisposable();
            })
        .WithDispatcher(action => { action(); return true; })
        .Start();

    PambaError.DuplicateSubscriptionKey duplicate =
        Assert.IsType<PambaError.DuplicateSubscriptionKey>(Assert.Single(runtimeErrors));
    Assert.Equal("ticker", duplicate.Key.Value);

    Assert.Equal([1], startedIntervals);
  }

  [Fact]
  public void Starter_exception_routes_SubscriptionStartFailed_carrying_key_and_cause()
  {
    List<PambaError> runtimeErrors = [];

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, true), []),
      Update = (_, state) => (state with { SubActive = false }, []),
      Subscriptions = state => state.SubActive
          ? [new TestSub(SubscriptionKey.From("ticker"))]
          : [],
      OnRuntimeError = err =>
      {
        runtimeErrors.Add(err);
        return new TestMsg.RuntimeErrored(err.ToString());
      },
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    InvalidOperationException startFailure = new("Timer init failed");
    IAsyncDisposable ThrowingStarter(TestSub sub, Dispatch<TestMsg> dispatch, Action<Exception> onError) =>
        throw startFailure;

    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(program, NoOpExecutor, ThrowingStarter);

    PambaError.SubscriptionStartFailed failed =
        Assert.IsType<PambaError.SubscriptionStartFailed>(Assert.Single(runtimeErrors));
    Assert.Equal("ticker", failed.Key.Value);
    Assert.Same(startFailure, failed.Cause);
  }

  [Fact]
  public void Dispatch_after_queue_shutdown_routes_DispatchRejected()
  {
    List<PambaError> runtimeErrors = [];

    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, false), []),
      Update = (_, state) => (state, []),
      Subscriptions = _ => [],
      OnRuntimeError = err =>
      {
        runtimeErrors.Add(err);
        return new TestMsg.RuntimeErrored(err.ToString());
      },
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    bool queueOpen = true;
    Func<Action, bool> shuttingDownDispatcher = action =>
    {
      if (!queueOpen)
      {
        return false;
      }

      action();
      return true;
    };

    using var runtime = MvuRuntimeBuilder
        .Create(program)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithDispatcher(shuttingDownDispatcher)
        .Start();

    queueOpen = false;
    runtime.Dispatch(new TestMsg.Increment());

    Assert.IsType<PambaError.DispatchRejected>(Assert.Single(runtimeErrors));
  }

  [Fact]
  public void MessageHistory_is_empty_when_history_is_not_enabled()
  {
    using MvuRuntime<TestState, TestMsg, TestCmd, TestSub> runtime =
        StartRuntime(CreateProgram(), NoOpExecutor, NoOpStarter);

    runtime.Dispatch(new TestMsg.Increment());

    Assert.Equal(1, runtime.State.Count);
    Assert.Empty(runtime.MessageHistory);
  }

  [Fact]
  public void MessageHistory_records_the_states_message_and_commands_of_each_transition()
  {
    using var runtime = MvuRuntimeBuilder
        .Create(CreateProgram())
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithDispatcher(action => { action(); return true; })
        .WithMaxHistorySize(8)
        .Start();

    runtime.Dispatch(new TestMsg.Increment());

    TransitionSnapshot<TestState, TestMsg, TestCmd, TestSub> snapshot =
        Assert.Single(runtime.MessageHistory);
    Assert.IsType<TestMsg.Increment>(snapshot.Message);
    Assert.Equal(0, snapshot.StateBefore.Count);
    Assert.Equal(1, snapshot.StateAfter.Count);
    Assert.Equal(new TestCmd.Save(1), Assert.Single(snapshot.Commands));
  }

  [Fact]
  public void MessageHistory_evicts_the_oldest_transition_beyond_its_configured_size()
  {
    using var runtime = MvuRuntimeBuilder
        .Create(CreateProgram())
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithDispatcher(action => { action(); return true; })
        .WithMaxHistorySize(2)
        .Start();

    runtime.Dispatch(new TestMsg.SetValue(1));
    runtime.Dispatch(new TestMsg.SetValue(2));
    runtime.Dispatch(new TestMsg.SetValue(3));

    List<TransitionSnapshot<TestState, TestMsg, TestCmd, TestSub>> history = [.. runtime.MessageHistory];
    Assert.Equal(2, history.Count);
    Assert.Equal([2, 3], history.ConvertAll(h => h.StateAfter.Count));
  }

  [Fact]
  public void MessageHistory_records_active_subscriptions_when_the_state_did_not_change()
  {
    MvuProgram<TestState, TestMsg, TestCmd, TestSub> program = new()
    {
      Init = () => (new TestState(0, true), []),
      Update = (_, state) => (state, []),
      Subscriptions = state => state.SubActive
          ? [new TestSub(SubscriptionKey.From("ticker"))]
          : [],
      OnRuntimeError = err => new TestMsg.RuntimeErrored(err.ToString()),
      Validate = ValidationResult<TestState, TestMsg>.AlwaysValid
    };

    using var runtime = MvuRuntimeBuilder
        .Create(program)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithDispatcher(action => { action(); return true; })
        .WithMaxHistorySize(4)
        .Start();

    runtime.Dispatch(new TestMsg.Increment());

    TransitionSnapshot<TestState, TestMsg, TestCmd, TestSub> snapshot =
        Assert.Single(runtime.MessageHistory);
    Assert.Equal(snapshot.StateBefore, snapshot.StateAfter);
    Assert.Equal("ticker", Assert.Single(snapshot.Subscriptions).Key.Value);
  }

  private sealed record TestSubWithData(SubscriptionKey Key, int Interval) : ISubscription<TestMsg>;
}
