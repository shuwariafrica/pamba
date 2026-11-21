use std::ops::Bound;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

/// A day the calendar has, by its three figures.
fn day(year: i16, month: u8, day: u8) -> CalendarDay {
    CalendarDay::new(year, month, day).unwrap()
}

/// The today every case of a span is reckoned from.
fn a_saturday() -> CalendarDay {
    day(2026, 11, 21)
}

/// Ten hours, in the hundred-nanosecond counts of Windows' time.
const TEN_HOURS: i64 = 10 * 60 * 60 * 10_000_000;

#[test]
fn a_day_the_calendar_lacks_is_no_day() {
    for (year, month, of_month) in [
        (2026, 2, 30),
        (2026, 2, 29),
        (1900, 2, 29),
        (2100, 2, 29),
        (2026, 4, 31),
        (2026, 0, 1),
        (2026, 13, 1),
        (2026, 1, 0),
        (2026, 1, 32),
    ] {
        assert_eq!(
            CalendarDay::new(year, month, of_month),
            None,
            "{year}-{month}-{of_month}"
        );
    }
    for (year, month, of_month) in [(2000, 2, 29), (2024, 2, 29), (2026, 2, 28), (2026, 12, 31)] {
        let made = CalendarDay::new(year, month, of_month).unwrap();
        assert_eq!(
            (made.year(), made.month(), made.day()),
            (year, month, of_month)
        );
    }
}

#[test]
fn a_day_is_of_the_years_windows_holds_in_every_zone() {
    assert_eq!(CalendarDay::new(1601, 12, 31), None);
    assert_eq!(CalendarDay::new(-1, 1, 1), None);
    assert_eq!(CalendarDay::new(10_000, 1, 1), None);
    assert!(CalendarDay::new(1602, 1, 1).is_some());
    assert!(CalendarDay::new(9999, 12, 31).is_some());
}

#[test]
fn days_stand_in_the_calendars_order() {
    let mut days = [
        day(2027, 1, 1),
        day(2026, 12, 1),
        day(2026, 11, 21),
        day(2026, 11, 3),
    ];
    days.sort_unstable();
    assert_eq!(
        days,
        [
            day(2026, 11, 3),
            day(2026, 11, 21),
            day(2026, 12, 1),
            day(2027, 1, 1)
        ]
    );
}

#[test]
fn the_day_after_and_the_day_before_cross_a_month_and_a_year_and_stop_at_the_ends() {
    assert_eq!(day(2026, 11, 21).next(), day(2026, 11, 22));
    assert_eq!(day(2026, 11, 30).next(), day(2026, 12, 1));
    assert_eq!(day(2024, 2, 28).next(), day(2024, 2, 29));
    assert_eq!(day(2026, 12, 31).next(), day(2027, 1, 1));
    assert_eq!(day(9999, 12, 31).next(), day(9999, 12, 31));
    assert_eq!(day(2026, 11, 21).previous(), day(2026, 11, 20));
    assert_eq!(day(2026, 12, 1).previous(), day(2026, 11, 30));
    assert_eq!(day(2024, 3, 1).previous(), day(2024, 2, 29));
    assert_eq!(day(2027, 1, 1).previous(), day(2026, 12, 31));
    assert_eq!(day(1602, 1, 1).previous(), day(1602, 1, 1));
}

#[test]
fn a_year_on_keeps_the_day_or_takes_the_last_of_a_shorter_month() {
    assert_eq!(day(2026, 11, 21).years_on(100), day(2126, 11, 21));
    assert_eq!(day(2026, 11, 21).years_on(-100), day(1926, 11, 21));
    assert_eq!(day(2024, 2, 29).years_on(1), day(2025, 2, 28));
    assert_eq!(day(2024, 2, 29).years_on(-100), day(1924, 2, 29));
    assert_eq!(day(2024, 2, 29).years_on(76), day(2100, 2, 28));
    assert_eq!(day(1650, 6, 15).years_on(-100), day(1602, 6, 15));
    assert_eq!(day(9950, 6, 15).years_on(100), day(9999, 6, 15));
}

#[test]
fn an_end_left_open_stands_a_hundred_years_from_today() {
    assert_eq!(
        span(None, &(..), a_saturday()),
        (day(1926, 11, 21), day(2126, 11, 21))
    );
    assert_eq!(
        span(Some(day(2026, 12, 4)), &(..), a_saturday()),
        (day(1926, 11, 21), day(2126, 11, 21))
    );
}

