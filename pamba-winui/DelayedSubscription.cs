// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Diagnostics;
using System.Threading;
using Microsoft.UI.Dispatching;

namespace Pamba.WinUI;

/// <summary>
/// Creates a one-shot delayed subscription that dispatches a message
/// after a specified delay. Disposed if cancelled before firing.
/// </summary>
/// <remarks>
/// The overload that accepts a <see cref="System.TimeProvider"/> uses
/// <see cref="TimeProvider.CreateTimer"/> for testable timing.
/// </remarks>
public static class DelayedSubscription
{
  /// <summary>Start a one-shot delayed subscription using the system clock.</summary>
  /// <typeparam name="TMsg">Message type.</typeparam>
  /// <param name="delay">Delay before dispatching.</param>
  /// <param name="createMessage">Factory for the message to dispatch.</param>
  /// <param name="dispatch">Dispatch function.</param>
  /// <param name="onError">Error callback for handler exceptions. Provided by the runtime via <see cref="SubscriptionStarter{TSub, TMsg}"/>.</param>
  /// <param name="dispatcherQueue">WinUI dispatcher queue.</param>
  /// <returns>An async-disposable that cancels the delay when disposed.</returns>
  public static IAsyncDisposable Start<TMsg>(
      TimeSpan delay,
      Func<TMsg> createMessage,
      Dispatch<TMsg> dispatch,
      Action<Exception> onError,
      DispatcherQueue dispatcherQueue)
  {
    ArgumentNullException.ThrowIfNull(dispatcherQueue);
    var timer = dispatcherQueue.CreateTimer();
    timer.Interval = delay;
    timer.IsRepeating = false;
    timer.Tick += (_, _) =>
    {
#pragma warning disable CA1031 // Framework boundary: handler exceptions routed via onError, must not crash the app
      try { dispatch(createMessage()); }
      catch (Exception ex) { onError(ex); }
#pragma warning restore CA1031
    };
    timer.Start();
    return new DispatcherTimerHandle(timer);
  }

  /// <summary>
  /// Start a one-shot delayed subscription using a custom <see cref="System.TimeProvider"/>.
  /// Callbacks are marshalled to the UI thread via <paramref name="dispatcherQueue"/>.
  /// </summary>
  /// <typeparam name="TMsg">Message type.</typeparam>
  /// <param name="delay">Delay before dispatching.</param>
  /// <param name="createMessage">Factory for the message to dispatch.</param>
  /// <param name="dispatch">Dispatch function.</param>
  /// <param name="onError">Error callback for handler exceptions.</param>
  /// <param name="dispatcherQueue">WinUI dispatcher queue.</param>
  /// <param name="timeProvider">Time provider. Use a fake for tests.</param>
  /// <returns>An async-disposable that cancels the delay when disposed.</returns>
  public static IAsyncDisposable Start<TMsg>(
      TimeSpan delay,
      Func<TMsg> createMessage,
      Dispatch<TMsg> dispatch,
      Action<Exception> onError,
      DispatcherQueue dispatcherQueue,
      TimeProvider timeProvider)
  {
    ArgumentNullException.ThrowIfNull(dispatcherQueue);
    ArgumentNullException.ThrowIfNull(timeProvider);

    ITimer timer = timeProvider.CreateTimer(
        _ =>
        {
          if (!dispatcherQueue.TryEnqueue(() =>
          {
#pragma warning disable CA1031 // Framework boundary: handler exceptions routed via onError, must not crash the app
            try { dispatch(createMessage()); }
            catch (Exception ex) { onError(ex); }
#pragma warning restore CA1031
          }))
          {
            Trace.TraceWarning("DelayedSubscription: tick dispatch rejected; queue shut down.");
          }
        },
        null,
        delay,
        Timeout.InfiniteTimeSpan);

    return new TimerProviderHandle(timer);
  }
}
