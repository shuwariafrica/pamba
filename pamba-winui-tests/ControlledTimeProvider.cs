// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Threading;
using System.Threading.Tasks;

namespace Pamba.WinUI.Tests;

/// <summary>
/// A <see cref="TimeProvider"/> whose timers never elapse on their own; <see cref="Advance"/>
/// elapses them.
/// </summary>
public sealed class ControlledTimeProvider : TimeProvider
{
  private readonly List<ControlledTimer> _timers = [];

  /// <summary>Timers created and not yet disposed, in creation order.</summary>
  public ReadOnlyCollection<ControlledTimer> Timers => _timers.AsReadOnly();

  public override ITimer CreateTimer(TimerCallback callback, object? state, TimeSpan dueTime, TimeSpan period)
  {
    ControlledTimer timer = new(callback, state, dueTime, period, _timers.Remove);
    _timers.Add(timer);
    return timer;
  }

  /// <summary>Elapse every live timer once, in creation order.</summary>
  public void Advance()
  {
    foreach (ControlledTimer timer in _timers.ToArray())
    {
      timer.Elapse();
    }
  }
}

/// <summary>
/// Records the schedule it was constructed with so a test can assert the caller requested
/// the interval it meant to.
/// </summary>
public sealed class ControlledTimer : ITimer
{
  private readonly TimerCallback _callback;
  private readonly object? _state;
  private readonly Func<ControlledTimer, bool> _onDisposed;

  internal ControlledTimer(
      TimerCallback callback,
      object? state,
      TimeSpan dueTime,
      TimeSpan period,
      Func<ControlledTimer, bool> onDisposed)
  {
    _callback = callback;
    _state = state;
    _onDisposed = onDisposed;
    DueTime = dueTime;
    Period = period;
  }

  public TimeSpan DueTime { get; private set; }

  public TimeSpan Period { get; private set; }

  public bool IsDisposed { get; private set; }

  public void Elapse() => _callback(_state);

  public bool Change(TimeSpan dueTime, TimeSpan period)
  {
    DueTime = dueTime;
    Period = period;
    return true;
  }

  public void Dispose()
  {
    IsDisposed = true;
    _onDisposed(this);
  }

  public ValueTask DisposeAsync()
  {
    Dispose();
    return ValueTask.CompletedTask;
  }
}
