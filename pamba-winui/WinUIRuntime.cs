// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using Microsoft.UI.Dispatching;

namespace Pamba.WinUI;

/// <summary>
/// Creates an <see cref="MvuRuntime{TState, TMsg, TCmd, TSub}"/> wired to
/// WinUI's <see cref="DispatcherQueue"/> for FIFO message ordering on the UI thread.
/// </summary>
public static class WinUIRuntime
{
  /// <summary>
  /// Begin constructing a WinUI-backed MVU runtime.
  /// Pre-wires the thread dispatcher to <paramref name="dispatcherQueue"/>.
  /// </summary>
  public static IWinUINeedsExecutor<TState, TMsg, TCmd, TSub>
      Create<TState, TMsg, TCmd, TSub>(
          MvuProgram<TState, TMsg, TCmd, TSub> program,
          DispatcherQueue dispatcherQueue)
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    ArgumentNullException.ThrowIfNull(program);
    ArgumentNullException.ThrowIfNull(dispatcherQueue);
    return new WinUIBuilder<TState, TMsg, TCmd, TSub>(program, dispatcherQueue);
  }

  private sealed class WinUIBuilder<TState, TMsg, TCmd, TSub>
      : IWinUINeedsExecutor<TState, TMsg, TCmd, TSub>,
        IWinUINeedsSubscriptions<TState, TMsg, TCmd, TSub>,
        IWinUIConfigurable<TState, TMsg, TCmd, TSub>,
        IWinUIReady<TState, TMsg, TCmd, TSub>
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    private readonly MvuProgram<TState, TMsg, TCmd, TSub> _program;
    private readonly DispatcherQueue _dispatcherQueue;
    private CommandExecutor<TCmd, TMsg>? _commandExecutor;
    private SubscriptionStarter<TSub, TMsg>? _subscriptionStarter;
    private Action<TState>? _onInit;
    private Action<TState, TState>? _onStateChanged;
    private int _maxHistorySize;

    internal WinUIBuilder(
        MvuProgram<TState, TMsg, TCmd, TSub> program,
        DispatcherQueue dispatcherQueue)
    {
      _program = program;
      _dispatcherQueue = dispatcherQueue;
    }

    public IWinUINeedsSubscriptions<TState, TMsg, TCmd, TSub>
        WithCommandExecutor(CommandExecutor<TCmd, TMsg> executor)
    {
      ArgumentNullException.ThrowIfNull(executor);
      _commandExecutor = executor;
      return this;
    }

    public IWinUIConfigurable<TState, TMsg, TCmd, TSub>
        WithSubscriptionStarter(SubscriptionStarter<TSub, TMsg> starter)
    {
      ArgumentNullException.ThrowIfNull(starter);
      _subscriptionStarter = starter;
      return this;
    }

    public IWinUIReady<TState, TMsg, TCmd, TSub>
        WithProjection(Action<TState, TState> onStateChanged)
    {
      ArgumentNullException.ThrowIfNull(onStateChanged);
      _onStateChanged = onStateChanged;
      return this;
    }

    public IWinUIReady<TState, TMsg, TCmd, TSub>
        WithProjection(
            Action<TState> onInit,
            Action<TState, TState> onStateChanged)
    {
      ArgumentNullException.ThrowIfNull(onInit);
      ArgumentNullException.ThrowIfNull(onStateChanged);
      _onInit = onInit;
      _onStateChanged = onStateChanged;
      return this;
    }

    public IWinUIReady<TState, TMsg, TCmd, TSub>
        WithProjection(Projection<TState> projection)
    {
      ArgumentNullException.ThrowIfNull(projection);
      _onInit = projection.ProjectInitial;
      _onStateChanged = projection.Project;
      return this;
    }

    public IWinUIReady<TState, TMsg, TCmd, TSub> WithMaxHistorySize(int maxSize)
    {
      ArgumentOutOfRangeException.ThrowIfLessThan(maxSize, 1);
      _maxHistorySize = maxSize;
      return this;
    }

    // IWinUIConfigurable exposes WithMaxHistorySize too, for the no-projection Start() path.
    IWinUIConfigurable<TState, TMsg, TCmd, TSub>
        IWinUIConfigurable<TState, TMsg, TCmd, TSub>.WithMaxHistorySize(int maxSize)
    {
      ArgumentOutOfRangeException.ThrowIfLessThan(maxSize, 1);
      _maxHistorySize = maxSize;
      return this;
    }

    public MvuRuntime<TState, TMsg, TCmd, TSub> Start()
    {
      // A false from TryEnqueue reaches MvuRuntime as PambaError.DispatchRejected.
      Func<Action, bool> enqueue = action => _dispatcherQueue.TryEnqueue(() => action());

      IRuntimeNeedsDispatcher<TState, TMsg, TCmd, TSub> withSubs = MvuRuntimeBuilder
          .Create(_program)
          .WithCommandExecutor(_commandExecutor!)
          .WithSubscriptionStarter(_subscriptionStarter!);

      IRuntimeReady<TState, TMsg, TCmd, TSub> ready;

      if (_onInit is not null)
      {
        ready = withSubs.WithDispatcher(enqueue, _onInit, _onStateChanged!);
      }
      else if (_onStateChanged is not null)
      {
        ready = withSubs.WithDispatcher(enqueue, _onStateChanged);
      }
      else
      {
        ready = withSubs.WithDispatcher(enqueue);
      }

      if (_maxHistorySize > 0)
      {
        ready = ready.WithMaxHistorySize(_maxHistorySize);
      }

      return ready.Start();
    }
  }
}

