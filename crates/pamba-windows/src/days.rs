//! A day of the calendar, and Windows' date picker made to hold one.
//!
//! Windows' date picker holds an instant, and shows the day that instant falls on where the
//! computer is. A program holds a day: a booking's, a birth's. Between the two stand the
//! computer's time zone and a time of day nobody chose, and a program that crosses them itself
//! shows or reads the day before in one zone or another. The crossing is made here, once. A day
//! is given to the picker as noon of that day where the computer is. The picker's first and
//! last days are given the same way, since Windows makes every day it offers from the first, at
//! the first's own time of day. The instant the picker reports is read as the day it falls on
//! where the computer is.

#![allow(
    unsafe_code,
    reason = "a day is placed in the computer's time zone through Windows' C interface, and nothing safe beneath this crate offers it"
)]

use std::error::Error;
use std::ffi::c_void;
use std::fmt;
use std::ops::{Bound, RangeBounds};
use std::ptr::null;

use windows_reactor::{CalendarDatePicker, DateTime, IntoPayloadCallback};

/// The first and last years a [`CalendarDay`] is of.
const YEARS: (i16, i16) = (1602, 9999);
/// How far a picker reaches each side of today where a program gives it no bound: Windows' own
/// reach for its picker, in years.
const REACH: i16 = 100;

/// A day of the calendar: a year, a month and a day of the month, with no time of day and no
/// time zone. It is what a date picker holds, and a program's own date converts to it and from
/// it by those three figures.
///
/// The figures are the Gregorian calendar's, whichever calendar Windows shows the person. The
/// years are 1602 to 9999: Windows counts time from 1601, and 1602 is the first year whose
/// every day it holds in every time zone.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CalendarDay {
    year: i16,
    month: u8,
    day: u8,
}

impl CalendarDay {
    /// The day of that year, month (1 to 12) and day of the month. Each figure is taken as a
    /// program's own date gives it, in whichever whole-number type that is.
    ///
    /// # Errors
    ///
    /// [`NoDay::NoSuchDay`] where the calendar has no such day, whatever its year.
    /// [`NoDay::YearOutside`] where the calendar has the day, and its year is outside 1602 to
    /// 9999.
    pub fn new(
        year: impl Into<i64>,
        month: impl TryInto<u8>,
        day: impl TryInto<u8>,
    ) -> Result<Self, NoDay> {
        let year = year.into();
        // A figure too large or too small for a month or a day of one is no figure of either.
        let (Ok(month), Ok(day)) = (month.try_into(), day.try_into()) else {
            return Err(NoDay::NoSuchDay);
        };
        if !((1..=12).contains(&month) && (1..=days_in(year, month)).contains(&day)) {
            return Err(NoDay::NoSuchDay);
        }
        match i16::try_from(year) {
            Ok(year) if (YEARS.0..=YEARS.1).contains(&year) => Ok(Self { year, month, day }),
            _ => Err(NoDay::YearOutside),
        }
    }

    /// The year.
    #[must_use]
    pub const fn year(self) -> i16 {
        self.year
    }

    /// The month, from 1 to 12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, from 1.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }

    /// The day after. The last day of all has none, and gives itself.
    fn next(self) -> Self {
        if self.day < days_in(self.year.into(), self.month) {
            Self {
                day: self.day + 1,
                ..self
            }
        } else if self.month < 12 {
            Self {
                month: self.month + 1,
                day: 1,
                ..self
            }
        } else if self.year < YEARS.1 {
            Self {
                year: self.year + 1,
                month: 1,
                day: 1,
            }
        } else {
            self
        }
    }

    /// The day before. The first day of all has none, and gives itself.
    fn previous(self) -> Self {
        if self.day > 1 {
            Self {
                day: self.day - 1,
                ..self
            }
        } else if self.month > 1 {
            let month = self.month - 1;
            Self {
                month,
                day: days_in(self.year.into(), month),
                ..self
            }
        } else if self.year > YEARS.0 {
            Self {
                year: self.year - 1,
                month: 12,
                day: 31,
            }
        } else {
            self
        }
    }

