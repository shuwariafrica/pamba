// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System.Collections.Immutable;

namespace Pamba;

/// <summary>
/// Recorded only when history is enabled through <c>WithMaxHistorySize</c>.
/// <see cref="Subscriptions"/> is the set active after the transition whether or not the
/// state changed. A rejected transition is recorded too, with <see cref="StateBefore"/>
/// equal to <see cref="StateAfter"/> and <see cref="Commands"/> empty.
/// </summary>
/// <typeparam name="TState">State type.</typeparam>
/// <typeparam name="TMsg">Message type.</typeparam>
/// <typeparam name="TCmd">Command type.</typeparam>
/// <typeparam name="TSub">Subscription type.</typeparam>
public sealed record TransitionSnapshot<TState, TMsg, TCmd, TSub>(
    TMsg Message,
    TState StateBefore,
    TState StateAfter,
    ImmutableArray<TCmd> Commands,
    ImmutableArray<TSub> Subscriptions);