#[test]
fn an_end_left_open_reaches_as_far_as_the_day_held() {
    assert_eq!(
        span(Some(day(1915, 3, 2)), &(..), a_saturday()),
        (day(1915, 3, 2), day(2126, 11, 21))
    );
    assert_eq!(
        span(Some(day(2200, 1, 1)), &(..), a_saturday()),
        (day(1926, 11, 21), day(2200, 1, 1))
    );
}

#[test]
fn an_end_the_program_states_is_that_end_whatever_day_is_held() {
    let (first, last) = (day(2026, 11, 1), day(2026, 11, 30));
    assert_eq!(
        span(Some(day(1915, 3, 2)), &(first..=last), a_saturday()),
        (first, last)
    );
    assert_eq!(
        span(None, &(first..last), a_saturday()),
        (first, day(2026, 11, 29))
    );
    assert_eq!(
        span(
            None,
            &(Bound::Excluded(first), Bound::Unbounded),
            a_saturday()
        ),
        (day(2026, 11, 2), day(2126, 11, 21))
    );
    assert_eq!(
        span(None, &(..=last), a_saturday()),
        (day(1926, 11, 21), last)
    );
}

#[test]
fn a_span_that_would_hold_no_day_is_its_first_day_alone() {
    let (later, earlier) = (day(2026, 11, 30), day(2026, 11, 1));
    assert_eq!(span(None, &(later..=earlier), a_saturday()), (later, later));
    assert_eq!(span(None, &(later..later), a_saturday()), (later, later));
}

/// Days of every kind the crossing must hold for: an ordinary one, a leap day, the first and the
/// last a `CalendarDay` is of, and the days on which clocks change in the north and in the south.
fn crossed() -> [CalendarDay; 10] {
    [
        day(2026, 11, 21),
        day(2024, 2, 29),
        day(1602, 1, 1),
        day(9999, 12, 31),
        day(1970, 1, 1),
        day(2026, 3, 8),
        day(2026, 3, 29),
        day(2026, 10, 25),
        day(2026, 11, 1),
        day(2026, 4, 5),
    ]
}

#[test]
fn noon_of_a_day_is_read_back_as_that_day_where_the_computer_is() {
    for held in crossed() {
        assert_eq!(noon(held).and_then(day_of), Some(held), "{held:?}");
    }
}

#[test]
fn an_instant_ten_hours_either_side_of_noon_is_still_that_day() {
    for held in crossed() {
        let at_noon = noon(held).unwrap().universal_time;
        for universal_time in [at_noon - TEN_HOURS, at_noon + TEN_HOURS] {
            assert_eq!(
                day_of(DateTime { universal_time }),
                Some(held),
                "{held:?} at {universal_time}"
            );
        }
    }
}

#[test]
fn noon_of_the_day_after_is_about_a_day_later() {
    let hour = TEN_HOURS / 10;
    for held in crossed() {
        let (today, tomorrow) = (noon(held).unwrap(), noon(held.next()).unwrap());
        let apart = tomorrow.universal_time - today.universal_time;
        if held.next() == held {
            assert_eq!(apart, 0, "{held:?}");
        } else {
            assert!(
                (23 * hour..=25 * hour).contains(&apart),
                "{held:?}: {apart}"
            );
        }
    }
}

#[test]
fn an_instant_before_windows_time_begins_is_no_day() {
    // As far before the count begins as a day of this century is after it: read without its sign it would be a day.
    let after = noon(a_saturday()).unwrap().universal_time;
    assert_eq!(
        day_of(DateTime {
            universal_time: -after
        }),
        None
    );
    assert_eq!(day_of(DateTime { universal_time: -1 }), None);
}

#[test]
fn today_is_the_day_the_computers_clock_stands_in() {
    // The clock read another way: the count since 1970 that the standard library gives, as Windows' count.
    let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let universal_time =
        DateTime::UNIX_EPOCH.universal_time + i64::try_from(since.as_nanos() / 100).unwrap();
    let before = today();
    let by_the_clock = day_of(DateTime { universal_time });
    let after = today();
    // Midnight may pass between the readings, and then the clock's day is the later one.
    assert!(
        by_the_clock == Some(before) || by_the_clock == Some(after),
        "{by_the_clock:?} is neither {before:?} nor {after:?}"
    );
}
