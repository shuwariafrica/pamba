use std::ops::{Bound, RangeBounds};
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

/// The first and the last day a picker is given for a day held, or none, among the days
/// `within`, on that Saturday.
fn offered(
    held: Option<CalendarDay>,
    within: impl RangeBounds<CalendarDay>,
) -> (CalendarDay, CalendarDay) {
    span(Days::within(held, within).unwrap(), a_saturday())
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
            Err(NoDay::NoSuchDay),
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
    assert_eq!(CalendarDay::new(1601, 12, 31), Err(NoDay::YearOutside));
    assert_eq!(CalendarDay::new(-1, 1, 1), Err(NoDay::YearOutside));
    assert_eq!(CalendarDay::new(10_000, 1, 1), Err(NoDay::YearOutside));
    assert!(CalendarDay::new(1602, 1, 1).is_ok());
    assert!(CalendarDay::new(9999, 12, 31).is_ok());
}

#[test]
fn the_calendar_is_judged_before_the_year() {
    // No year has a thirtieth of February, so the year is never the reason.
    assert_eq!(CalendarDay::new(10_000, 2, 30), Err(NoDay::NoSuchDay));
    assert_eq!(CalendarDay::new(1500, 13, 1), Err(NoDay::NoSuchDay));
    // A twenty-ninth of February is judged by its own year's rule, however far that year lies.
    assert_eq!(CalendarDay::new(1600, 2, 29), Err(NoDay::YearOutside));
    assert_eq!(CalendarDay::new(1500, 2, 29), Err(NoDay::NoSuchDay));
    assert_eq!(CalendarDay::new(40_000, 2, 29), Err(NoDay::YearOutside));
    assert_eq!(CalendarDay::new(40_001, 2, 29), Err(NoDay::NoSuchDay));
    assert_eq!(CalendarDay::new(-4, 2, 29), Err(NoDay::YearOutside));
    assert_eq!(CalendarDay::new(i64::MAX, 2, 29), Err(NoDay::NoSuchDay));
}

#[test]
fn a_day_is_made_of_figures_as_a_programs_own_date_gives_them() {
    let held = day(2026, 11, 21);
    assert_eq!(CalendarDay::new(2026_i16, 11_i8, 21_i8), Ok(held));
    assert_eq!(CalendarDay::new(2026_i32, 11_u32, 21_u32), Ok(held));
    assert_eq!(CalendarDay::new(2026_u16, 11_u16, 21_u16), Ok(held));
    assert_eq!(CalendarDay::new(2026_i64, 11_u64, 21_usize), Ok(held));
}

#[test]
fn a_figure_no_month_or_day_can_be_is_no_day() {
    assert_eq!(CalendarDay::new(2026, -1_i8, 1), Err(NoDay::NoSuchDay));
    assert_eq!(CalendarDay::new(2026, 1, -1_i8), Err(NoDay::NoSuchDay));
    // 257 and 268 would be 1 and 12 were a figure cut to the eight bits a month is held in.
    assert_eq!(CalendarDay::new(2026, 257_u32, 1), Err(NoDay::NoSuchDay));
    assert_eq!(CalendarDay::new(2026, 1, 257_u32), Err(NoDay::NoSuchDay));
    assert_eq!(CalendarDay::new(2026, 268_i64, 31), Err(NoDay::NoSuchDay));
}

#[test]
fn each_reason_says_itself() {
    assert_eq!(NoDay::NoSuchDay.to_string(), "the calendar has no such day");
    assert_eq!(
        NoDay::YearOutside.to_string(),
        "the day's year is outside 1602 to 9999"
    );
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
        span(Days::any(None), a_saturday()),
        (day(1926, 11, 21), day(2126, 11, 21))
    );
    assert_eq!(
        span(day(2026, 12, 4).into(), a_saturday()),
        (day(1926, 11, 21), day(2126, 11, 21))
    );
}

#[test]
fn an_end_left_open_reaches_as_far_as_the_day_held() {
    assert_eq!(
        span(day(1915, 3, 2).into(), a_saturday()),
        (day(1915, 3, 2), day(2126, 11, 21))
    );
    assert_eq!(
        span(day(2200, 1, 1).into(), a_saturday()),
        (day(1926, 11, 21), day(2200, 1, 1))
    );
    assert_eq!(
        offered(Some(day(2200, 1, 1)), day(2026, 11, 1)..),
        (day(2026, 11, 1), day(2200, 1, 1))
    );
}

#[test]
fn an_end_the_program_states_is_that_end_whatever_day_was_to_be_held() {
    let (first, last) = (day(2026, 11, 1), day(2026, 11, 30));
    let refused = Days::within(day(1915, 3, 2), first..=last).unwrap_err();
    assert_eq!(span(refused.offered(), a_saturday()), (first, last));
    assert_eq!(offered(None, first..last), (first, day(2026, 11, 29)));
    assert_eq!(
        offered(None, (Bound::Excluded(first), Bound::Unbounded)),
        (day(2026, 11, 2), day(2126, 11, 21))
    );
    assert_eq!(offered(None, ..=last), (day(1926, 11, 21), last));
}

#[test]
fn days_that_would_be_none_are_their_first_day_alone_and_that_day_is_offered() {
    let (later, earlier) = (day(2026, 11, 30), day(2026, 11, 1));
    assert_eq!(offered(None, later..=earlier), (later, later));
    assert_eq!(offered(None, later..later), (later, later));
    assert_eq!(offered(Some(later), later..later), (later, later));
}

