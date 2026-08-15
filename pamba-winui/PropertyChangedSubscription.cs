// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.ComponentModel;
using System.Diagnostics;
using Microsoft.UI.Dispatching;

namespace Pamba.WinUI;

/// <summary>
/// Creates a subscription that bridges <see cref="INotifyPropertyChanged"/> events
/// into the MVU loop. Dispatches a message when the specified property changes on the source.
/// Use inside a <see cref="SubscriptionStarter{TSub, TMsg}"/> to subscribe to external
/// observable state (e.g. Lugha <c>LocaleHost</c>, system theme providers).
/// </summary>
public static class PropertyChangedSubscription
{
  /// <summary>Start listening for property changes on an <see cref="INotifyPropertyChanged"/> source.</summary>
  /// <typeparam name="TMsg">Message type.</typeparam>
  /// <param name="source">The observable source.</param>
  /// <param name="propertyName">
  /// Property name to filter on. When <c>null</c>, all property changes dispatch a message.
  /// </param>
  /// <param name="createMessage">Factory for the message to dispatch on property change.</param>
  /// <param name="dispatch">Dispatch function.</param>
  /// <param name="onError">Error callback for handler exceptions. Provided by the runtime via <see cref="SubscriptionStarter{TSub, TMsg}"/>.</param>
  /// <param name="dispatcherQueue">
  /// WinUI dispatcher queue. Ensures <paramref name="createMessage"/> runs on the UI thread
  /// regardless of which thread raised the event.
  /// </param>
  /// <returns>An async-disposable that detaches the event handler when disposed.</returns>
  public static IAsyncDisposable Start<TMsg>(
      INotifyPropertyChanged source,
      string? propertyName,
      Func<TMsg> createMessage,
      Dispatch<TMsg> dispatch,
      Action<Exception> onError,
      DispatcherQueue dispatcherQueue)
  {
    ArgumentNullException.ThrowIfNull(source);
    ArgumentNullException.ThrowIfNull(createMessage);
    ArgumentNullException.ThrowIfNull(dispatch);
    ArgumentNullException.ThrowIfNull(dispatcherQueue);

    source.PropertyChanged += Handler;
    return new EventSubscriptionHandle(() => source.PropertyChanged -= Handler);

    void Handler(object? sender, PropertyChangedEventArgs e)
    {
      if (propertyName is null || e.PropertyName == propertyName)
      {
#pragma warning disable CA1031 // Framework boundary: handler exceptions routed via onError, must not crash the app
        if (!dispatcherQueue.TryEnqueue(() =>
        {
          try { dispatch(createMessage()); }
          catch (Exception ex) { onError(ex); }
        }))
        {
          Trace.TraceWarning("PropertyChanged dispatch rejected: queue shut down.");
        }
#pragma warning restore CA1031
      }
    }
  }
}
