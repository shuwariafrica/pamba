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

use std::ffi::c_void;
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
    /// The day of that year, month (1 to 12) and day of the month. None where the calendar has
    /// no such day, or the year is outside 1602 to 9999.
    #[must_use]
    pub fn new(year: i16, month: u8, day: u8) -> Option<Self> {
        let real = (YEARS.0..=YEARS.1).contains(&year)
            && (1..=12).contains(&month)
            && (1..=days_in(year, month)).contains(&day);
        real.then_some(Self { year, month, day })
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
        if self.day < days_in(self.year, self.month) {
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
                day: days_in(self.year, month),
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
            day: self.day.min(days_in(year, self.month)),
        }
    }
}

const fn leap(year: i16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// How many days that month of that year has.
const fn days_in(year: i16, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => 31,
    }
}

/// The first and the last day a picker offers. Each is the program's where `within` states it.
/// Where it does not, it is [`REACH`] years from `today`, as Windows' picker stands when given
/// no bound, and further where `held` lies beyond that, so that the day held is always one the
/// picker can show. A span that would hold no day is its first day alone.
fn span(
    held: Option<CalendarDay>,
    within: &impl RangeBounds<CalendarDay>,
    today: CalendarDay,
) -> (CalendarDay, CalendarDay) {
    let first = match within.start_bound() {
        Bound::Included(day) => *day,
        Bound::Excluded(day) => day.next(),
        Bound::Unbounded => {
            let reach = today.years_on(-REACH);
            held.map_or(reach, |held| held.min(reach))
        }
    };
    let last = match within.end_bound() {
        Bound::Included(day) => *day,
        Bound::Excluded(day) => day.previous(),
        Bound::Unbounded => {
            let reach = today.years_on(REACH);
            held.map_or(reach, |held| held.max(reach))
        }
    };
    (first, last.max(first))
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
    CalendarDay::new(
        i16::try_from(clock.year).ok()?,
        u8::try_from(clock.month).ok()?,
        u8::try_from(clock.day).ok()?,
    )
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
    /// Holds `held`, or no day, among the days `within`: `..` for any day, `first..=last` for
    /// those two and the days between, or a range open at one end.
    ///
    /// An end `within` leaves open stands a hundred years from the computer's today, as
    /// Windows' picker stands when given none, and further where `held` lies beyond that. A day
    /// held outside an end `within` states is shown as that end.
    #[must_use]
    fn days(
        self,
        held: impl Into<Option<CalendarDay>>,
        within: impl RangeBounds<CalendarDay>,
    ) -> Self;

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
    fn days(
        self,
        held: impl Into<Option<CalendarDay>>,
        within: impl RangeBounds<CalendarDay>,
    ) -> Self {
        let held = held.into();
        let (first, last) = span(held, &within, today());
        // Windows places every day a `CalendarDay` holds. Were it to refuse one, the picker
        // would keep Windows' own bounds and hold no day.
        let (Some(first), Some(last)) = (noon(first), noon(last)) else {
            return self.date(None);
        };
        self.min_date(first)
            .max_date(last)
            .date(held.and_then(noon))
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