#[test]
fn a_day_before_or_after_a_stated_end_is_refused_with_the_day_and_the_same_days_holding_none() {
    let (first, last) = (day(2026, 11, 1), day(2026, 11, 30));
    let (before, after) = (day(2026, 10, 31), day(2026, 12, 1));
    let none = Days::within(None, first..=last).unwrap();
    for outside in [before, after, day(1915, 3, 2)] {
        let refused = Days::within(outside, first..=last).unwrap_err();
        assert_eq!((refused.held(), refused.offered()), (outside, none));
        assert_eq!(refused.offered().held(), None);
    }
    // An open end refuses no day on its own side.
    assert_eq!(Days::within(before, first..).unwrap_err().held(), before);
    assert_eq!(
        Days::within(after, first..).map(Days::held),
        Ok(Some(after))
    );
    assert_eq!(Days::within(after, ..=last).unwrap_err().held(), after);
    assert_eq!(
        Days::within(before, ..=last).map(Days::held),
        Ok(Some(before))
    );
}

#[test]
fn an_end_a_range_leaves_out_is_not_offered() {
    let (first, last) = (day(2026, 11, 1), day(2026, 11, 30));
    assert_eq!(Days::within(last, first..last).unwrap_err().held(), last);
    let after_first = (Bound::Excluded(first), Bound::Unbounded);
    assert_eq!(Days::within(first, after_first).unwrap_err().held(), first);
    assert_eq!(
        Days::within(day(2026, 11, 29), first..last).map(Days::held),
        Ok(Some(day(2026, 11, 29)))
    );
}

#[test]
fn a_day_at_either_stated_end_a_day_between_and_no_day_are_offered() {
    let (first, last) = (day(2026, 11, 1), day(2026, 11, 30));
    for held in [Some(first), Some(last), Some(day(2026, 11, 15)), None] {
        assert_eq!(Days::within(held, first..=last).map(Days::held), Ok(held));
    }
}

#[test]
fn a_day_alone_or_none_is_held_among_any_days() {
    let held = day(1915, 3, 2);
    assert_eq!(Days::from(held), Days::any(held));
    assert_eq!(Days::from(Some(held)), Days::any(held));
    assert_eq!(Days::from(None), Days::any(None));
    assert_eq!(Days::within(held, ..), Ok(Days::any(held)));
    assert_eq!(Days::any(held).held(), Some(held));
    assert_ne!(Days::any(held), Days::any(None));
}

#[test]
fn an_open_end_stands_a_hundred_years_beyond_a_stated_end_that_todays_reach_does_not_hold() {
    // A hundred years before that Saturday is 1926, which lies after a last day of 1800.
    assert_eq!(
        offered(None, ..=day(1800, 1, 1)),
        (day(1700, 1, 1), day(1800, 1, 1))
    );
    assert_eq!(
        offered(None, day(2300, 6, 15)..),
        (day(2300, 6, 15), day(2400, 6, 15))
    );
    // A stated end that today's reach holds leaves the open end where Windows stands.
    assert_eq!(
        offered(None, ..=day(2008, 11, 21)),
        (day(1926, 11, 21), day(2008, 11, 21))
    );
    assert_eq!(
        offered(Some(day(1650, 1, 1)), ..=day(1800, 1, 1)),
        (day(1650, 1, 1), day(1800, 1, 1))
    );
}

/// Over every pairing of eleven days and no day as the day held, the first stated and the last
/// stated: days that are made hold a day the picker is given, between ends that stand in order
/// and are the program's where it states them; days that are refused name a day outside them.
#[test]
fn a_picker_is_given_ends_in_order_that_hold_its_day_and_keep_every_stated_end() {
    let days = [
        day(1602, 1, 1),
        day(1650, 1, 1),
        day(1800, 1, 1),
        day(1926, 11, 20),
        day(1926, 11, 21),
        day(2026, 11, 21),
        day(2126, 11, 21),
        day(2126, 11, 22),
        day(2300, 6, 15),
        day(9999, 12, 30),
        day(9999, 12, 31),
    ];
    let or_none = || std::iter::once(None).chain(days.into_iter().map(Some));
    let mut walked = 0;
    for held in or_none() {
        for first in or_none() {
            for last in or_none() {
                let bound =
                    |end: Option<CalendarDay>| end.map_or(Bound::Unbounded, Bound::Included);
                walked += 1;
                // What the program states, a last before its first being that first alone.
                let stated_last = last.map(|last| first.map_or(last, |first| last.max(first)));
                match Days::within(held, (bound(first), bound(last))) {
                    Ok(made) => {
                        let (given_first, given_last) = span(made, a_saturday());
                        assert!(given_first <= given_last, "{held:?} {first:?} {last:?}");
                        assert_eq!(made.held(), held);
                        if let Some(held) = held {
                            assert!((given_first..=given_last).contains(&held));
                        }
                        assert_eq!(first.unwrap_or(given_first), given_first);
                        assert_eq!(stated_last.unwrap_or(given_last), given_last);
                    }
                    Err(refused) => {
                        let outside = refused.held();
                        assert_eq!(Some(outside), held);
                        assert!(
                            first.is_some_and(|first| outside < first)
                                || stated_last.is_some_and(|last| outside > last),
                            "{held:?} {first:?} {last:?}"
                        );
                        assert_eq!(refused.offered().held(), None);
                    }
                }
            }
        }
    }
    assert_eq!(walked, 12 * 12 * 12);
}

#[test]
fn the_refusal_says_the_day_that_is_not_offered() {
    let refused = Days::within(day(2026, 10, 15), day(2026, 11, 1)..).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "2026-10-15 is not one of the days offered"
    );
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
