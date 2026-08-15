// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System.Threading;
using System.Threading.Tasks;

namespace Pamba;

/// <summary>
/// Executes a single command and dispatches resulting messages.
/// Implemented by the Shell.
/// Returns a <see cref="CommandResult{TMsg}"/>: <c>Ok</c> on success (zero allocation),
/// or <c>Error(msg)</c> to dispatch a typed error message into the Update loop.
/// </summary>
/// <remarks>
/// The executor owns its error handling. Use <see cref="CommandResult{TMsg}"/>
/// to return typed errors at the point of failure where full context is available.
/// <c>OperationCanceledException</c> thrown during cancellation is absorbed by the runtime.
/// Unexpected throws (programming bugs) are caught by the runtime and routed via
/// <see cref="MvuProgram{TState,TMsg,TCmd,TSub}.OnRuntimeError"/> as
/// <see cref="PambaError.CommandExecutorFailed"/>.
/// </remarks>
/// <typeparam name="TCmd">Command type.</typeparam>
/// <typeparam name="TMsg">Message type.</typeparam>
/// <param name="command">The command to execute.</param>
/// <param name="dispatch">Dispatch function for resulting messages.</param>
/// <param name="cancellationToken">Cancellation token for the operation.</param>
public delegate ValueTask<CommandResult<TMsg>> CommandExecutor<in TCmd, TMsg>(
    TCmd command,
    Dispatch<TMsg> dispatch,
    CancellationToken cancellationToken);
