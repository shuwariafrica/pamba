// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.UI.Dispatching;

namespace Pamba.WinUI;

/// <summary>
/// Debounces high-frequency command execution.
/// Each new invocation cancels the previous pending execution and resets the delay.
/// </summary>
/// <remarks>
/// <para>
/// Uses <see cref="System.TimeProvider"/>-based timer for reliable, testable debounce timing.
/// The default constructor uses <see cref="System.TimeProvider.System"/>; pass a custom
/// <see cref="System.TimeProvider"/> for tests.
/// </para>
/// <para>
/// Timer callbacks are marshalled back to the <see cref="DispatcherQueue"/> supplied at
/// construction, ensuring command execution and dispatch always occur on the UI thread.
/// </para>
/// <para>
/// <see cref="Execute"/> returns <c>CommandResult&lt;TMsg&gt;.Ok</c> immediately (the actual
/// execution is deferred by the debounce delay). This makes the method signature compatible
/// with <see cref="CommandExecutor{TCmd, TMsg}"/> for direct use as a wrapping executor.
/// </para>
/// </remarks>
/// <typeparam name="TCmd">Command type.</typeparam>
/// <typeparam name="TMsg">Message type.</typeparam>
public sealed class CommandDebouncer<TCmd, TMsg> : IDisposable, IAsyncDisposable
    where TCmd : notnull
{
  private readonly TimeSpan _delay;
  private readonly CommandExecutor<TCmd, TMsg> _inner;
  private readonly DispatcherQueue _dispatcherQueue;
  private readonly TimeProvider _timeProvider;

  private ITimer? _timer;
  private TCmd? _pendingCommand;
  private Dispatch<TMsg>? _pendingDispatch;
  private CancellationTokenSource? _pendingCts;
  private bool _flushed;

  /// <summary>
  /// Create a debouncer wrapping an inner command executor using the system clock.
  /// </summary>
  /// <param name="delay">Debounce delay. Each new command resets the timer.</param>
  /// <param name="inner">The actual command executor to invoke after the delay.</param>
  /// <param name="dispatcherQueue">WinUI dispatcher queue for marshalling execution back to the UI thread.</param>
  public CommandDebouncer(
      TimeSpan delay,
      CommandExecutor<TCmd, TMsg> inner,
      DispatcherQueue dispatcherQueue)
      : this(delay, inner, dispatcherQueue, TimeProvider.System) { }

  /// <summary>
  /// Create a debouncer wrapping an inner command executor using a custom <see cref="System.TimeProvider"/>.
  /// </summary>
  /// <param name="delay">Debounce delay. Each new command resets the timer.</param>
  /// <param name="inner">The actual command executor to invoke after the delay.</param>
  /// <param name="dispatcherQueue">WinUI dispatcher queue for marshalling execution back to the UI thread.</param>
  /// <param name="timeProvider">Time provider for the debounce timer. Use a fake for tests.</param>
  public CommandDebouncer(
      TimeSpan delay,
      CommandExecutor<TCmd, TMsg> inner,
      DispatcherQueue dispatcherQueue,
      TimeProvider timeProvider)
  {
    ArgumentNullException.ThrowIfNull(dispatcherQueue);
    ArgumentNullException.ThrowIfNull(timeProvider);
    _delay = delay;
    _inner = inner;
    _dispatcherQueue = dispatcherQueue;
    _timeProvider = timeProvider;
  }

  /// <summary>
  /// Schedule a command for debounced execution.
  /// Cancels any previously pending command and resets the timer.
  /// No-ops after <see cref="FlushAsync"/> or <see cref="Dispose"/>.
  /// Returns <c>CommandResult&lt;TMsg&gt;.Ok</c> immediately; the actual execution is
  /// deferred by the debounce delay. This signature is compatible with
  /// <see cref="CommandExecutor{TCmd, TMsg}"/> for use as a wrapping executor.
  /// </summary>
  public async ValueTask<CommandResult<TMsg>> Execute(TCmd command, Dispatch<TMsg> dispatch, CancellationToken ct)
  {
    Debug.Assert(
        _dispatcherQueue.HasThreadAccess,
        "CommandDebouncer.Execute must be called on the dispatcher thread.");

    if (_flushed)
    {
      return CommandResult<TMsg>.Ok;
    }

    _timer?.Dispose();
    _timer = null;

    if (_pendingCts is not null)
    {
      // ConfigureAwait(true): continuation must return to UI thread
      await _pendingCts.CancelAsync().ConfigureAwait(true);
      _pendingCts.Dispose();
    }

    _pendingCommand = command;
    _pendingDispatch = dispatch;
    _pendingCts = CancellationTokenSource.CreateLinkedTokenSource(ct);

    _timer = _timeProvider.CreateTimer(OnTimerFired, null, _delay, Timeout.InfiniteTimeSpan);

    return CommandResult<TMsg>.Ok;
  }

  /// <summary>
  /// Immediately execute any pending debounced command and stop the timer.
  /// Call during graceful shutdown to ensure pending work completes before disposal.
  /// After flushing, further <see cref="Execute"/> calls are no-ops.
  /// </summary>
  public ValueTask FlushAsync()
  {
    _timer?.Dispose();
    _timer = null;
    _flushed = true;
    return ExecutePendingAsync();
  }

  /// <summary>
  /// Cancel any pending debounced command without executing it.
  /// The debouncer remains usable for subsequent <see cref="Execute"/> calls.
  /// Use when the pending work is no longer valid (e.g. session expiry).
  /// </summary>
  public void DiscardPending()
  {
    _timer?.Dispose();
    _timer = null;
    _pendingCommand = default;
    _pendingDispatch = null;

    if (_pendingCts is not null)
    {
      _pendingCts.Cancel();
      _pendingCts.Dispose();
      _pendingCts = null;
    }
  }

  /// <inheritdoc />
  public void Dispose()
  {
    _timer?.Dispose();
    _timer = null;
    _flushed = true;
    _pendingCts?.Cancel();
    _pendingCts?.Dispose();
    GC.SuppressFinalize(this);
  }

  /// <inheritdoc />
  public async ValueTask DisposeAsync()
  {
    _timer?.Dispose();
    _timer = null;
    _flushed = true;
    if (_pendingCts is not null)
    {
      await _pendingCts.CancelAsync().ConfigureAwait(false);
      _pendingCts.Dispose();
    }

    GC.SuppressFinalize(this);
  }

  private void OnTimerFired(object? state)
  {
    if (!
#pragma warning disable CA2012 // fire-and-forget ValueTask: dispatched onto the UI thread; result handled inside ExecutePendingAsync
        _dispatcherQueue.TryEnqueue(() => _ = ExecutePendingAsync())
#pragma warning restore CA2012
        )
    {
      Trace.TraceWarning("CommandDebouncer: dispatcher queue unavailable; pending command dropped.");
    }
  }

#pragma warning disable CA1031 // Runtime boundary: last-resort catch for unexpected throws from the inner executor
  private async ValueTask ExecutePendingAsync()
  {
    if (_pendingCommand is null || _pendingDispatch is null || _pendingCts is null)
    {
      return;
    }

    TCmd cmd = _pendingCommand;
    Dispatch<TMsg> dispatch = _pendingDispatch;
    CancellationTokenSource cts = _pendingCts;

    _pendingCommand = default;
    _pendingDispatch = null;
    _pendingCts = null;

    if (!cts.Token.IsCancellationRequested)
    {
      try
      {
        CommandResult<TMsg> result = await _inner(cmd, dispatch, cts.Token).ConfigureAwait(false);
        if (result.HasError && !_flushed)
        {
          dispatch(result.ErrorMessage!);
        }
      }
      catch (OperationCanceledException) when (cts.Token.IsCancellationRequested)
      {
        // Expected cancellation during debounce reset or disposal
      }
      catch (Exception ex)
      {
        Trace.TraceError(
            $"CommandDebouncer: unexpected throw from executor ({cmd.GetType().Name}): " +
            $"{ex.GetType().Name}: {ex.Message}");
      }
    }

    cts.Dispose();
  }
#pragma warning restore CA1031
}