    /// The same day of the year that many years on, or back where the count is below nought:
    /// the last day of its month where that year's month is shorter, and of the first or the
    /// last year where the count would pass it.
    fn years_on(self, years: i16) -> Self {
        let year = self.year.saturating_add(years).clamp(YEARS.0, YEARS.1);
        Self {
            year,
            month: self.month,
            day: self.day.min(days_in(year.into(), self.month)),
        }
    }
}

/// Why three figures are no [`CalendarDay`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NoDay {
    /// The calendar has no such day: the month is not one of 1 to 12, or that month of that
    /// year has no such day of the month.
    NoSuchDay,
    /// The calendar has the day, and its year is outside 1602 to 9999, the years a
    /// [`CalendarDay`] is of.
    YearOutside,
}

impl fmt::Display for NoDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchDay => formatter.write_str("the calendar has no such day"),
            Self::YearOutside => write!(
                formatter,
                "the day's year is outside {} to {}",
                YEARS.0, YEARS.1
            ),
        }
    }
}

impl Error for NoDay {}

/// Whether that year has a twenty-ninth of February, by the Gregorian calendar's rule carried
/// to every year.
const fn leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// How many days that month of that year has.
const fn days_in(year: i64, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => 31,
    }
}

/// The days a date picker offers, and the day it holds among them, or that it holds none.
///
/// A picker shows the day it holds only where it offers that day: Windows moves any other to
/// the nearer end of the days offered, and shows that end. So the two are made together, and a
/// day that is not offered is refused where they are made, with the day and the same days
/// holding none. No picker is given a day it does not offer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Days {
    held: Option<CalendarDay>,
    /// The first day offered, where the program states one.
    first: Option<CalendarDay>,
    /// The last day offered, where the program states one: never before a first it states.
    last: Option<CalendarDay>,
}

impl Days {
    /// Any day offered, holding `held` or no day. A day converts to this, and so does an
    /// `Option` of one.
    #[must_use]
    pub fn any(held: impl Into<Option<CalendarDay>>) -> Self {
        Self {
            held: held.into(),
            first: None,
            last: None,
        }
    }

    /// The days `within` offered, holding `held` or no day: `first..=last` for those two and
    /// the days between, or a range open at one end. Days that would be none are their first
    /// day alone.
    ///
    /// # Errors
    ///
    /// [`NotOffered`] where `held` is a day before the first day `within` states, or after the
    /// last.
    pub fn within(
        held: impl Into<Option<CalendarDay>>,
        within: impl RangeBounds<CalendarDay>,
    ) -> Result<Self, NotOffered> {
        let first = match within.start_bound() {
            Bound::Included(day) => Some(*day),
            Bound::Excluded(day) => Some(day.next()),
            Bound::Unbounded => None,
        };
        let last = match within.end_bound() {
            Bound::Included(day) => Some(*day),
            Bound::Excluded(day) => Some(day.previous()),
            Bound::Unbounded => None,
        };
        let last = last.map(|last| first.map_or(last, |first| last.max(first)));
        let offered = Self {
            held: None,
            first,
            last,
        };
        match held.into() {
            Some(day)
                if first.is_some_and(|first| day < first)
                    || last.is_some_and(|last| day > last) =>
            {
                Err(NotOffered { held: day, offered })
            }
            held => Ok(Self { held, ..offered }),
        }
    }

    /// The day held.
    #[must_use]
    pub const fn held(self) -> Option<CalendarDay> {
        self.held
    }
}

impl From<CalendarDay> for Days {
    fn from(held: CalendarDay) -> Self {
        Self::any(held)
    }
}

impl From<Option<CalendarDay>> for Days {
    fn from(held: Option<CalendarDay>) -> Self {
        Self::any(held)
    }
}

/// Why [`Days`] were not made: the day to hold is not one of the days offered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotOffered {
    held: CalendarDay,
    offered: Days,
}

