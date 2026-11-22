//! The session a program runs in.
//!
//! A computer several people share, each with an account of her own, lets a person leave the screen
//! to the next one while her own work goes on running. [`switch_user`] says, once, whether this
//! computer has the act, and why not where it has not; a program draws its press where it holds the
//! act and leaves it out where it does not, and [`SwitchUser::now`] acts at the press.

use std::error::Error;
use std::fmt;
use std::io;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows::Side;

/// No platform's side is built here, so there is no value of it and no [`SwitchUser`] can exist.
#[cfg(not(windows))]
enum Side {}

/// The platform's own way to leave the screen to another person: this session stops being shown and
/// goes on running, the platform's sign-in stands for whoever comes next, and its person comes back
/// to it by her own proof.
///
/// Its `Debug` says the three facts it was offered on, each as the platform's side read it or as
/// unread: whether switching is hidden, how many enabled accounts there are, and whether the
/// computer is joined to a directory. That answers why the act is offered on a computer where it
/// seems it should not be.
pub struct SwitchUser {
    side: Side,
    read: Read,
}

/// That way, where this computer has it: the platform has the act, whoever administers the computer
/// has not hidden it, and another person can sign in here. Otherwise, why it has not.
///
/// It is read once, when it is called: an account added afterwards shows at the next call. On
/// Windows, "another person can sign in here" is more than one enabled account on the computer, or
/// a computer joined to a domain or to Entra. Every enabled account counts, an account a program
/// made for itself included, so the act can be offered on a computer only one person uses; there it
/// brings back the sign-in for her alone and harms nothing. Where Windows does not say how many
/// accounts there are or whether the computer is joined, the act is offered. Where it does not say
/// whether the administrator hid switching, it is not: the answer is [`Unoffered::Unread`].
///
/// # Errors
///
/// [`Unoffered`], with why, where the computer does not offer the act. It is the other answer, not
/// a failure: the press is left out.
///
/// # Examples
///
/// A program asks as it starts, and draws the press only where it holds the act:
///
/// ```
/// use pamba_host::session::{self, SwitchUser, Unoffered};
///
/// struct Desk {
///     switch: Result<SwitchUser, Unoffered>,
/// }
///
/// let desk = Desk {
///     switch: session::switch_user(),
/// };
/// if let Ok(switch) = &desk.switch {
///     // The press is drawn; at the press, `switch.now()`.
/// #   let _ = switch;
/// }
/// ```
pub fn switch_user() -> Result<SwitchUser, Unoffered> {
    #[cfg(windows)]
    {
        let read = windows::read();
        decide(read).map(|()| SwitchUser { side: Side, read })
    }
    #[cfg(not(windows))]
    {
        Err(Unoffered::Unbuilt)
    }
}

impl SwitchUser {
    /// Leaves the screen. It returns once the platform has taken the request, and the session
    /// leaves the screen a moment after.
    ///
    /// # Errors
    ///
    /// [`Stayed`] where the platform refused the request: the session is still on the screen.
    ///
    /// # Examples
    ///
    /// At the press, the person is told where the screen stayed hers:
    ///
    /// ```
    /// use pamba_host::session::{Stayed, SwitchUser};
    ///
    /// fn pressed(switch: &SwitchUser) -> Result<(), Stayed> {
    ///     switch.now()
    /// }
    /// # let _ = pressed;
    /// ```
    pub fn now(&self) -> Result<(), Stayed> {
        #[cfg(windows)]
        {
            self.side.leave()
        }
        #[cfg(not(windows))]
        {
            match self.side {}
        }
    }
}

#[expect(
    clippy::missing_fields_in_debug,
    reason = "the side holds nothing; what a reader needs is what the act was offered on"
)]
impl fmt::Debug for SwitchUser {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Read {
            policy,
            accounts,
            join,
        } = self.read;
        let hidden = match policy {
            Policy::Shown => "no",
            Policy::Hidden => "yes",
            Policy::Unread => "unread",
        };
        let joined = match join {
            Join::Joined => "yes",
            Join::Alone => "no",
            Join::Unread => "unread",
        };
        let mut facts = formatter.debug_struct("SwitchUser");
        facts.field("hidden", &format_args!("{hidden}"));
        match accounts {
            Accounts::Counted(accounts) => facts.field("accounts", &accounts),
            Accounts::Unread => facts.field("accounts", &format_args!("unread")),
        };
        facts.field("joined", &format_args!("{joined}")).finish()
    }
}

