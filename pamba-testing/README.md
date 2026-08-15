# Pamba.Testing

Test utilities for Pamba MVU programs. No test framework dependency - works with
xUnit, NUnit, MSTest, or any assertion library.

## Install

```shell
dotnet add package Pamba.Testing
```

Requires: `Pamba`. Namespace: `Pamba.Testing`.

## Transition Pipeline

`program.Step(state, msg)` and `program.Initialize()` live in `Pamba` (core) and
execute the canonical Update + Validate + Subscriptions sequence. They return a
`Transition<TState, TMsg, TCmd, TSub>` containing all outputs for assertion:

```csharp
Transition<AppState, Msg, Cmd, Sub> result =
    program.Step(currentState, new Msg.Increment());

Assert.Equal(1, result.State.Count);
Assert.Single(result.Commands);
Assert.IsType<Cmd.Persist>(result.Commands[0]);
Assert.Empty(result.Subscriptions);
```

When `Validate` returns `Invalid`, the state reverts, commands are dropped, and
`CorrectionMessage` is set on the result:

```csharp
Transition<AppState, Msg, Cmd, Sub> result =
    program.Step(currentState, new Msg.SetCount(-1));

Assert.True(result.WasRejected);
Assert.NotNull(result.CorrectionMessage);
```

`program.Initialize()` tests the Init path:

```csharp
Transition<AppState, Msg, Cmd, Sub> result = program.Initialize();

Assert.Equal(0, result.State.Count);
Assert.Contains(result.Commands, c => c is Cmd.LoadPreferences);
```

## Scenario

Fluent API for multi-step flow testing. Dispatches a sequence of messages through
the program, preserving state between steps. Each step optionally accepts an
assertion on the transition result.

```csharp
Scenario.For(program)
    .Dispatch(new Msg.Start(), r =>
    {
      Assert.True(r.State.IsRunning);
      Assert.Single(r.Subscriptions);
    })
    .Dispatch(new Msg.Increment(), r => Assert.Equal(1, r.State.Count))
    .Dispatch(new Msg.Increment())
    .AssertState(s => Assert.Equal(2, s.Count));
```

### API

| Method                                       | Purpose                                                                       |
| -------------------------------------------- | ----------------------------------------------------------------------------- |
| `Scenario.For(program)`                      | Create a runner. Calls `Init` to establish starting state.                    |
| `Scenario.For(program, assertInit)`          | Create a runner and assert on the Init result.                                |
| `.Dispatch(msg)`                             | Advance state by one message.                                                 |
| `.Dispatch(msg, assert)`                     | Advance and assert on the `Transition`.                                       |
| `.DispatchAll(msgs...)`                      | Advance by multiple messages in sequence (per-message validation, not batch). |
| `.DispatchWithCorrections(msg, maxDepth)`    | Advance and auto-process corrective messages from validation rejection.       |
| `.DispatchWithCorrections(msg, assert, max)` | Same with assertion on first transition.                                      |
| `.AssertState(assert)`                       | Assert on the current accumulated state.                                      |
| `.AssertHistory(assert)`                     | Assert on the full transition history.                                        |
| `.AssertLastTransition(assert)`              | Assert on the most recent transition.                                         |
| `.State`                                     | Current state after all dispatched messages.                                  |
| `.History`                                   | Immutable array of `Transition` entries, including `Init`.                    |

All methods return the runner for chaining. `History` includes the `Init` result
at index 0, followed by one entry per `Dispatch` call.

## Related Packages

- **Pamba** - Framework-agnostic core: contracts, dispatch loop, command/subscription infrastructure,
  transition pipeline (`program.Step()` / `program.Initialize()`).
- **Pamba.WinUI** - WinUI 3 integration with `DispatcherQueue`, timer/event subscription
  helpers, and command debouncer.

## Licence

Apache License 2.0
