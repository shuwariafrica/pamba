// Copyright (c) 2026 Ali Rashid. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;

namespace Pamba;

/// <summary>
/// The MVU dispatch loop. Framework-agnostic.
/// Requires a thread dispatcher for FIFO message ordering.
/// See <see cref="MvuRuntimeBuilder"/> for construction.
/// </summary>
/// <typeparam name="TState">Immutable application state.</typeparam>
/// <typeparam name="TMsg">Message type.</typeparam>
/// <typeparam name="TCmd">Command type.</typeparam>
/// <typeparam name="TSub">Subscription type.</typeparam>
public sealed class MvuRuntime<TState, TMsg, TCmd, TSub> : IDisposable, IAsyncDisposable
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  private readonly MvuProgram<TState, TMsg, TCmd, TSub> _program;
  private readonly CommandExecutor<TCmd, TMsg> _commandExecutor;
  private readonly Func<Action, bool> _enqueue;
  private readonly Action<TState, TState>? _onStateChanged;
  private readonly CancellationTokenSource _cts;
  private readonly SubscriptionManager<TSub, TMsg> _subscriptionManager;

  private readonly int _maxHistorySize;
  private readonly Queue<TransitionSnapshot<TState, TMsg, TCmd, TSub>>? _messageHistory;

  private TState _state;
  private int _disposed; // 0 = alive, 1 = disposed (Interlocked for thread-safe check-and-set)

  internal MvuRuntime(
      MvuProgram<TState, TMsg, TCmd, TSub> program,
      CommandExecutor<TCmd, TMsg> commandExecutor,
      SubscriptionStarter<TSub, TMsg> subscriptionStarter,
      Func<Action, bool> enqueue,
      Action<TState>? onInit,
      Action<TState, TState>? onStateChanged,
      int maxHistorySize)
  {
    _program = program;
    _commandExecutor = commandExecutor;
    _enqueue = enqueue;
    _onStateChanged = onStateChanged;
    _cts = new CancellationTokenSource();

    // Wrap the starter so that exceptions are routed via OnRuntimeError rather than propagating
    _subscriptionManager = new SubscriptionManager<TSub, TMsg>(SafeStarter, SafeDispatchRuntimeError);

    // 0 = disabled (no allocation). Positive = ring buffer of that size.
    _maxHistorySize = maxHistorySize;
    _messageHistory = maxHistorySize > 0
        ? new Queue<TransitionSnapshot<TState, TMsg, TCmd, TSub>>(maxHistorySize)
        : null;

    try
    {
      (TState initialState, ImmutableArray<TCmd> startupCmds) = program.Init();

      bool hasCorrectiveMessage = false;
      TMsg correctiveMessage = default!;
      switch (program.Validate(initialState))
      {
        case ValidationResult<TState, TMsg>.Valid v:
          initialState = v.State;
          break;
        case ValidationResult<TState, TMsg>.Invalid i:
          startupCmds = ImmutableArray<TCmd>.Empty;
          hasCorrectiveMessage = true;
          correctiveMessage = i.Error;
          break;
      }

      _state = initialState;

      ImmutableArray<TSub> initialSubs = program.Subscriptions(initialState);
      _subscriptionManager.Diff(initialSubs, Dispatch);

      onInit?.Invoke(initialState);

      foreach (TCmd cmd in startupCmds)
      {
        ExecuteCommand(cmd);
      }

      if (hasCorrectiveMessage)
      {
        Dispatch(correctiveMessage);
      }
    }
    catch
    {
#pragma warning disable CA2012 // constructor cleanup: no ambient async context; fire-and-forget
      _ = _subscriptionManager.DisposeAsync();
#pragma warning restore CA2012
      _cts.Dispose();
      throw;
    }

    return;

    IAsyncDisposable SafeStarter(TSub sub, Dispatch<TMsg> dispatch)
    {
      Action<Exception> onError = ex =>
          SafeDispatchRuntimeError(new PambaError.SubscriptionFaulted(sub.Key, ex));

#pragma warning disable CA1031 // Runtime boundary: subscription starter exceptions routed via OnRuntimeError
      try
      {
        return subscriptionStarter(sub, dispatch, onError);
      }
      catch (Exception ex)
      {
        SafeDispatchRuntimeError(new PambaError.SubscriptionStartFailed(sub.Key, ex));
        return NoopAsyncDisposable._instance;
      }
#pragma warning restore CA1031
    }
  }

  /// <summary>
  /// Current application state. Read-only snapshot.
  /// Read on the dispatcher thread for guaranteed consistency.
  /// </summary>
  public TState State => _state;

  /// <summary>
  /// Transition history. Bounded ring buffer configured via <c>WithMaxHistorySize</c>.
  /// Empty when history is disabled (the default). Never null.
  /// </summary>
  public IReadOnlyCollection<TransitionSnapshot<TState, TMsg, TCmd, TSub>> MessageHistory =>
      _messageHistory ?? (IReadOnlyCollection<TransitionSnapshot<TState, TMsg, TCmd, TSub>>)Array.Empty<TransitionSnapshot<TState, TMsg, TCmd, TSub>>();

  private bool IsDisposed => Volatile.Read(ref _disposed) != 0;

  /// <summary>Dispatch a message into the loop. Thread-safe.</summary>
  public void Dispatch(TMsg message)
  {
    if (IsDisposed)
    {
      return;
    }

    if (!_enqueue(() => ProcessMessage(message)))
    {
      // Queue has shut down. Cannot safely mutate state - no thread context for ProcessMessage.
      // Consumer's OnRuntimeError can observe/log but the resulting message is not dispatched.
      NotifyRuntimeError(new PambaError.DispatchRejected());
    }
  }

  /// <summary>
  /// Dispatch multiple messages as a single batch.
  /// All messages are processed through Update sequentially (state flows through each).
  /// Subscription diffing and projection run once after the last message, not per message.
  /// Thread-safe.
  /// </summary>
  public void DispatchAll(params ReadOnlySpan<TMsg> messages)
  {
    if (IsDisposed)
    {
      return;
    }

    if (messages.IsEmpty)
    {
      return;
    }

    TMsg[] captured = messages.ToArray();

    if (!_enqueue(() => ProcessBatch(captured)))
    {
      NotifyRuntimeError(new PambaError.DispatchRejected());
    }
  }

  /// <inheritdoc />
  public void Dispose()
  {
    if (Interlocked.Exchange(ref _disposed, 1) != 0)
    {
      return;
    }

    _cts.Cancel();
#pragma warning disable CA2012 // synchronous Dispose cannot await; fire-and-forget is correct here per IDisposable pattern
    _ = _subscriptionManager.DisposeAsync();
#pragma warning restore CA2012
    _cts.Dispose();
    GC.SuppressFinalize(this);
  }

  /// <inheritdoc />
  public async ValueTask DisposeAsync()
  {
    if (Interlocked.Exchange(ref _disposed, 1) != 0)
    {
      return;
    }

    await _cts.CancelAsync().ConfigureAwait(false);
    await _subscriptionManager.DisposeAsync().ConfigureAwait(false);
    _cts.Dispose();
    GC.SuppressFinalize(this);
  }

  private void ProcessMessage(TMsg message)
  {
    if (IsDisposed)
    {
      return;
    }

    TState oldState = _state;
    (TState newState, ImmutableArray<TCmd> cmds) = _program.Update(message, oldState);

    bool hasCorrective = false;
    TMsg corrective = default!;
    switch (_program.Validate(newState))
    {
      case ValidationResult<TState, TMsg>.Valid v:
        newState = v.State;
        break;
      case ValidationResult<TState, TMsg>.Invalid i:
        newState = oldState;
        cmds = ImmutableArray<TCmd>.Empty;
        hasCorrective = true;
        corrective = i.Error;
        break;
    }

    _state = newState;

    ImmutableArray<TSub> newSubs = ImmutableArray<TSub>.Empty;
    if (!oldState.Equals(newState))
    {
      newSubs = _program.Subscriptions(newState);
      _subscriptionManager.Diff(newSubs, Dispatch);
      SafeInvokeProjection(oldState, newState);
    }

    if (_messageHistory is not null)
    {
      if (_messageHistory.Count >= _maxHistorySize)
      {
        _messageHistory.Dequeue();
      }

      _messageHistory.Enqueue(new TransitionSnapshot<TState, TMsg, TCmd, TSub>(
          message, oldState, newState, cmds, newSubs));
    }

    foreach (TCmd cmd in cmds)
    {
      ExecuteCommand(cmd);
    }

    if (hasCorrective)
    {
      Dispatch(corrective);
    }
  }

  private void ProcessBatch(TMsg[] messages)
  {
    if (IsDisposed)
    {
      return;
    }

    TState batchStartState = _state;
    List<TCmd> pendingCmds = new(messages.Length);
    List<TMsg> pendingCorrections = [];

    foreach (TMsg msg in messages)
    {
      TState oldState = _state;
      (TState newState, ImmutableArray<TCmd> cmds) = _program.Update(msg, oldState);

      bool hasCorrective = false;
      TMsg corrective = default!;
      switch (_program.Validate(newState))
      {
        case ValidationResult<TState, TMsg>.Valid v:
          newState = v.State;
          pendingCmds.AddRange(cmds);
          break;
        case ValidationResult<TState, TMsg>.Invalid i:
          newState = oldState;
          hasCorrective = true;
          corrective = i.Error;
          break;
      }

      _state = newState;

      if (_messageHistory is not null)
      {
        if (_messageHistory.Count >= _maxHistorySize)
        {
          _messageHistory.Dequeue();
        }

        // Subscriptions computed per-step for accurate history; diffing deferred to end of batch.
        ImmutableArray<TSub> historySubs = _program.Subscriptions(newState);
        _messageHistory.Enqueue(new TransitionSnapshot<TState, TMsg, TCmd, TSub>(
            msg, oldState, newState, hasCorrective ? ImmutableArray<TCmd>.Empty : cmds, historySubs));
      }

      if (hasCorrective)
      {
        pendingCorrections.Add(corrective);
      }
    }

    // Single subscription diff + projection after entire batch
    if (!batchStartState.Equals(_state))
    {
      ImmutableArray<TSub> finalSubs = _program.Subscriptions(_state);
      _subscriptionManager.Diff(finalSubs, Dispatch);
      SafeInvokeProjection(batchStartState, _state);
    }

    foreach (TCmd cmd in pendingCmds)
    {
      ExecuteCommand(cmd);
    }

    foreach (TMsg c in pendingCorrections)
    {
      Dispatch(c);
    }
  }

