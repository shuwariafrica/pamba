// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Immutable;

namespace Pamba;

/// <summary>
/// The result of a single program transition (Update + Validate + Subscriptions)
/// or initialisation (Init + Validate + Subscriptions).
/// Returned by <see cref="MvuProgramExtensions"/> for composition and testing.
/// </summary>
/// <typeparam name="TState">State type.</typeparam>
/// <typeparam name="TMsg">Message type.</typeparam>
/// <typeparam name="TCmd">Command type.</typeparam>
/// <typeparam name="TSub">Subscription type.</typeparam>
/// <param name="State">The state after the transition (or the pre-rejection state if <see cref="CorrectionMessage"/> is non-null).</param>
/// <param name="Message">The message that triggered this transition. <c>null</c> for Init results.</param>
/// <param name="CorrectionMessage">Non-null when the validator rejected the transition. The corrective message the validator produced.</param>
/// <param name="Commands">Commands returned by Update. Empty when the transition was rejected by the validator.</param>
/// <param name="Subscriptions">Active subscriptions for the current state.</param>
public sealed record Transition<TState, TMsg, TCmd, TSub>(
    TState State,
    TMsg? Message,
    TMsg? CorrectionMessage,
    ImmutableArray<TCmd> Commands,
    ImmutableArray<TSub> Subscriptions)
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>;
