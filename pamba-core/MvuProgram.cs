// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Immutable;

namespace Pamba;

/// <summary>
/// A complete MVU program definition.
/// All functions are pure. The runtime executes them.
/// See <see cref="MvuRuntime{TState, TMsg, TCmd, TSub}"/> for execution.
/// </summary>
/// <remarks>
/// A configuration object. Its identity is referential; equality is not meaningful.
/// </remarks>
/// <typeparam name="TState">Immutable application state. Must implement value equality.</typeparam>
/// <typeparam name="TMsg">Message type - sealed record hierarchy.</typeparam>
/// <typeparam name="TCmd">Command type - sealed record hierarchy describing effects.</typeparam>
/// <typeparam name="TSub">Subscription type - sealed record hierarchy describing ongoing effects.</typeparam>
public sealed class MvuProgram<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Returns the initial state and startup commands.</summary>
  /// <remarks>
  /// When <see cref="Validate"/> rejects the initial state, there is no previous state to revert to.
  /// The runtime keeps the initial state, drops startup commands, and dispatches the corrective message
  /// after construction completes. With a synchronous dispatcher the corrective message processes
  /// immediately; with an async dispatcher the invalid initial state is briefly visible.
  /// </remarks>
  public required Func<(TState State, ImmutableArray<TCmd> Commands)> Init { get; init; }

  /// <summary>Pure state transition function. Total over all (TMsg, TState) inputs.</summary>
  public required Func<TMsg, TState, (TState State, ImmutableArray<TCmd> Commands)> Update { get; init; }

  /// <summary>Returns the set of active subscriptions for the current state.</summary>
  public required Func<TState, ImmutableArray<TSub>> Subscriptions { get; init; }

  /// <summary>
  /// Maps a library-originated <see cref="PambaError"/> to a message for the Update loop.
  /// Ensures runtime errors (subscription start failures, dispatch rejections, error handler
  /// failures, projection failures, unexpected command executor throws) are routed as typed
  /// values into the Update loop rather than silently lost.
  /// </summary>
  public required Func<PambaError, TMsg> OnRuntimeError { get; init; }

  /// <summary>
  /// State validator invoked after every transition.
  /// Returns <see cref="ValidationResult{TState, TMsg}.Valid"/> with the accepted (optionally normalised) state,
  /// or <see cref="ValidationResult{TState, TMsg}.Invalid"/> with a corrective message to dispatch.
  /// A total function - never throws.
  /// Assign <see cref="ValidationResultExtensions">ValidationResult&lt;TState, TMsg&gt;.AlwaysValid</see>
  /// when structural validation is not needed.
  /// </summary>
  public required Func<TState, ValidationResult<TState, TMsg>> Validate { get; init; }
}
