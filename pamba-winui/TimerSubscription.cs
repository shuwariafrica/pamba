// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Diagnostics;
using System.Threading;
using Microsoft.UI.Dispatching;

namespace Pamba.WinUI;

/// <summary>
/// Creates a timer-based subscription that dispatches a message at a fixed interval.
/// Disposed when the subscription is removed.
/// </summary>
/// <remarks>
/// The overload that accepts a <see cref="System.TimeProvider"/> uses
/// <see cref="TimeProvider.CreateTimer"/> for testable timing; callbacks are marshalled
/// back to the <see cref="DispatcherQueue"/> via <see cref="DispatcherQueue.TryEnqueue(DispatcherQueueHandler)"/>.
/// The overload without <see cref="System.TimeProvider"/> uses
/// <see cref="DispatcherQueueTimer"/> and fires directly on the UI thread.
/// </remarks>
public static class TimerSubscription
{
  /// <summary>Start a repeating timer subscription using the system clock.</summary>
  /// <typeparam name="TMsg">Message type.</typeparam>
  /// <param name="interval">Time between dispatches.</param>
  /// <param name="createMessage">Factory for the message to dispatch on each tick.</param>
  /// <param name="dispatch">Dispatch function.</param>
  /// <param name="onError">Error callback for tick handler exceptions. Provided by the runtime via <see cref="SubscriptionStarter{TSub, TMsg}"/>.</param>
  /// <param name="dispatcherQueue">WinUI dispatcher queue.</param>
  /// <returns>An async-disposable that stops the timer when disposed.</returns>
  public static IAsyncDisposable Start<TMsg>(
      TimeSpan interval,
      Func<TMsg> createMessage,
      Dispatch<TMsg> dispatch,
      Action<Exception> onError,
      DispatcherQueue dispatcherQueue)
  {
    ArgumentNullException.ThrowIfNull(dispatcherQueue);
    var timer = dispatcherQueue.CreateTimer();
    timer.Interval = interval;
    timer.IsRepeating = true;
    timer.Tick += (_, _) =>
    {
#pragma warning disable CA1031 // Framework boundary: tick handler exceptions routed via onError, must not crash the app
      try { dispatch(createMessage()); }
      catch (Exception ex) { onError(ex); }
#pragma warning restore CA1031
    };
    timer.Start();
    return new DispatcherTimerHandle(timer);
  }

  /// <summary>
  /// Start a repeating timer subscription using a custom <see cref="System.TimeProvider"/>.
  /// Callbacks are marshalled to the UI thread via <paramref name="dispatcherQueue"/>.
  /// </summary>
  /// <typeparam name="TMsg">Message type.</typeparam>
  /// <param name="interval">Time between dispatches.</param>
  /// <param name="createMessage">Factory for the message to dispatch on each tick.</param>
  /// <param name="dispatch">Dispatch function.</param>
  /// <param name="onError">Error callback for tick handler exceptions.</param>
  /// <param name="dispatcherQueue">WinUI dispatcher queue.</param>
  /// <param name="timeProvider">Time provider. Use a fake for tests.</param>
  /// <returns>An async-disposable that stops the timer when disposed.</returns>
  public static IAsyncDisposable Start<TMsg>(
      TimeSpan interval,
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
#pragma warning disable CA1031 // Framework boundary: tick handler exceptions routed via onError, must not crash the app
            try { dispatch(createMessage()); }
            catch (Exception ex) { onError(ex); }
#pragma warning restore CA1031
          }))
          {
            Trace.TraceWarning("TimerSubscription: tick dispatch rejected; queue shut down.");
          }
        },
        null,
        interval,
        interval);

    return new TimerProviderHandle(timer);
  }
}
