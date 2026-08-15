// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Xunit;

namespace Pamba.WinUI.Tests;

public sealed class WinUIRuntimeTests : IClassFixture<DispatcherQueueFixture>
{
  private readonly DispatcherQueueFixture _fixture;

  public WinUIRuntimeTests(DispatcherQueueFixture fixture) => _fixture = fixture;

  private sealed record State(int Count);

  private abstract record Msg
  {
    internal sealed record Increment : Msg;

    internal sealed record RuntimeErrored(string Detail) : Msg;
  }

  private abstract record Cmd;

  private sealed record Sub(SubscriptionKey Key) : ISubscription<Msg>;

  private sealed class CountProjection : Projection<State>
  {
    public List<int> Initial { get; } = [];

    public List<int> Changes { get; } = [];

    public CountProjection() =>
        Segment(s => s.Count, Initial.Add, (_, @new) => Changes.Add(@new));
  }

  private static MvuProgram<State, Msg, Cmd, Sub> CreateProgram(List<PambaError>? errors = null) => new()
  {
    Init = () => (new State(0), []),
    Update = (msg, state) => msg switch
    {
      Msg.Increment => (state with { Count = state.Count + 1 }, []),
      _ => (state, [])
    },
    Subscriptions = _ => [],
    OnRuntimeError = err =>
    {
      errors?.Add(err);
      return new Msg.RuntimeErrored(err.ToString());
    },
    Validate = ValidationResult<State, Msg>.AlwaysValid
  };

  private static ValueTask<CommandResult<Msg>> NoOpExecutor(Cmd cmd, Dispatch<Msg> dispatch, CancellationToken ct) =>
      ValueTask.FromResult(CommandResult<Msg>.Ok);

  private static IAsyncDisposable NoOpStarter(Sub sub, Dispatch<Msg> dispatch, Action<Exception> onError) =>
      new NoOpHandle();

  private sealed class NoOpHandle : IAsyncDisposable
  {
    public ValueTask DisposeAsync() => ValueTask.CompletedTask;
  }

  [Fact]
  public void Messages_are_processed_on_the_dispatcher_thread()
  {
    List<bool> onDispatcherThread = [];

    using MvuRuntime<State, Msg, Cmd, Sub> runtime = WinUIRuntime
        .Create(CreateProgram(), _fixture.Queue)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithProjection((_, _) => onDispatcherThread.Add(_fixture.Queue.HasThreadAccess))
        .Start();

    runtime.Dispatch(new Msg.Increment());
    _fixture.Drain();

    Assert.Equal(1, runtime.State.Count);
    Assert.True(Assert.Single(onDispatcherThread));
  }

  [Fact]
  public void Start_without_projection_still_runs_the_loop()
  {
    using MvuRuntime<State, Msg, Cmd, Sub> runtime = WinUIRuntime
        .Create(CreateProgram(), _fixture.Queue)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .Start();

    runtime.Dispatch(new Msg.Increment());
    _fixture.Drain();

    Assert.Equal(1, runtime.State.Count);
  }

  [Fact]
  public void The_state_change_only_overload_does_not_project_the_initial_state()
  {
    List<(int Old, int New)> changes = [];

    using MvuRuntime<State, Msg, Cmd, Sub> runtime = WinUIRuntime
        .Create(CreateProgram(), _fixture.Queue)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithProjection((old, @new) => changes.Add((old.Count, @new.Count)))
        .Start();

    _fixture.Drain();
    Assert.Empty(changes);

    runtime.Dispatch(new Msg.Increment());
    _fixture.Drain();

    Assert.Equal((0, 1), Assert.Single(changes));
  }

  [Fact]
  public void The_init_overload_projects_the_initial_state_once_before_any_message()
  {
    List<int> initial = [];
    List<int> changes = [];

    using MvuRuntime<State, Msg, Cmd, Sub> runtime = WinUIRuntime
        .Create(CreateProgram(), _fixture.Queue)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithProjection(s => initial.Add(s.Count), (_, @new) => changes.Add(@new.Count))
        .Start();

    Assert.Equal(0, Assert.Single(initial));
    Assert.Empty(changes);

    runtime.Dispatch(new Msg.Increment());
    _fixture.Drain();

    Assert.Equal(1, Assert.Single(changes));
  }

  [Fact]
  public void A_Projection_wires_both_its_initial_and_its_transition_segments()
  {
    CountProjection projection = new();

    using MvuRuntime<State, Msg, Cmd, Sub> runtime = WinUIRuntime
        .Create(CreateProgram(), _fixture.Queue)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithProjection(projection)
        .Start();

    Assert.Equal(0, Assert.Single(projection.Initial));

    runtime.Dispatch(new Msg.Increment());
    _fixture.Drain();

    Assert.Equal(1, Assert.Single(projection.Changes));
  }

  [Fact]
  public void History_is_recorded_when_the_builder_enables_it()
  {
    using MvuRuntime<State, Msg, Cmd, Sub> runtime = WinUIRuntime
        .Create(CreateProgram(), _fixture.Queue)
        .WithCommandExecutor(NoOpExecutor)
        .WithSubscriptionStarter(NoOpStarter)
        .WithMaxHistorySize(4)
        .Start();

    runtime.Dispatch(new Msg.Increment());
    _fixture.Drain();

    TransitionSnapshot<State, Msg, Cmd, Sub> snapshot = Assert.Single(runtime.MessageHistory);
    Assert.Equal(0, snapshot.StateBefore.Count);
    Assert.Equal(1, snapshot.StateAfter.Count);
  }

  [Fact]
  public void Create_rejects_a_missing_program_or_queue()
  {
    Assert.Throws<ArgumentNullException>(() =>
        WinUIRuntime.Create<State, Msg, Cmd, Sub>(null!, _fixture.Queue));

    Assert.Throws<ArgumentNullException>(() =>
        WinUIRuntime.Create(CreateProgram(), null!));
  }
}
