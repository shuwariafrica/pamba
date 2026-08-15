// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System.Collections.Generic;
using Xunit;

namespace Pamba.Tests;

public sealed class ProjectionTests
{
  private sealed record TestState(int Count, string Label) : System.IEquatable<TestState>;

  private sealed class TestProjection : Projection<TestState>
  {
    public List<int> CountProjections { get; } = [];
    public List<string> LabelProjections { get; } = [];

    public TestProjection()
    {
      Segment(s => s.Count, count => CountProjections.Add(count));
      Segment(s => s.Label, label => LabelProjections.Add(label));
    }
  }

  private sealed class TransitionAwareProjection : Projection<TestState>
  {
    public List<int> InitProjections { get; } = [];
    public List<(int Old, int New)> TransitionProjections { get; } = [];
    public List<string> LabelProjections { get; } = [];

    public TransitionAwareProjection()
    {
      Segment(
          s => s.Count,
          count => InitProjections.Add(count),
          (old, @new) => TransitionProjections.Add((old, @new)));
      Segment(s => s.Label, label => LabelProjections.Add(label));
    }
  }

  [Fact]
  public void Project_fires_only_changed_segments()
  {
    TestProjection projection = new();
    TestState oldState = new(1, "hello");
    TestState newState = new(2, "hello"); // Only Count changed

    projection.Project(oldState, newState);

    Assert.Single(projection.CountProjections);
    Assert.Equal(2, projection.CountProjections[0]);
    Assert.Empty(projection.LabelProjections); // Label unchanged, not projected
  }

  [Fact]
  public void Project_fires_all_changed_segments()
  {
    TestProjection projection = new();
    TestState oldState = new(1, "hello");
    TestState newState = new(2, "world"); // Both changed

    projection.Project(oldState, newState);

    Assert.Single(projection.CountProjections);
    Assert.Single(projection.LabelProjections);
  }

  [Fact]
  public void Project_skips_entirely_when_state_unchanged()
  {
    TestProjection projection = new();
    TestState state = new(1, "hello");

    projection.Project(state, state);

    Assert.Empty(projection.CountProjections);
    Assert.Empty(projection.LabelProjections);
  }

  [Fact]
  public void ProjectInitial_default_implementation_projects_all_segments()
  {
    TestProjection projection = new();
    TestState initial = new(5, "start");

    projection.ProjectInitial(initial);

    Assert.Single(projection.CountProjections);
    Assert.Equal(5, projection.CountProjections[0]);
    Assert.Single(projection.LabelProjections);
    Assert.Equal("start", projection.LabelProjections[0]);
  }

  [Fact]
  public void Transition_segment_receives_old_and_new_values()
  {
    TransitionAwareProjection projection = new();
    TestState oldState = new(1, "hello");
    TestState newState = new(3, "hello");

    projection.Project(oldState, newState);

    Assert.Single(projection.TransitionProjections);
    Assert.Equal((1, 3), projection.TransitionProjections[0]);
    Assert.Empty(projection.InitProjections); // Init not called during Project
    Assert.Empty(projection.LabelProjections); // Label unchanged
  }

  [Fact]
  public void Transition_segment_skips_when_unchanged()
  {
    TransitionAwareProjection projection = new();
    TestState oldState = new(1, "hello");
    TestState newState = new(1, "world"); // Count unchanged, label changed

    projection.Project(oldState, newState);

    Assert.Empty(projection.TransitionProjections); // Count unchanged
    Assert.Single(projection.LabelProjections);
    Assert.Equal("world", projection.LabelProjections[0]);
  }

  [Fact]
  public void Transition_segment_projectInitial_fires_on_startup()
  {
    TransitionAwareProjection projection = new();
    TestState initial = new(7, "start");

    projection.ProjectInitial(initial);

    Assert.Single(projection.InitProjections);
    Assert.Equal(7, projection.InitProjections[0]);
    Assert.Empty(projection.TransitionProjections); // Transition not called during init
    Assert.Single(projection.LabelProjections);
    Assert.Equal("start", projection.LabelProjections[0]);
  }
}