#pragma warning disable CA2012 // ValueTask fire-and-forget: command execution is intentionally not awaited; all results dispatch back as messages via CommandResult or OnRuntimeError
  private void ExecuteCommand(TCmd cmd) =>
      _ = ExecuteCommandCore(cmd);
#pragma warning restore CA2012

#pragma warning disable CA1031 // Runtime boundary: last-resort catch for unexpected throws from command executors (programming bugs)
  private async ValueTask ExecuteCommandCore(TCmd cmd)
  {
    CancellationToken token = _cts.Token;
    try
    {
      CommandResult<TMsg> result = await _commandExecutor(cmd, Dispatch, token).ConfigureAwait(false);
      if (result.HasError && !IsDisposed)
      {
        Dispatch(result.ErrorMessage!);
      }
    }
    catch (OperationCanceledException) when (token.IsCancellationRequested)
    {
      // Expected during disposal - command was cancelled
    }
    catch (Exception ex)
    {
      if (!IsDisposed)
      {
        SafeDispatchRuntimeError(
            new PambaError.CommandExecutorFailed(cmd.GetType().Name, ex));
      }
    }
  }
#pragma warning restore CA1031

#pragma warning disable CA1031 // Runtime boundary: projection failures must not propagate into the dispatch loop
  private void SafeInvokeProjection(TState oldState, TState newState)
  {
    if (_onStateChanged is null)
    {
      return;
    }

    try
    {
      _onStateChanged(oldState, newState);
    }
    catch (Exception ex)
    {
      SafeDispatchRuntimeError(new PambaError.ProjectionFailed(ex));
    }
  }
#pragma warning restore CA1031

  private void SafeDispatchRuntimeError(PambaError error)
  {
    TMsg? msg = NotifyRuntimeError(error);
    if (msg is not null)
    {
      Dispatch(msg);
    }
  }

#pragma warning disable CA1031 // OnRuntimeError is a last-resort handler; if it throws there is no typed channel left
  private TMsg? NotifyRuntimeError(PambaError error)
  {
    try
    {
      return _program.OnRuntimeError(error);
    }
    catch (Exception handlerEx)
    {
      // OnRuntimeError threw — no typed channel remains. Trace for production visibility.
      Trace.TraceError(
          $"OnRuntimeError threw an exception. Original error: {error}\nHandler exception: {handlerEx}");
      return default;
    }
  }
#pragma warning restore CA1031

  private sealed class NoopAsyncDisposable : IAsyncDisposable
  {
    internal static readonly NoopAsyncDisposable _instance = new();

    public ValueTask DisposeAsync() => ValueTask.CompletedTask;
  }
}
