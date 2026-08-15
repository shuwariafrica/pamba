// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;

namespace Pamba;

/// <summary>
/// Entry point for constructing an <see cref="MvuRuntime{TState, TMsg, TCmd, TSub}"/>.
/// </summary>
public static class MvuRuntimeBuilder
{
  /// <summary>
  /// Begin constructing a runtime for the given program.
  /// </summary>
  public static IRuntimeNeedsExecutor<TState, TMsg, TCmd, TSub>
      Create<TState, TMsg, TCmd, TSub>(
          MvuProgram<TState, TMsg, TCmd, TSub> program)
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    ArgumentNullException.ThrowIfNull(program);
    return new Builder<TState, TMsg, TCmd, TSub>(program);
  }

  private sealed class Builder<TState, TMsg, TCmd, TSub>
      : IRuntimeNeedsExecutor<TState, TMsg, TCmd, TSub>,
        IRuntimeNeedsSubscriptions<TState, TMsg, TCmd, TSub>,
        IRuntimeNeedsDispatcher<TState, TMsg, TCmd, TSub>,
        IRuntimeReady<TState, TMsg, TCmd, TSub>
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    private readonly MvuProgram<TState, TMsg, TCmd, TSub> _program;
    private CommandExecutor<TCmd, TMsg>? _commandExecutor;
    private SubscriptionStarter<TSub, TMsg>? _subscriptionStarter;
    private Func<Action, bool>? _enqueue;
    private Action<TState>? _onInit;
    private Action<TState, TState>? _onStateChanged;
    private int _maxHistorySize;

    internal Builder(MvuProgram<TState, TMsg, TCmd, TSub> program)
    {
      _program = program;
    }

    public IRuntimeNeedsSubscriptions<TState, TMsg, TCmd, TSub>
        WithCommandExecutor(CommandExecutor<TCmd, TMsg> executor)
    {
      ArgumentNullException.ThrowIfNull(executor);
      _commandExecutor = executor;
      return this;
    }

    public IRuntimeNeedsDispatcher<TState, TMsg, TCmd, TSub>
        WithSubscriptionStarter(SubscriptionStarter<TSub, TMsg> starter)
    {
      ArgumentNullException.ThrowIfNull(starter);
      _subscriptionStarter = starter;
      return this;
    }

    public IRuntimeReady<TState, TMsg, TCmd, TSub>
        WithDispatcher(Func<Action, bool> enqueue)
    {
      ArgumentNullException.ThrowIfNull(enqueue);
      _enqueue = enqueue;
      return this;
    }

    public IRuntimeReady<TState, TMsg, TCmd, TSub>
        WithDispatcher(Func<Action, bool> enqueue, Action<TState, TState> onStateChanged)
    {
      ArgumentNullException.ThrowIfNull(enqueue);
      ArgumentNullException.ThrowIfNull(onStateChanged);
      _enqueue = enqueue;
      _onStateChanged = onStateChanged;
      return this;
    }

    public IRuntimeReady<TState, TMsg, TCmd, TSub>
        WithDispatcher(
            Func<Action, bool> enqueue,
            Action<TState> onInit,
            Action<TState, TState> onStateChanged)
    {
      ArgumentNullException.ThrowIfNull(enqueue);
      ArgumentNullException.ThrowIfNull(onInit);
      ArgumentNullException.ThrowIfNull(onStateChanged);
      _enqueue = enqueue;
      _onInit = onInit;
      _onStateChanged = onStateChanged;
      return this;
    }

    public IRuntimeReady<TState, TMsg, TCmd, TSub> WithMaxHistorySize(int maxSize)
    {
      ArgumentOutOfRangeException.ThrowIfLessThan(maxSize, 1);
      _maxHistorySize = maxSize;
      return this;
    }

    public MvuRuntime<TState, TMsg, TCmd, TSub> Start()
    {
      // Null-forgiving: the stepping interfaces guarantee all three are set before Start() is reachable
      return new MvuRuntime<TState, TMsg, TCmd, TSub>(
          _program,
          _commandExecutor!,
          _subscriptionStarter!,
          _enqueue!,
          _onInit,
          _onStateChanged,
          _maxHistorySize);
    }
  }
}

/// <summary>Needs a command executor. Provide via <c>WithCommandExecutor</c>.</summary>
public interface IRuntimeNeedsExecutor<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Provide the command executor (Shell concern).</summary>
  public IRuntimeNeedsSubscriptions<TState, TMsg, TCmd, TSub>
      WithCommandExecutor(CommandExecutor<TCmd, TMsg> executor);
}

/// <summary>Needs a subscription starter. Provide via <c>WithSubscriptionStarter</c>.</summary>
public interface IRuntimeNeedsSubscriptions<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Provide the subscription starter (Shell concern).</summary>
  public IRuntimeNeedsDispatcher<TState, TMsg, TCmd, TSub>
      WithSubscriptionStarter(SubscriptionStarter<TSub, TMsg> starter);
}

/// <summary>Needs a thread dispatcher. Provide via <c>WithDispatcher</c>.</summary>
public interface IRuntimeNeedsDispatcher<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>Provide the thread dispatcher for FIFO message ordering.</summary>
  public IRuntimeReady<TState, TMsg, TCmd, TSub>
      WithDispatcher(Func<Action, bool> enqueue);

  /// <summary>Provide the thread dispatcher with a state change callback for projection.</summary>
  public IRuntimeReady<TState, TMsg, TCmd, TSub>
      WithDispatcher(Func<Action, bool> enqueue, Action<TState, TState> onStateChanged);

  /// <summary>Provide the thread dispatcher with init and state change callbacks for projection.</summary>
  public IRuntimeReady<TState, TMsg, TCmd, TSub>
      WithDispatcher(
          Func<Action, bool> enqueue,
          Action<TState> onInit,
          Action<TState, TState> onStateChanged);
}

/// <summary>All dependencies provided. Call <c>Start</c> to begin the dispatch loop.</summary>
public interface IRuntimeReady<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  /// <summary>
  /// Enable transition history with the specified ring buffer size.
  /// Disabled by default (no allocation, no recording overhead).
  /// </summary>
  public IRuntimeReady<TState, TMsg, TCmd, TSub> WithMaxHistorySize(int maxSize);

  /// <summary>
  /// Start the runtime. Calls Init, validates, projects initial state, and executes startup commands.
  /// </summary>
  public MvuRuntime<TState, TMsg, TCmd, TSub> Start();
}
