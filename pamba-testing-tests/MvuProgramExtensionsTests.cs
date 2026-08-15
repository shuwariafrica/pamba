// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using Xunit;

namespace Pamba.Testing.Tests;

/// <summary>
/// Tests for <see cref="MvuProgramExtensions"/> (<c>Step</c> and <c>Initialize</c>)
/// and <see cref="TransitionExtensions"/>.
/// </summary>
public sealed class MvuProgramExtensionsTests
{
  private sealed record TestState(int Count);

  private abstract record TestMsg
  {
    internal sealed record SetCount(int Value) : TestMsg;
    internal sealed record CountWasNegative : TestMsg;
  }

  private abstract record TestCmd
  {
    internal sealed record Persist(int Value) : TestCmd;
  }

  private sealed record TestSub(SubscriptionKey Key) : ISubscription<TestMsg>;

  private static readonly MvuProgram<TestState, TestMsg, TestCmd, TestSub> _program = new()
  {
    Init = () => (new TestState(0), []),
    Update = (msg, state) => msg switch
    {
      TestMsg.SetCount s => (new TestState(s.Value), [new TestCmd.Persist(s.Value)]),
      _ => (state, [])
    },
    Subscriptions = state => state.Count > 0
        ? [new TestSub(SubscriptionKey.From("tick-timer"))]
        : [],
    OnRuntimeError = err => throw new InvalidOperationException($"Unexpected runtime error: {err}"),
    Validate = state => state.Count >= 0
        ? new ValidationResult<TestState, TestMsg>.Valid(state)
        : new ValidationResult<TestState, TestMsg>.Invalid(new TestMsg.CountWasNegative())
  };

  [Fact]
  public void Initialize_returns_initial_state_and_subscriptions()
  {
    Transition<TestState, TestMsg, TestCmd, TestSub> result = _program.Initialize();

    Assert.Equal(0, result.State.Count);
    Assert.Empty(result.Commands);
    Assert.Empty(result.Subscriptions);
    Assert.Null(result.Message);
    Assert.Null(result.CorrectionMessage);
  }

  [Fact]
  public void Step_returns_state_commands_and_subscriptions()
  {
    TestState initial = new(0);
    Transition<TestState, TestMsg, TestCmd, TestSub> result =
        _program.Step(initial, new TestMsg.SetCount(5));

    Assert.Equal(5, result.State.Count);
    Assert.Single(result.Commands);
    Assert.IsType<TestCmd.Persist>(result.Commands[0]);
    Assert.Single(result.Subscriptions);
    Assert.Equal("tick-timer", result.Subscriptions[0].Key.Value);
    Assert.Null(result.CorrectionMessage);
  }

  [Fact]
  public void Step_returns_correction_message_when_validation_rejects()
  {
    TestState initial = new(0);
    Transition<TestState, TestMsg, TestCmd, TestSub> result =
        _program.Step(initial, new TestMsg.SetCount(-1));

    // Transition rejected: state reverts to initial, commands dropped
    Assert.Equal(0, result.State.Count);
    Assert.Empty(result.Commands);
    Assert.NotNull(result.CorrectionMessage);
    Assert.IsType<TestMsg.CountWasNegative>(result.CorrectionMessage);
  }

  [Fact]
  public void Transition_extension_WasRejected_reflects_correction()
  {
    TestState initial = new(0);

    Transition<TestState, TestMsg, TestCmd, TestSub> accepted =
        _program.Step(initial, new TestMsg.SetCount(5));
    Assert.True(accepted.WasAccepted);
    Assert.False(accepted.WasRejected);
    Assert.True(accepted.HasCommands);
    Assert.True(accepted.HasSubscriptions);

    Transition<TestState, TestMsg, TestCmd, TestSub> rejected =
        _program.Step(initial, new TestMsg.SetCount(-1));
    Assert.True(rejected.WasRejected);
    Assert.False(rejected.WasAccepted);
    Assert.False(rejected.HasCommands);
    Assert.False(rejected.HasSubscriptions);
  }
}
