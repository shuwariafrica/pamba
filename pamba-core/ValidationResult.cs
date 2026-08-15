// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;

namespace Pamba;

/// <summary>
/// The result of applying <see cref="MvuProgram{TState,TMsg,TCmd,TSub}.Validate"/> to a state.
/// Either the state is <see cref="Valid"/> (optionally normalised) or <see cref="Invalid"/>,
/// in which case the validator produces a corrective <typeparamref name="TMsg"/> to dispatch.
/// </summary>
/// <typeparam name="TState">Immutable application state.</typeparam>
/// <typeparam name="TMsg">Message type produced when the state is invalid.</typeparam>
public abstract record ValidationResult<TState, TMsg>
    where TState : IEquatable<TState>
    where TMsg : notnull
{
  private protected ValidationResult() { }

#pragma warning disable CA1034 // Sealed ADT hierarchy: nested public types form a closed discriminated union. private protected constructor prevents external derivation.
  /// <summary>The state is valid. <see cref="State"/> is the accepted (and optionally normalised) state.</summary>
  /// <param name="State">The accepted state, possibly normalised by the validator.</param>
  public sealed record Valid(TState State) : ValidationResult<TState, TMsg>;

  /// <summary>
  /// The state is invalid. The transition is rejected and <see cref="Error"/> is dispatched as a corrective message.
  /// </summary>
  /// <param name="Error">The corrective message to dispatch into the Update loop.</param>
  public sealed record Invalid(TMsg Error) : ValidationResult<TState, TMsg>;
#pragma warning restore CA1034
}
