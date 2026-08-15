// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Collections.Immutable;

namespace Pamba;

/// <summary>
/// Base class for state-to-UI projection with automatic diffing.
/// Subclass and register domain-specific projection segments in the constructor.
/// Each segment fires only when its selected sub-state changes by value equality.
/// Framework-agnostic - usable with any UI layer that calls <see cref="Project"/> and
/// <see cref="ProjectInitial"/> on the appropriate thread.
/// </summary>
/// <typeparam name="TState">Immutable application state.</typeparam>
public abstract class Projection<TState>
    where TState : IEquatable<TState>
{
  private readonly ImmutableArray<Action<TState, TState>>.Builder _diffBuilder;
  private readonly ImmutableArray<Action<TState>>.Builder _initBuilder;
  private ImmutableArray<Action<TState, TState>> _diffSegments;
  private ImmutableArray<Action<TState>> _initSegments;
  private bool _frozen;

  /// <summary>
  /// Initialises the projection base. Subclasses register segments via
  /// <see cref="Segment{TSegment}(Func{TState, TSegment}, Action{TSegment})"/> in their constructor.
  /// </summary>
  protected Projection()
  {
    _diffBuilder = ImmutableArray.CreateBuilder<Action<TState, TState>>();
    _initBuilder = ImmutableArray.CreateBuilder<Action<TState>>();
    _diffSegments = ImmutableArray<Action<TState, TState>>.Empty;
    _initSegments = ImmutableArray<Action<TState>>.Empty;
  }

  private void Freeze()
  {
    if (_frozen)
    {
      return;
    }
    _frozen = true;
    _diffSegments = _diffBuilder.ToImmutable();
    _initSegments = _initBuilder.ToImmutable();
  }

  /// <summary>
  /// Register a projection segment: a selector extracting a sub-state,
  /// and an action to update UI when that sub-state changes.
  /// </summary>
  /// <typeparam name="TSegment">The sub-state type. Must implement value equality.</typeparam>
  /// <param name="selector">Extracts the sub-state from the full state.</param>
  /// <param name="project">Updates the UI for the changed sub-state. Called on the UI thread.</param>
  protected void Segment<TSegment>(
      Func<TState, TSegment> selector,
      Action<TSegment> project)
      where TSegment : IEquatable<TSegment>
  {
    _diffBuilder.Add((oldState, newState) =>
    {
      TSegment oldSegment = selector(oldState);
      TSegment newSegment = selector(newState);

      if (!EqualityComparer<TSegment>.Default.Equals(oldSegment, newSegment))
      {
        project(newSegment);
      }
    });

    _initBuilder.Add(state => project(selector(state)));
  }

  /// <summary>
  /// Register a transition-aware projection segment: a selector extracting a sub-state,
  /// an action for initial projection, and an action receiving both old and new values on transitions.
  /// Use this overload when the projection needs the previous value to determine transition
  /// behaviour (e.g. animation direction, diff highlighting).
  /// </summary>
  /// <typeparam name="TSegment">The sub-state type. Must implement value equality.</typeparam>
  /// <param name="selector">Extracts the sub-state from the full state.</param>
  /// <param name="projectInitial">Sets UI from the initial value. Called once at startup.</param>
  /// <param name="projectTransition">
  /// Updates the UI when the sub-state changes. Receives the previous and current values.
  /// Called on the UI thread.
  /// </param>
  protected void Segment<TSegment>(
      Func<TState, TSegment> selector,
      Action<TSegment> projectInitial,
      Action<TSegment, TSegment> projectTransition)
      where TSegment : IEquatable<TSegment>
  {
    _diffBuilder.Add((oldState, newState) =>
    {
      TSegment oldSegment = selector(oldState);
      TSegment newSegment = selector(newState);

      if (!EqualityComparer<TSegment>.Default.Equals(oldSegment, newSegment))
      {
        projectTransition(oldSegment, newSegment);
      }
    });

    _initBuilder.Add(state => projectInitial(selector(state)));
  }

  /// <summary>
  /// Called on the UI thread after every state transition.
  /// Evaluates all registered segments and invokes those whose sub-state changed.
  /// </summary>
  /// <param name="oldState">State before the transition.</param>
  /// <param name="newState">State after the transition.</param>
  public void Project(TState oldState, TState newState)
  {
    if (oldState.Equals(newState))
    {
      return;
    }

    Freeze();

    foreach (Action<TState, TState> segment in _diffSegments)
    {
      segment(oldState, newState);
    }
  }

  /// <summary>
  /// Force-project all segments from a default/initial state.
  /// Called once during startup to set initial UI state.
  /// Fires every registered segment projector with <paramref name="initialState"/>.
  /// </summary>
  /// <param name="initialState">The initial state to project.</param>
  public void ProjectInitial(TState initialState)
  {
    Freeze();

    foreach (Action<TState> segment in _initSegments)
    {
      segment(initialState);
    }
  }
}
