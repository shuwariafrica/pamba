// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Xunit;

namespace Pamba.WinUI.Tests;

public sealed class CommandDebouncerTests : IClassFixture<DispatcherQueueFixture>
{
  private readonly DispatcherQueueFixture _fixture;

  public CommandDebouncerTests(DispatcherQueueFixture fixture) => _fixture = fixture;

  private sealed record Search(string Term);

  private sealed record Msg(string Detail);

  private static readonly TimeSpan _delay = TimeSpan.FromMilliseconds(300);

  private CommandDebouncer<Search, Msg> Create(
      List<Search> executed,
      ControlledTimeProvider time) =>
      new(
          _delay,
          (cmd, dispatch, ct) =>
          {
            executed.Add(cmd);
            return ValueTask.FromResult(CommandResult<Msg>.Ok);
          },
          _fixture.Queue,
          time);

  [Fact]
  public async Task Nothing_executes_until_the_debounce_delay_elapses()
  {
    ControlledTimeProvider time = new();
    List<Search> executed = [];
    await using CommandDebouncer<Search, Msg> debouncer = Create(executed, time);

    await debouncer.Execute(new Search("a"), _ => { }, CancellationToken.None);

    Assert.Empty(executed);
    Assert.Equal(_delay, Assert.Single(time.Timers).DueTime);
  }

  [Fact]
  public async Task Elapsing_the_delay_executes_the_pending_command()
  {
    ControlledTimeProvider time = new();
    List<Search> executed = [];
    await using CommandDebouncer<Search, Msg> debouncer = Create(executed, time);

    await debouncer.Execute(new Search("a"), _ => { }, CancellationToken.None);
    time.Advance();
    _fixture.Drain();

    Assert.Equal(new Search("a"), Assert.Single(executed));
  }

  [Fact]
  public async Task Only_the_last_command_of_a_burst_executes()
  {
    ControlledTimeProvider time = new();
    List<Search> executed = [];
    await using CommandDebouncer<Search, Msg> debouncer = Create(executed, time);

    await debouncer.Execute(new Search("a"), _ => { }, CancellationToken.None);
    await debouncer.Execute(new Search("ab"), _ => { }, CancellationToken.None);
    await debouncer.Execute(new Search("abc"), _ => { }, CancellationToken.None);

    time.Advance();
    _fixture.Drain();

    Assert.Equal(new Search("abc"), Assert.Single(executed));
  }

  [Fact]
  public async Task Execute_returns_Ok_immediately_so_it_composes_as_a_command_executor()
  {
    ControlledTimeProvider time = new();
    List<Search> executed = [];
    await using CommandDebouncer<Search, Msg> debouncer = Create(executed, time);

    CommandExecutor<Search, Msg> asExecutor = debouncer.Execute;
    CommandResult<Msg> result = await asExecutor(new Search("a"), _ => { }, CancellationToken.None);

    Assert.Equal(CommandResult<Msg>.Ok, result);
  }

  [Fact]
  public async Task FlushAsync_executes_the_pending_command_and_closes_the_debouncer()
  {
    ControlledTimeProvider time = new();
    List<Search> executed = [];
    await using CommandDebouncer<Search, Msg> debouncer = Create(executed, time);

    await debouncer.Execute(new Search("a"), _ => { }, CancellationToken.None);
    await debouncer.FlushAsync();

    Assert.Equal(new Search("a"), Assert.Single(executed));

    await debouncer.Execute(new Search("b"), _ => { }, CancellationToken.None);
    time.Advance();
    _fixture.Drain();

    Assert.Single(executed);
  }

  [Fact]
  public async Task DiscardPending_drops_the_command_and_leaves_the_debouncer_usable()
  {
    ControlledTimeProvider time = new();
    List<Search> executed = [];
    await using CommandDebouncer<Search, Msg> debouncer = Create(executed, time);

    await debouncer.Execute(new Search("a"), _ => { }, CancellationToken.None);
    debouncer.DiscardPending();

    time.Advance();
    _fixture.Drain();
    Assert.Empty(executed);

    await debouncer.Execute(new Search("b"), _ => { }, CancellationToken.None);
    time.Advance();
    _fixture.Drain();

    Assert.Equal(new Search("b"), Assert.Single(executed));
  }

  [Fact]
  public async Task An_error_from_the_inner_executor_is_dispatched_into_the_loop()
  {
    ControlledTimeProvider time = new();
    List<Msg> dispatched = [];

    await using CommandDebouncer<Search, Msg> debouncer = new(
        _delay,
        (cmd, dispatch, ct) => ValueTask.FromResult(CommandResult<Msg>.Error(new Msg("db down"))),
        _fixture.Queue,
        time);

    await debouncer.Execute(new Search("a"), dispatched.Add, CancellationToken.None);
    time.Advance();
    _fixture.Drain();

    Assert.Equal(new Msg("db down"), Assert.Single(dispatched));
  }

  [Fact]
  public async Task Disposal_stops_the_timer_and_makes_further_scheduling_a_no_op()
  {
    ControlledTimeProvider time = new();
    List<Search> executed = [];
    CommandDebouncer<Search, Msg> debouncer = Create(executed, time);

    await debouncer.Execute(new Search("a"), _ => { }, CancellationToken.None);
    ControlledTimer timer = Assert.Single(time.Timers);

    await debouncer.DisposeAsync();

    Assert.True(timer.IsDisposed);

    await debouncer.Execute(new Search("b"), _ => { }, CancellationToken.None);
    time.Advance();
    _fixture.Drain();

    Assert.Empty(executed);
  }

  [Fact]
  public void Construction_rejects_a_missing_queue_or_time_provider()
  {
    CommandExecutor<Search, Msg> inner =
        (cmd, dispatch, ct) => ValueTask.FromResult(CommandResult<Msg>.Ok);

    Assert.Throws<ArgumentNullException>(() =>
        new CommandDebouncer<Search, Msg>(_delay, inner, null!));

    Assert.Throws<ArgumentNullException>(() =>
        new CommandDebouncer<Search, Msg>(_delay, inner, _fixture.Queue, null!));
  }
}