/// Why a computer does not offer it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unoffered {
    /// Whoever administers the computer has turned switching off.
    Hidden,
    /// Whether whoever administers the computer turned switching off could not be read.
    Unread,
    /// Nobody else can sign in here.
    NobodyElse,
    /// This platform's side is not built.
    Unbuilt,
}

impl fmt::Display for Unoffered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Hidden => "whoever administers this computer has turned switching user off",
            Self::Unread => {
                "whether whoever administers this computer turned switching user off could not be read"
            }
            Self::NobodyElse => "nobody else can sign in to this computer",
            Self::Unbuilt => "this platform's side of switching user is not built",
        })
    }
}

impl Error for Unoffered {}

/// The platform refused, and the session stayed on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stayed {
    /// The platform's own code for why, for a log.
    code: i32,
}

impl fmt::Display for Stayed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "the platform did not leave the screen to another person: {}",
            io::Error::from_raw_os_error(self.code)
        )
    }
}

impl Error for Stayed {}

/// Whether the administrator hid switching, as the platform's side read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(any(windows, test)),
    expect(
        dead_code,
        reason = "a platform's side reads each fact, and the rule is tested on every platform"
    )
)]
enum Policy {
    Shown,
    Hidden,
    Unread,
}

/// How many enabled accounts the computer holds, as the platform's side read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(any(windows, test)),
    expect(
        dead_code,
        reason = "a platform's side reads each fact, and the rule is tested on every platform"
    )
)]
enum Accounts {
    Counted(u32),
    Unread,
}

/// Whether the computer is joined to a directory, whose people can each sign in to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(any(windows, test)),
    expect(
        dead_code,
        reason = "a platform's side reads each fact, and the rule is tested on every platform"
    )
)]
enum Join {
    Joined,
    Alone,
    Unread,
}

/// The three facts the rule decides from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Read {
    policy: Policy,
    accounts: Accounts,
    join: Join,
}

/// The act, or why it is not offered. What says whether there is someone to leave the screen to,
/// unread, gives the act, since the act is harmless where there is nobody; what says the
/// administrator turned switching off, unread, withholds it.
#[cfg_attr(
    not(any(windows, test)),
    expect(
        dead_code,
        reason = "a platform's side reads each fact, and the rule is tested on every platform"
    )
)]
fn decide(read: Read) -> Result<(), Unoffered> {
    let someone = matches!(read.join, Join::Joined | Join::Unread)
        || match read.accounts {
            Accounts::Counted(accounts) => accounts > 1,
            Accounts::Unread => true,
        };
    match read.policy {
        Policy::Hidden => Err(Unoffered::Hidden),
        Policy::Unread => Err(Unoffered::Unread),
        Policy::Shown if someone => Ok(()),
        Policy::Shown => Err(Unoffered::NobodyElse),
    }
}

#[cfg(test)]
mod tests {
    use super::{Accounts, Join, Policy, Read, Unoffered, decide};

    const fn read(policy: Policy, accounts: Accounts, join: Join) -> Read {
        Read {
            policy,
            accounts,
            join,
        }
    }