impl NotOffered {
    /// The day that is not offered.
    #[must_use]
    pub const fn held(self) -> CalendarDay {
        self.held
    }

    /// The same days offered, holding no day: what a picker is given by a program that says
    /// the day in words of its own beside it.
    #[must_use]
    pub const fn offered(self) -> Days {
        self.offered
    }
}

impl fmt::Display for NotOffered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02} is not one of the days offered",
            self.held.year, self.held.month, self.held.day
        )
    }
}

impl Error for NotOffered {}

/// The first and the last day a picker is given for `days`. Each is the program's where it
/// states one. An end it leaves open stands [`REACH`] years from `today`, as Windows' picker
/// stands when given no bound: further where the day held lies beyond that, and [`REACH`]
/// years beyond the end the program does state where that end lies beyond it. So the picker
/// always offers the day it holds, and never a day beyond an end the program states.
fn span(days: Days, today: CalendarDay) -> (CalendarDay, CalendarDay) {
    let first = days.first.unwrap_or_else(|| {
        let reach = today.years_on(-REACH);
        let reach = match days.last {
            Some(last) if last < reach => last.years_on(-REACH),
            _ => reach,
        };
        days.held.map_or(reach, |held| held.min(reach))
    });
    let last = days.last.unwrap_or_else(|| {
        let reach = today.years_on(REACH);
        let reach = match days.first {
            Some(first) if first > reach => first.years_on(REACH),
            _ => reach,
        };
        days.held.map_or(reach, |held| held.max(reach))
    });
    (first, last)
}

/// A date and a time of day as Windows' C interface states them (`SYSTEMTIME`).
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Clock {
    year: u16,
    month: u16,
    weekday: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    millisecond: u16,
}

/// An instant as Windows' C interface counts it, in two halves (`FILETIME`): the count
/// [`DateTime`] holds whole.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Count {
    low: u32,
    high: u32,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLocalTime(local: *mut Clock);
    fn SystemTimeToFileTime(clock: *const Clock, count: *mut Count) -> i32;
    fn FileTimeToSystemTime(count: *const Count, clock: *mut Clock) -> i32;
    fn SystemTimeToTzSpecificLocalTimeEx(
        zone: *const c_void,
        universal: *const Clock,
        local: *mut Clock,
    ) -> i32;
    fn TzSpecificLocalTimeToSystemTimeEx(
        zone: *const c_void,
        local: *const Clock,
        universal: *mut Clock,
    ) -> i32;
}

/// Today where the computer is, by its own clock: the day Windows' calendar marks as today.
/// Where the clock stands outside the years a [`CalendarDay`] is of, the nearest day of them.
fn today() -> CalendarDay {
    let mut local = Clock::default();
    // SAFETY: `local` is a `SYSTEMTIME` that lives through the call, and Windows writes within
    // it alone.
    unsafe { GetLocalTime(&raw mut local) };
    day_on(local).unwrap_or_else(|| {
        let (year, month, day) = if i32::from(local.year) < i32::from(YEARS.0) {
            (YEARS.0, 1, 1)
        } else {
            (YEARS.1, 12, 31)
        };
        CalendarDay { year, month, day }
    })
}

/// The day a clock's date is, where it is one a [`CalendarDay`] holds.
fn day_on(clock: Clock) -> Option<CalendarDay> {
    CalendarDay::new(clock.year, clock.month, clock.day).ok()
}

/// Noon of `day` where the computer is, as the instant Windows' picker takes. None where
/// Windows cannot place it.
fn noon(day: CalendarDay) -> Option<DateTime> {
    let local = Clock {
        year: u16::try_from(day.year).ok()?,
        month: day.month.into(),
        day: day.day.into(),
        hour: 12,
        ..Clock::default()
    };
    let mut universal = Clock::default();
    let mut count = Count::default();
    // SAFETY: each pointer is to a value of the layout Windows names for it that lives through
    // the call; Windows reads the first of each call and writes within the second alone. A null
    // zone asks for the computer's own.
    let placed = unsafe {
        TzSpecificLocalTimeToSystemTimeEx(null(), &raw const local, &raw mut universal) != 0
            && SystemTimeToFileTime(&raw const universal, &raw mut count) != 0
    };
    let whole = (u64::from(count.high) << 32) | u64::from(count.low);
    let universal_time = i64::try_from(whole).ok()?;
    placed.then_some(DateTime { universal_time })
}

