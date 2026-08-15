// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Threading.Tasks;
using Xunit;

namespace Pamba.WinUI.Tests;

public sealed class PropertyChangedSubscriptionTests : IClassFixture<DispatcherQueueFixture>
{
  private readonly DispatcherQueueFixture _fixture;

  public PropertyChangedSubscriptionTests(DispatcherQueueFixture fixture) => _fixture = fixture;

  private sealed record LocaleChanged(string Tag);

  private sealed class Source : INotifyPropertyChanged
  {
    public event PropertyChangedEventHandler? PropertyChanged;

    public bool HasSubscribers => PropertyChanged is not null;

    public void Raise(string? propertyName) =>
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
  }

  [Fact]
  public async Task Change_to_the_watched_property_dispatches_on_the_dispatcher_thread()
  {
    Source source = new();
    List<LocaleChanged> dispatched = [];
    List<bool> onDispatcherThread = [];

    IAsyncDisposable handle = PropertyChangedSubscription.Start(
        source,
        "Current",
        () => new LocaleChanged("en-GB"),
        msg => { dispatched.Add(msg); onDispatcherThread.Add(_fixture.Queue.HasThreadAccess); },
        _ => Assert.Fail("no error expected"),
        _fixture.Queue);

    source.Raise("Current");
    _fixture.Drain();

    Assert.Equal(new LocaleChanged("en-GB"), Assert.Single(dispatched));
    Assert.True(Assert.Single(onDispatcherThread));

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task Change_to_another_property_is_ignored_when_a_name_filter_is_given()
  {
    Source source = new();
    List<LocaleChanged> dispatched = [];

    IAsyncDisposable handle = PropertyChangedSubscription.Start(
        source,
        "Current",
        () => new LocaleChanged("en-GB"),
        dispatched.Add,
        _ => Assert.Fail("no error expected"),
        _fixture.Queue);

    source.Raise("Unrelated");
    _fixture.Drain();

    Assert.Empty(dispatched);

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task A_null_filter_dispatches_for_every_property()
  {
    Source source = new();
    List<LocaleChanged> dispatched = [];

    IAsyncDisposable handle = PropertyChangedSubscription.Start(
        source,
        null,
        () => new LocaleChanged("en-GB"),
        dispatched.Add,
        _ => Assert.Fail("no error expected"),
        _fixture.Queue);

    source.Raise("Current");
    source.Raise("Unrelated");
    _fixture.Drain();

    Assert.Equal(2, dispatched.Count);

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task Handler_exception_reaches_onError_instead_of_escaping_to_the_event_source()
  {
    Source source = new();
    List<Exception> errors = [];
    InvalidOperationException failure = new("message factory threw");

    IAsyncDisposable handle = PropertyChangedSubscription.Start(
        source,
        "Current",
        LocaleChanged () => throw failure,
        _ => { },
        errors.Add,
        _fixture.Queue);

    source.Raise("Current");
    _fixture.Drain();

    Assert.Same(failure, Assert.Single(errors));

    await handle.DisposeAsync();
  }

  [Fact]
  public async Task Disposal_detaches_the_handler_so_later_changes_dispatch_nothing()
  {
    Source source = new();
    List<LocaleChanged> dispatched = [];

    IAsyncDisposable handle = PropertyChangedSubscription.Start(
        source,
        "Current",
        () => new LocaleChanged("en-GB"),
        dispatched.Add,
        _ => Assert.Fail("no error expected"),
        _fixture.Queue);

    Assert.True(source.HasSubscribers);

    await handle.DisposeAsync();

    Assert.False(source.HasSubscribers);

    source.Raise("Current");
    _fixture.Drain();

    Assert.Empty(dispatched);
  }

  [Fact]
  public void Start_rejects_a_missing_source_dispatch_or_queue()
  {
    Source source = new();

    Assert.Throws<ArgumentNullException>(() => PropertyChangedSubscription.Start(
        null!, "Current", () => new LocaleChanged("en-GB"), _ => { }, _ => { }, _fixture.Queue));

    Assert.Throws<ArgumentNullException>(() => PropertyChangedSubscription.Start<LocaleChanged>(
        source, "Current", null!, _ => { }, _ => { }, _fixture.Queue));

    Assert.Throws<ArgumentNullException>(() => PropertyChangedSubscription.Start(
        source, "Current", () => new LocaleChanged("en-GB"), null!, _ => { }, _fixture.Queue));

    Assert.Throws<ArgumentNullException>(() => PropertyChangedSubscription.Start(
        source, "Current", () => new LocaleChanged("en-GB"), _ => { }, _ => { }, null!));
  }
}
