// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;

namespace Pamba;

/// <summary>
/// Ergonomic extension members for <see cref="Transition{TState, TMsg, TCmd, TSub}"/>.
/// </summary>
#pragma warning disable CA1034 // C# 14 extension blocks appear as nested types to the analyser
public static class TransitionExtensions
{
  extension<TState, TMsg, TCmd, TSub>(Transition<TState, TMsg, TCmd, TSub> t)
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    /// <summary>True when the validator rejected the transition.</summary>
    public bool WasRejected => t.CorrectionMessage is not null;

    /// <summary>True when the transition was accepted (no correction needed).</summary>
    public bool WasAccepted => t.CorrectionMessage is null;

    /// <summary>True when at least one command was produced.</summary>
    public bool HasCommands => !t.Commands.IsEmpty;

    /// <summary>True when at least one subscription is active.</summary>
    public bool HasSubscriptions => !t.Subscriptions.IsEmpty;
  }
}
#pragma warning restore CA1034
