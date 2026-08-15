// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Immutable;
using System.Diagnostics;

namespace Pamba;

/// <summary>
/// Transition pipeline operations on <see cref="MvuProgram{TState, TMsg, TCmd, TSub}"/>.
/// These execute the canonical Update + Validate + Subscriptions sequence as a single
/// atomic operation. Used for composition (host delegating to guest programs) and testing.
/// </summary>
#pragma warning disable CA1034 // C# 14 extension blocks appear as nested types to the analyser
public static class MvuProgramExtensions
{
  extension<TState, TMsg, TCmd, TSub>(MvuProgram<TState, TMsg, TCmd, TSub> program)
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    /// <summary>
    /// Execute a single transition: Update, then Validate, then Subscriptions.
    /// When Validate rejects, state reverts, commands are dropped,
    /// and <see cref="Transition{TState, TMsg, TCmd, TSub}.CorrectionMessage"/> is set.
    /// </summary>
    /// <param name="currentState">State before the transition.</param>
    /// <param name="message">The message to process.</param>
    public Transition<TState, TMsg, TCmd, TSub> Step(TState currentState, TMsg message)
    {
      (TState newState, ImmutableArray<TCmd> cmds) = program.Update(message, currentState);
      TMsg? correctionMessage = default;

      ValidationResult<TState, TMsg> validation = program.Validate(newState);
      switch (validation)
      {
        case ValidationResult<TState, TMsg>.Valid v:
          newState = v.State;
          break;
        case ValidationResult<TState, TMsg>.Invalid i:
          newState = currentState;
          cmds = ImmutableArray<TCmd>.Empty;
          correctionMessage = i.Error;
          break;
        default:
          throw new UnreachableException($"unhandled {validation.GetType().Name}");
      }

      ImmutableArray<TSub> subs = program.Subscriptions(newState);
      return new Transition<TState, TMsg, TCmd, TSub>(newState, message, correctionMessage, cmds, subs);
    }

    /// <summary>
    /// Execute initialisation: Init, then Validate, then Subscriptions.
    /// <see cref="Transition{TState, TMsg, TCmd, TSub}.Message"/> is <c>null</c>
    /// (no triggering message for Init).
    /// </summary>
    public Transition<TState, TMsg, TCmd, TSub> Initialize()
    {
      (TState initialState, ImmutableArray<TCmd> cmds) = program.Init();
      TMsg? correctionMessage = default;

      ValidationResult<TState, TMsg> validation = program.Validate(initialState);
      switch (validation)
      {
        case ValidationResult<TState, TMsg>.Valid v:
          initialState = v.State;
          break;
        case ValidationResult<TState, TMsg>.Invalid i:
          cmds = ImmutableArray<TCmd>.Empty;
          correctionMessage = i.Error;
          break;
        default:
          throw new UnreachableException($"unhandled {validation.GetType().Name}");
      }

      ImmutableArray<TSub> subs = program.Subscriptions(initialState);
      return new Transition<TState, TMsg, TCmd, TSub>(initialState, default, correctionMessage, cmds, subs);
    }
  }
}
#pragma warning restore CA1034
