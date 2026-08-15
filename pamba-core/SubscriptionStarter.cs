// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;

namespace Pamba;

/// <summary>
/// Creates and manages the lifecycle of a single subscription.
/// Implemented by the Shell per subscription type.
/// Returns an <see cref="IAsyncDisposable"/> that cancels the subscription when disposed.
/// </summary>
/// <remarks>
/// <para>
/// The <paramref name="onError"/> callback routes exceptions that occur during the
/// subscription's ongoing operation (tick handlers, event handlers) into the MVU loop
/// as <see cref="PambaError.SubscriptionFaulted"/>. The runtime provides this callback;
/// the Shell passes it through to subscription helpers.
/// </para>
/// <para>
/// Exceptions thrown from the starter itself (before returning <see cref="IAsyncDisposable"/>)
/// are caught by the runtime and routed separately as <see cref="PambaError.SubscriptionStartFailed"/>.
/// </para>
/// </remarks>
/// <typeparam name="TSub">Subscription type.</typeparam>
/// <typeparam name="TMsg">Message type.</typeparam>
/// <param name="subscription">The subscription to start.</param>
/// <param name="dispatch">Dispatch function for messages produced by the subscription.</param>
/// <param name="onError">Error callback for exceptions during the subscription's ongoing operation.</param>
/// <returns>An async disposable handle that cancels the subscription when disposed.</returns>
public delegate IAsyncDisposable SubscriptionStarter<in TSub, TMsg>(
    TSub subscription,
    Dispatch<TMsg> dispatch,
    Action<Exception> onError)
    where TSub : ISubscription<TMsg>;
