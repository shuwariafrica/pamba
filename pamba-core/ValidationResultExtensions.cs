// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;

namespace Pamba;

/// <summary>
/// Extension members for <see cref="ValidationResult{TState, TMsg}"/>.
/// </summary>
#pragma warning disable CA1034 // C# 14 extension blocks appear as nested types to the analyser
#pragma warning disable CA1000 // C# 14 extension block static members appear as generic-type statics to the analyser; this is a false positive
public static class ValidationResultExtensions
{
  extension<TState, TMsg>(ValidationResult<TState, TMsg>)
      where TState : IEquatable<TState>
      where TMsg : notnull
  {
    /// <summary>
    /// A validator that accepts every state unchanged.
    /// Assign to <see cref="MvuProgram{TState,TMsg,TCmd,TSub}.Validate"/> when structural validation is not needed.
    /// </summary>
    public static Func<TState, ValidationResult<TState, TMsg>> AlwaysValid =>
        static state => new ValidationResult<TState, TMsg>.Valid(state);
  }
}
#pragma warning restore CA1000
#pragma warning restore CA1034