/// The day `instant` falls on where the computer is. None where Windows cannot place it, or it
/// falls outside the years a [`CalendarDay`] is of.
fn day_of(instant: DateTime) -> Option<CalendarDay> {
    let whole = u64::try_from(instant.universal_time).ok()?;
    let count = Count {
        low: u32::try_from(whole & 0xFFFF_FFFF).ok()?,
        high: u32::try_from(whole >> 32).ok()?,
    };
    let mut universal = Clock::default();
    let mut local = Clock::default();
    // SAFETY: each pointer is to a value of the layout Windows names for it that lives through
    // the call; Windows reads the first of each call and writes within the second alone. A null
    // zone asks for the computer's own.
    let placed = unsafe {
        FileTimeToSystemTime(&raw const count, &raw mut universal) != 0
            && SystemTimeToTzSpecificLocalTimeEx(null(), &raw const universal, &raw mut local) != 0
    };
    placed.then(|| day_on(local)).flatten()
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for windows_reactor::CalendarDatePicker {}
}

/// Windows' date picker, made to hold a [`CalendarDay`].
///
/// The picker's own `date`, `min_date` and `max_date` take an instant, and its own
/// `on_date_changed` gives one. These three take and give a day, and are what a program that
/// holds a day calls in their place.
pub trait DayPicker: sealed::Sealed + Sized {
    /// Offers the days of `days` and holds the day they hold, or no day. A day, or an `Option`
    /// of one, is held among any days: `picker.days(day)`. Stated ends are [`Days::within`].
    ///
    /// An end left open stands a hundred years from the computer's today, as Windows' picker
    /// stands when given none: further where the day held lies beyond that, and a hundred years
    /// beyond a stated end that itself lies beyond it.
    #[must_use]
    fn days(self, days: impl Into<Days>) -> Self;

    /// States which day is today to the program. Windows marks today in the picker by the
    /// computer's own clock and can be told no other day, so its mark is shown only where the
    /// two are one day, and no day is marked as today where they differ or `today` is none.
    #[must_use]
    fn today(self, today: impl Into<Option<CalendarDay>>) -> Self;

    /// Tells the program the day a person picked, or none where she cleared it. Nothing is told
    /// of a day the program itself gave.
    #[must_use]
    fn on_day_changed(self, changed: impl IntoPayloadCallback<Option<CalendarDay>>) -> Self;
}

impl DayPicker for CalendarDatePicker {
    fn days(self, days: impl Into<Days>) -> Self {
        let days = days.into();
        let (first, last) = span(days, today());
        // Windows places every day a `CalendarDay` holds. Were it to refuse one, the picker
        // would keep Windows' own bounds and hold no day.
        let (Some(first), Some(last)) = (noon(first), noon(last)) else {
            return self.date(None);
        };
        self.min_date(first)
            .max_date(last)
            .date(days.held.and_then(noon))
    }

    fn today(self, today: impl Into<Option<CalendarDay>>) -> Self {
        self.is_today_highlighted(today.into() == Some(self::today()))
    }

    fn on_day_changed(self, changed: impl IntoPayloadCallback<Option<CalendarDay>>) -> Self {
        let changed = changed.into_payload_callback();
        self.on_date_changed(move |picked: Option<DateTime>| match picked {
            None => changed.call(None),
            // An instant Windows cannot place is no day she picked, and nothing is told of it.
            Some(instant) => {
                if let Some(day) = day_of(instant) {
                    changed.call(Some(day));
                }
            }
        })
    }
}

#[cfg(test)]
mod tests;
