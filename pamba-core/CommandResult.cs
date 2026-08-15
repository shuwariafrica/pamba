// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

namespace Pamba;

/// <summary>
/// Terminal result of a <see cref="CommandExecutor{TCmd, TMsg}"/> invocation.
/// Either a success (no message dispatched) or a typed error message to dispatch into the Update loop.
/// Use <see cref="CommandResultExtensions"/> for the <c>Ok</c> singleton and <c>Error</c> factory.
/// </summary>
/// <typeparam name="TMsg">Message type.</typeparam>
#pragma warning disable CA1815 // CommandResult<TMsg> is an internal-use struct compared only via HasError; equality operators are never used
public readonly struct CommandResult<TMsg>
{
  internal TMsg? ErrorMessage { get; }
  internal bool HasError { get; }

  internal CommandResult(TMsg errorMessage)
  {
    HasError = true;
    ErrorMessage = errorMessage;
  }
}
#pragma warning restore CA1815

/// <summary>
/// Factory members for <see cref="CommandResult{TMsg}"/>.
/// </summary>
#pragma warning disable CA1034 // C# 14 extension blocks appear as nested types to the analyser
#pragma warning disable CA1000 // C# 14 extension block static members appear as generic-type statics to the analyser; this is a false positive
public static class CommandResultExtensions
{
  extension<TMsg>(CommandResult<TMsg>)
  {
    /// <summary>
    /// A successful command result. Zero allocation - the default struct value.
    /// </summary>
    public static CommandResult<TMsg> Ok => default;

    /// <summary>
    /// Create an error result carrying the message to dispatch into the Update loop.
    /// </summary>
    /// <param name="errorMessage">The message to dispatch.</param>
    public static CommandResult<TMsg> Error(TMsg errorMessage) => new(errorMessage);
  }
}
#pragma warning restore CA1000
#pragma warning restore CA1034
