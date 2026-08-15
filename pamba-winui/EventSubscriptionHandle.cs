// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Threading.Tasks;

namespace Pamba.WinUI;

/// <summary>
/// Async-disposable handle that invokes an unsubscribe action on disposal.
/// Used by event-based subscription helpers to detach event handlers.
/// </summary>
internal sealed class EventSubscriptionHandle(Action unsubscribe) : IAsyncDisposable
{
  public ValueTask DisposeAsync()
  {
    unsubscribe();
    return ValueTask.CompletedTask;
  }
}
