// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;

namespace Pamba.WinUI.Tests;

public sealed class TimerSubscriptionTests : IClassFixture<DispatcherQueueFixture>
{
  private readonly DispatcherQueueFixture _fixture;

  public TimerSubscriptionTests(DispatcherQueueFixture fixture) => _fixture = fixture;

  private sealed record Tick(int Ordinal);

  [Fact]
  public async Task Timer_tick_dispatches_a_message_on_the_dispatcher_thread()
  {
    ControlledTimeProvider time = new();
    List<Tick> dispatched = [];
    List<bool> onDispatcherThread = [];
    int ordinal = 0;

    IAsyncDisposable handle = TimerSubscription.Start(
        TimeSpan.FromMilliseconds(50),
        () => new Tick(++ordinal),
        msg => { dispatched.Add(msg); onDispatcherThread.Add(_fixture.Queue.HasThreadAccess); },
        _ => Assert.Fail("no error expected"),
        _fixture.Queue,
        time);

    time.Advance();
    _fixture.Drain();

    Assert.Equal(new Tick(1), Assert.Single(dispatched));
    Assert.True(Assert.Single(onDispatcherThread), "tick must be marshalled onto the dispatcher thread");

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task Timer_repeats_on_its_interval_rather_than_firing_once()
  {
    ControlledTimeProvider time = new();
    TimeSpan interval = TimeSpan.FromMilliseconds(50);

    IAsyncDisposable handle = TimerSubscription.Start(
        interval,
        () => new Tick(0),
        _ => { },
        _ => Assert.Fail("no error expected"),
        _fixture.Queue,
        time);

    ControlledTimer timer = Assert.Single(time.Timers);
    Assert.Equal(interval, timer.DueTime);
    Assert.Equal(interval, timer.Period);

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task Timer_tick_handler_exception_reaches_onError_instead_of_escaping()
  {
    ControlledTimeProvider time = new();
    List<Exception> errors = [];
    InvalidOperationException failure = new("message factory threw");

    IAsyncDisposable handle = TimerSubscription.Start(
        TimeSpan.FromMilliseconds(50),
        Tick () => throw failure,
        _ => { },
        errors.Add,
        _fixture.Queue,
        time);

    time.Advance();
    _fixture.Drain();

    Assert.Same(failure, Assert.Single(errors));

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task Disposing_the_handle_disposes_the_underlying_timer()
  {
    ControlledTimeProvider time = new();

    IAsyncDisposable handle = TimerSubscription.Start(
        TimeSpan.FromMilliseconds(50),
        () => new Tick(0),
        _ => { },
        _ => { },
        _fixture.Queue,
        time);

    ControlledTimer timer = Assert.Single(time.Timers);
    await handle.DisposeAsync();

    Assert.True(timer.IsDisposed);
  }

  [Fact]
  public async Task Delayed_subscription_fires_once_and_does_not_repeat()
  {
    ControlledTimeProvider time = new();
    TimeSpan delay = TimeSpan.FromMilliseconds(200);
    List<Tick> dispatched = [];

    IAsyncDisposable handle = DelayedSubscription.Start(
        delay,
        () => new Tick(1),
        dispatched.Add,
        _ => Assert.Fail("no error expected"),
        _fixture.Queue,
        time);

    ControlledTimer timer = Assert.Single(time.Timers);
    Assert.Equal(delay, timer.DueTime);
    Assert.Equal(System.Threading.Timeout.InfiniteTimeSpan, timer.Period);

    time.Advance();
    _fixture.Drain();

    Assert.Equal(new Tick(1), Assert.Single(dispatched));

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task Delayed_subscription_handler_exception_reaches_onError()
  {
    ControlledTimeProvider time = new();
    List<Exception> errors = [];
    InvalidOperationException failure = new("message factory threw");

    IAsyncDisposable handle = DelayedSubscription.Start(
        TimeSpan.FromMilliseconds(200),
        Tick () => throw failure,
        _ => { },
        errors.Add,
        _fixture.Queue,
        time);

    time.Advance();
    _fixture.Drain();

    Assert.Same(failure, Assert.Single(errors));

    await handle.DisposeAsync();
  }
}
