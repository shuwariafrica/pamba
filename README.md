# Pamba

*Pamba* (Swahili: to decorate, to adorn) is the loop a native program runs on, written in Rust.

A program is state that answers each message by changing and by returning the side effects it wants, as data. It
names no platform, no thread and no clock, so the same program runs under any platform's window, and each of its
rules is tested with a function call and a comparison.

```rust
use pamba::{Never, Program};

struct Counter {
    count: u32,
}

enum Message {
    Increment,
    Saved,
}

#[derive(Debug, PartialEq)]
enum Effect {
    Save(u32),
}

impl Program for Counter {
    type Message = Message;
    type Effect = Effect;
    type Subscription = Never;
    type Moment = ();

    fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
        match message {
            Message::Increment => {
                self.count += 1;
                vec![Effect::Save(self.count)]
            }
            Message::Saved => Vec::new(),
        }
    }
}

let mut counter = Counter { count: 0 };
assert_eq!(counter.update(Message::Increment, &()), [Effect::Save(1)]);
```

## What a program is made of

| | What it is | Who writes it |
| --- | --- | --- |
| `Program` | The state, how it answers a message, and what it stays subscribed to. | Once, for every platform. |
| `Effect` | A side effect asked for once, as a value: save this, send that. | The program's own type. |
| `Subscription` | Something attended to while the state says so: a connection, a clock, a file. | The program's own type. |
| `Moment` | What the clocks read when a step runs, handed to `update`. | The program's own type. |
| `Performer` | Carries the effects out and runs the subscriptions on one platform. | Once a platform. |
| `Loop` | Runs a program with its performer on the thread that draws. | Pamba. |

## What the loop guarantees

- One message at a time. A step's effects are performed, in the order returned, before the next message is taken.
- When an effect is performed, exactly the subscriptions of the state that asked for it are running.
- Nothing a subscription sent reaches the program once that subscription is stopped. A piece of work that can be
  cancelled or superseded is a subscription held by the state that wants it.
- Messages from one sender arrive in the order sent, from any thread.
- The mailbox is bounded. A source on another thread that outruns the loop waits; nothing is dropped. The loop's own
  thread never waits.
- A long burst is taken a bounded number of messages at a time, so the thread that draws is never held.

## Testing a program

A rule of the program is a call of `update` and a comparison of what it returns. A program together with its real
performer runs under `pamba::Manual`, a dispatcher driven by hand: no window, no second thread, and the case decides
the order everything happens in.

## Crates

| Crate | Builds on | What it is |
| --- | --- | --- |
| `pamba` | every platform | The loop. No dependency and no `unsafe`. |
| `pamba-windows` | Windows | The Windows layer: a program drawn with WinUI 3. Empty elsewhere. |

## Verifying a release

Each release is signed with Pamba's own key, whose fingerprint is

```
7C96D0926D61D7F76544BD010E0E86A8608B7CC5
```

[SECURITY.md](https://github.com/shuwariafrica/pamba/blob/main/SECURITY.md) checks a release against it, with a
GitHub login or without one.

## Licence

Apache License 2.0
