// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Threading;
using Microsoft.UI.Dispatching;
using Xunit;

namespace Pamba.WinUI.Tests;

/// <summary>
/// A dedicated <see cref="DispatcherQueue"/> thread, shared by one test class.
/// </summary>
public sealed class DispatcherQueueFixture : IDisposable
{
  private static readonly TimeSpan _timeout = TimeSpan.FromSeconds(10);

  private readonly DispatcherQueueController _controller;

  public DispatcherQueueFixture()
  {
    _controller = DispatcherQueueController.CreateOnDedicatedThread();
    Queue = _controller.DispatcherQueue;
  }

  public DispatcherQueue Queue { get; }

  /// <summary>Run <paramref name="action"/> on the queue thread and wait for it to finish.</summary>
  public void Run(Action action)
  {
    ArgumentNullException.ThrowIfNull(action);

    using ManualResetEventSlim done = new();
    Exception? failure = null;

    Assert.True(Queue.TryEnqueue(() =>
    {
      try
      {
        action();
      }
#pragma warning disable CA1031 // the failure is rethrown on the calling thread below
      catch (Exception ex)
      {
        failure = ex;
      }
#pragma warning restore CA1031
      finally
      {
        done.Set();
      }
    }));

    Assert.True(done.Wait(_timeout), "dispatcher queue did not run the action");

    if (failure is not null)
    {
      throw new InvalidOperationException("action failed on the dispatcher thread", failure);
    }
  }

  /// <summary>Wait until every item already on the queue has been processed.</summary>
  public void Drain() => Run(static () => { });

  public void Dispose()
  {
    // ShutdownQueueAsync must be initiated from the queue's own thread.
    using ManualResetEventSlim shutdown = new();
    if (Queue.TryEnqueue(() => _ = _controller.ShutdownQueueAsync()))
    {
      Queue.ShutdownCompleted += (_, _) => shutdown.Set();
      shutdown.Wait(_timeout);
    }
  }
}
