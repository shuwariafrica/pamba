// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Collections.Immutable;

namespace Pamba.Testing;

/// <summary>
/// Entry point for scenario-based testing of MVU programs.
/// Dispatches message sequences and provides assertion hooks at each step.
/// </summary>
public static class Scenario
{
  /// <summary>
  /// Create a scenario runner for the given program.
  /// Calls Init to establish the starting state.
  /// </summary>
  public static ScenarioRunner<TState, TMsg, TCmd, TSub>
      For<TState, TMsg, TCmd, TSub>(
          MvuProgram<TState, TMsg, TCmd, TSub> program)
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    ArgumentNullException.ThrowIfNull(program);
    var init = program.Initialize();
    return new ScenarioRunner<TState, TMsg, TCmd, TSub>(program, init);
  }

  /// <summary>
  /// Create a scenario runner for the given program and assert on the Init result.
  /// Calls Init to establish the starting state, then invokes <paramref name="assertInit"/>.
  /// </summary>
  public static ScenarioRunner<TState, TMsg, TCmd, TSub>
      For<TState, TMsg, TCmd, TSub>(
          MvuProgram<TState, TMsg, TCmd, TSub> program,
          Action<Transition<TState, TMsg, TCmd, TSub>> assertInit)
      where TState : IEquatable<TState>
      where TMsg : notnull
      where TCmd : notnull
      where TSub : IEquatable<TSub>, ISubscription<TMsg>
  {
    ArgumentNullException.ThrowIfNull(program);
    ArgumentNullException.ThrowIfNull(assertInit);
    var init = program.Initialize();
    assertInit(init);
    return new ScenarioRunner<TState, TMsg, TCmd, TSub>(program, init);
  }
}

/// <summary>
/// Fluent scenario runner for multi-step MVU testing.
/// Each <see cref="Dispatch(TMsg)"/> call advances the state machine by one message.
/// </summary>
/// <typeparam name="TState">State type.</typeparam>
/// <typeparam name="TMsg">Message type.</typeparam>
/// <typeparam name="TCmd">Command type.</typeparam>
/// <typeparam name="TSub">Subscription type.</typeparam>
public sealed class ScenarioRunner<TState, TMsg, TCmd, TSub>
    where TState : IEquatable<TState>
    where TMsg : notnull
    where TCmd : notnull
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  private readonly MvuProgram<TState, TMsg, TCmd, TSub> _program;
  private readonly List<Transition<TState, TMsg, TCmd, TSub>> _history;
  private ImmutableArray<Transition<TState, TMsg, TCmd, TSub>>? _cachedHistory;
  private TState _currentState;

  internal ScenarioRunner(
      MvuProgram<TState, TMsg, TCmd, TSub> program,
      Transition<TState, TMsg, TCmd, TSub> initResult)
  {
    _program = program;
    _currentState = initResult.State;
    _history = [initResult];
  }

  /// <summary>Current state after all dispatched messages.</summary>
  public TState State => _currentState;

  /// <summary>Full transition history including Init.</summary>
  public ImmutableArray<Transition<TState, TMsg, TCmd, TSub>> History =>
      _cachedHistory ??= [.. _history];

  /// <summary>Dispatch a message and optionally assert on the result.</summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> Dispatch(
      TMsg message,
      Action<Transition<TState, TMsg, TCmd, TSub>>? assert)
  {
    Transition<TState, TMsg, TCmd, TSub> result = _program.Step(_currentState, message);
    _currentState = result.State;
    _history.Add(result);
    _cachedHistory = null;
    assert?.Invoke(result);
    return this;
  }

  /// <summary>Dispatch a message without intermediate assertion.</summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> Dispatch(TMsg message)
  {
    return Dispatch(message, null);
  }

  /// <summary>Dispatch multiple messages in sequence without intermediate assertions.</summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> DispatchAll(
      params ReadOnlySpan<TMsg> messages)
  {
    foreach (TMsg msg in messages)
    {
      Dispatch(msg);
    }

    return this;
  }

  /// <summary>
  /// Dispatch a message and automatically process corrective messages from validation rejection.
  /// Continues dispatching corrective messages until the chain settles or <paramref name="maxDepth"/> is reached.
  /// </summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> DispatchWithCorrections(
      TMsg message,
      Action<Transition<TState, TMsg, TCmd, TSub>>? assert,
      int maxDepth)
  {
    Transition<TState, TMsg, TCmd, TSub> result = _program.Step(_currentState, message);
    _currentState = result.State;
    _history.Add(result);
    _cachedHistory = null;
    assert?.Invoke(result);

    int depth = 0;
    while (result.CorrectionMessage is not null && depth < maxDepth)
    {
      result = _program.Step(_currentState, result.CorrectionMessage);
      _currentState = result.State;
      _history.Add(result);
      _cachedHistory = null;
      depth++;
    }

    return this;
  }

  /// <summary>
  /// Dispatch a message and automatically process corrective messages from validation rejection.
  /// </summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> DispatchWithCorrections(
      TMsg message,
      int maxDepth)
  {
    return DispatchWithCorrections(message, null, maxDepth);
  }

  /// <summary>Assert on the current state.</summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> AssertState(Action<TState> assert)
  {
    ArgumentNullException.ThrowIfNull(assert);
    assert(_currentState);
    return this;
  }

  /// <summary>Assert on the full transition history.</summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> AssertHistory(
      Action<ImmutableArray<Transition<TState, TMsg, TCmd, TSub>>> assert)
  {
    ArgumentNullException.ThrowIfNull(assert);
    assert(History);
    return this;
  }

  /// <summary>Assert on the last transition result.</summary>
  public ScenarioRunner<TState, TMsg, TCmd, TSub> AssertLastTransition(
      Action<Transition<TState, TMsg, TCmd, TSub>> assert)
  {
    ArgumentNullException.ThrowIfNull(assert);
    assert(_history[^1]);
    return this;
  }
}