/// <summary>Needs a command executor. Provide via <c>WithCommandExecutor</c>.</summary>
public interface IWinUINeedsExecutor<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Provide the command executor (Shell concern).</summary>
  public IWinUINeedsSubscriptions<TState, TMsg, TCmd, TSub>
      WithCommandExecutor(CommandExecutor<TCmd, TMsg> executor);
}

/// <summary>Needs a subscription starter. Provide via <c>WithSubscriptionStarter</c>.</summary>
public interface IWinUINeedsSubscriptions<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Provide the subscription starter (Shell concern).</summary>
  public IWinUIConfigurable<TState, TMsg, TCmd, TSub>
      WithSubscriptionStarter(SubscriptionStarter<TSub, TMsg> starter);
}

/// <summary>Optionally add projection, history size, or call <c>Start</c> directly.</summary>
public interface IWinUIConfigurable<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Provide a state change callback for projection.</summary>
  public IWinUIReady<TState, TMsg, TCmd, TSub>
      WithProjection(Action<TState, TState> onStateChanged);

  /// <summary>Provide init and state change callbacks for projection.</summary>
  public IWinUIReady<TState, TMsg, TCmd, TSub>
      WithProjection(
          Action<TState> onInit,
          Action<TState, TState> onStateChanged);

  /// <summary>Provide a <see cref="Projection{TState}"/> for segment-based projection.</summary>
  public IWinUIReady<TState, TMsg, TCmd, TSub>
      WithProjection(Projection<TState> projection);

  /// <summary>Enable transition history with the specified ring buffer size. Disabled by default.</summary>
  public IWinUIConfigurable<TState, TMsg, TCmd, TSub> WithMaxHistorySize(int maxSize);

  /// <summary>Start without projection.</summary>
  public MvuRuntime<TState, TMsg, TCmd, TSub> Start();
}

/// <summary>Projection provided. Call <c>Start</c> to begin the dispatch loop.</summary>
public interface IWinUIReady<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Enable transition history with the specified ring buffer size. Disabled by default.</summary>
  public IWinUIReady<TState, TMsg, TCmd, TSub> WithMaxHistorySize(int maxSize);

  /// <summary>
  /// Start the runtime. Calls Init, projects initial state, and executes startup commands.
  /// </summary>
  public MvuRuntime<TState, TMsg, TCmd, TSub> Start();
}