    #[test]
    fn each_of_the_thirty_six_points_gives_the_answer_written_for_it() {
        use Accounts::{Counted, Unread as Uncounted};
        use Join::{Alone, Joined, Unread as Unjoined};
        use Policy::{Hidden, Shown, Unread as Unknown};
        const ACT: Result<(), Unoffered> = Ok(());
        const HIDDEN: Result<(), Unoffered> = Err(Unoffered::Hidden);
        const UNREAD: Result<(), Unoffered> = Err(Unoffered::Unread);
        const NOBODY: Result<(), Unoffered> = Err(Unoffered::NobodyElse);
        let walk = [
            (Shown, Counted(0), Joined, ACT),
            (Shown, Counted(0), Alone, NOBODY),
            (Shown, Counted(0), Unjoined, ACT),
            (Shown, Counted(1), Joined, ACT),
            (Shown, Counted(1), Alone, NOBODY),
            (Shown, Counted(1), Unjoined, ACT),
            (Shown, Counted(2), Joined, ACT),
            (Shown, Counted(2), Alone, ACT),
            (Shown, Counted(2), Unjoined, ACT),
            (Shown, Uncounted, Joined, ACT),
            (Shown, Uncounted, Alone, ACT),
            (Shown, Uncounted, Unjoined, ACT),
            (Hidden, Counted(0), Joined, HIDDEN),
            (Hidden, Counted(0), Alone, HIDDEN),
            (Hidden, Counted(0), Unjoined, HIDDEN),
            (Hidden, Counted(1), Joined, HIDDEN),
            (Hidden, Counted(1), Alone, HIDDEN),
            (Hidden, Counted(1), Unjoined, HIDDEN),
            (Hidden, Counted(2), Joined, HIDDEN),
            (Hidden, Counted(2), Alone, HIDDEN),
            (Hidden, Counted(2), Unjoined, HIDDEN),
            (Hidden, Uncounted, Joined, HIDDEN),
            (Hidden, Uncounted, Alone, HIDDEN),
            (Hidden, Uncounted, Unjoined, HIDDEN),
            (Unknown, Counted(0), Joined, UNREAD),
            (Unknown, Counted(0), Alone, UNREAD),
            (Unknown, Counted(0), Unjoined, UNREAD),
            (Unknown, Counted(1), Joined, UNREAD),
            (Unknown, Counted(1), Alone, UNREAD),
            (Unknown, Counted(1), Unjoined, UNREAD),
            (Unknown, Counted(2), Joined, UNREAD),
            (Unknown, Counted(2), Alone, UNREAD),
            (Unknown, Counted(2), Unjoined, UNREAD),
            (Unknown, Uncounted, Joined, UNREAD),
            (Unknown, Uncounted, Alone, UNREAD),
            (Unknown, Uncounted, Unjoined, UNREAD),
        ];
        let points: Vec<Read> = walk
            .iter()
            .map(|&(policy, accounts, join, _)| read(policy, accounts, join))
            .collect();
        for (first, point) in points.iter().enumerate() {
            assert!(
                !points.iter().skip(first + 1).any(|other| other == point),
                "the walk holds {point:?} twice"
            );
        }
        assert_eq!(points.len(), 36);
        for (point, (.., given)) in points.into_iter().zip(walk) {
            assert_eq!(decide(point), given, "{point:?}");
        }
    }

    #[test]
    fn two_accounts_with_nothing_hidden_give_the_act() {
        assert_eq!(
            decide(read(Policy::Shown, Accounts::Counted(2), Join::Alone)),
            Ok(())
        );
    }

    #[test]
    fn one_account_on_a_computer_joined_to_nothing_gives_nobody_else() {
        assert_eq!(
            decide(read(Policy::Shown, Accounts::Counted(1), Join::Alone)),
            Err(Unoffered::NobodyElse)
        );
    }

    #[test]
    fn one_account_on_a_joined_computer_gives_the_act() {
        assert_eq!(
            decide(read(Policy::Shown, Accounts::Counted(1), Join::Joined)),
            Ok(())
        );
    }

    #[test]
    fn switching_hidden_gives_hidden_whatever_else_holds() {
        for accounts in [
            Accounts::Counted(0),
            Accounts::Counted(1),
            Accounts::Counted(2),
            Accounts::Unread,
        ] {
            for join in [Join::Joined, Join::Alone, Join::Unread] {
                let point = read(Policy::Hidden, accounts, join);
                assert_eq!(decide(point), Err(Unoffered::Hidden), "{point:?}");
            }
        }
    }

    #[test]
    fn a_count_that_could_not_be_read_gives_the_act() {
        assert_eq!(
            decide(read(Policy::Shown, Accounts::Unread, Join::Alone)),
            Ok(())
        );
    }

    #[test]
    fn a_join_that_could_not_be_read_gives_the_act() {
        assert_eq!(
            decide(read(Policy::Shown, Accounts::Counted(1), Join::Unread)),
            Ok(())
        );
    }

    #[test]
    fn a_policy_that_could_not_be_read_gives_unread() {
        assert_eq!(
            decide(read(Policy::Unread, Accounts::Counted(2), Join::Joined)),
            Err(Unoffered::Unread)
        );
    }
}
