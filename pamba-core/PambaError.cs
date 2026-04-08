// Copyright (c) 2026 Ali Rashid. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;

namespace Pamba;

/// <summary>
/// Sealed hierarchy of library-originated errors.
/// All errors produced by the runtime are subtypes of <see cref="PambaError"/>.
/// Routed into the MVU loop via <see cref="MvuProgram{TState,TMsg,TCmd,TSub}.OnRuntimeError"/>.
/// Variants carrying an <see cref="Exception"/> use reference equality for that field -
/// two different exception instances represent two different faults.
/// </summary>
public abstract record PambaError
{
  private protected PambaError() { }

#pragma warning disable CA1034 // Sealed ADT hierarchy: nested public types form a closed discriminated union. private protected constructor prevents external derivation.
  /// <summary>
  /// The dispatcher queue rejected a message because it has shut down.
  /// Produced when the underlying <c>TryEnqueue</c> returns <c>false</c>.
  /// </summary>
  public sealed record DispatchRejected : PambaError;

  /// <summary>
  /// A subscription starter threw an exception when starting the subscription identified by <see cref="Key"/>.
  /// </summary>
  /// <param name="Key">The subscription key whose starter threw.</param>
  /// <param name="Cause">The exception thrown by the starter.</param>
  public sealed record SubscriptionStartFailed(
      SubscriptionKey Key,
      Exception Cause) : PambaError;

  /// <summary>
  /// An active subscription threw an exception during its ongoing operation (tick handler,
  /// event handler). The subscription remains active; the exception is routed for handling.
  /// </summary>
  /// <param name="Key">The subscription key whose handler threw.</param>
  /// <param name="Cause">The exception thrown during the subscription's operation.</param>
  public sealed record SubscriptionFaulted(
      SubscriptionKey Key,
      Exception Cause) : PambaError;

  /// <summary>
  /// The <see cref="MvuProgram{TState,TMsg,TCmd,TSub}.Subscriptions"/> function returned
  /// multiple subscriptions with the same <see cref="SubscriptionKey"/>. First occurrence wins;
  /// duplicates are skipped.
  /// </summary>
  /// <param name="Key">The duplicated subscription key.</param>
  public sealed record DuplicateSubscriptionKey(SubscriptionKey Key) : PambaError;

  /// <summary>
  /// The state projection callback threw an exception during a state transition.
  /// The state transition itself completed successfully; only the UI projection failed.
  /// </summary>
  /// <param name="Cause">The exception thrown by the projection callback.</param>
  public sealed record ProjectionFailed(Exception Cause) : PambaError;

  /// <summary>
  /// A command executor threw an unexpected exception (programming bug in the executor).
  /// Well-implemented executors signal expected failures via <see cref="CommandResultExtensions"/>
  /// rather than throwing. This variant covers only unhandled throws.
  /// </summary>
  /// <param name="CommandType">The command type name.</param>
  /// <param name="Cause">The exception thrown by the executor.</param>
  public sealed record CommandExecutorFailed(
      string CommandType,
      Exception Cause) : PambaError;
#pragma warning restore CA1034
}
