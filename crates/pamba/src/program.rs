/// A program's logic: state that answers each message by changing and by asking for effects, and
/// that says what it stays subscribed to.
///
/// Nothing here names a platform, a thread or a clock, so the same program runs under any
/// platform's window and is tested by calling [`update`](Self::update) and comparing what it
/// returns:
///
/// ```
/// use pamba::{Never, Program};
///
/// struct Counter {
///     count: u32,
/// }
///
/// enum Message {
///     Increment,
///     Saved,
/// }
///
/// #[derive(Debug, PartialEq)]
/// enum Effect {
///     Save(u32),
/// }
///
/// impl Program for Counter {
///     type Message = Message;
///     type Effect = Effect;
///     type Subscription = Never;
///     type Moment = ();
///
///     fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
///         match message {
///             Message::Increment => {
///                 self.count += 1;
///                 vec![Effect::Save(self.count)]
///             }
///             Message::Saved => Vec::new(),
///         }
///     }
/// }
///
/// let mut counter = Counter { count: 0 };
/// assert_eq!(counter.update(Message::Increment, &()), [Effect::Save(1)]);
/// assert_eq!(counter.count, 1);
/// ```
pub trait Program {
    /// Something that happened: what a person did, what an effect came to, what a subscription
    /// told.
    type Message;

    /// A side effect the program asks for once, as data. A [`Performer`](crate::Performer) carries
    /// it out and sends whatever comes of it back as a message.
    type Effect;

    /// Something outside the program that it attends to for as long as its state says so: a
    /// connection's frames, a clock's next change, a file's changes. Two that compare equal are
    /// the same subscription.
    type Subscription: PartialEq;

    /// What the world reads as at the instant a step runs, in the clocks this program counts on:
    /// read once a step by the [`Performer`](crate::Performer), so `update` never reads a clock
    /// itself. `()` for a program that reads none.
    type Moment;

    /// Answers `message`: changes the state and returns the effects to perform, in order.
    ///
    /// The result depends on the state, `message` and `now` alone.
    fn update(&mut self, message: Self::Message, now: &Self::Moment) -> Vec<Self::Effect>;

    /// What the program is subscribed to in its present state, in the order to start them.
    ///
    /// The loop reads it after every step and brings what runs into line: a subscription no
    /// longer returned is stopped, and nothing it sent after that reaches `update`; one newly
    /// returned is started. A cancellable piece of work is therefore a subscription held by the
    /// state that wants it, and ends when its own message changes that state.
    fn subscriptions(&self) -> Vec<Self::Subscription> {
        Vec::new()
    }
}

/// The effect of a program that asks for none, and the subscription of one that holds none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Never {}
