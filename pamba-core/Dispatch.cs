// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

namespace Pamba;

/// <summary>
/// Dispatches a message into the MVU loop.
/// Thread-safe. Messages are queued and processed in FIFO order.
/// </summary>
/// <typeparam name="TMsg">Message type.</typeparam>
/// <param name="message">The message to dispatch.</param>
public delegate void Dispatch<in TMsg>(TMsg message);
