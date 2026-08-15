// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Threading;
using System.Threading.Tasks;

namespace Pamba.WinUI;

/// <summary>
/// Async-disposable handle that disposes an <see cref="ITimer"/> returned by
/// <see cref="TimeProvider.CreateTimer"/>.
/// </summary>
internal sealed class TimerProviderHandle(ITimer timer) : IAsyncDisposable
{
  public async ValueTask DisposeAsync()
  {
    await timer.DisposeAsync().ConfigureAwait(false);
  }
}
