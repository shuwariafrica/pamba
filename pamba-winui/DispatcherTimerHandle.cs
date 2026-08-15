// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Threading.Tasks;
using Microsoft.UI.Dispatching;

namespace Pamba.WinUI;

/// <summary>
/// Async-disposable handle that stops a <see cref="DispatcherQueueTimer"/> on disposal.
/// Shared by <see cref="TimerSubscription"/> and <see cref="DelayedSubscription"/>.
/// </summary>
internal sealed class DispatcherTimerHandle(DispatcherQueueTimer timer) : IAsyncDisposable
{
  public ValueTask DisposeAsync()
  {
    timer.Stop();
    return ValueTask.CompletedTask;
  }
}
