use std::cmp;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Debug)]
enum State {
    Init,
    Shift { guard_id: usize },
    Asleep { guard_id: usize, from: NaiveTime },
}

fn main() -> Result<(), EventParsingFailure> {
    let events: Result<Vec<Event>, EventParsingFailure> =
        aoc2018::try_fold_stdin(Vec::new(), |events, line| {
            events.push(Event::try_from(line)?);
            Ok(())
        });

    let mut events = events?;

    events.sort();

    let mut slept = HashMap::new();
    let mut state = State::Init;

    for event in events {
        state = match (state, event) {
            (
                State::Init,
                Event {
                    payload: Payload::ShiftStart { guard_id },
                    ..
                },
            ) => State::Shift { guard_id },
            (
                State::Shift { guard_id },
                Event {
                    ts: Timestamp { time: from, .. },
                    payload: Payload::FellAsleep,
                },
            ) => State::Asleep { guard_id, from },
            (
                State::Asleep { guard_id, from },
                Event {
                    ts: Timestamp { time: to, .. },
                    payload: Payload::WokeUp,
                },
            ) => {
                let sleeping_hours = slept.entry(guard_id).or_insert_with(|| HashMap::new());

                for minute in minutes_between(from, to) {
                    let times = sleeping_hours.entry(minute).or_insert(0u32);
                    *times += 1;
                }

                State::Shift { guard_id }
            }
            (
                State::Shift { .. },
                Event {
                    payload: Payload::ShiftStart { guard_id },
                    ..
                },
            ) => State::Shift { guard_id },
            (state, event) => unreachable!("unsupported: {:?} when {:?}", event, state),
        };
    }

    let (guard_id, minutes) = slept
        .iter()
        .max_by_key(|&(_, minutes)| minutes.values().sum::<u32>())
        .unwrap();

    let part1 = guard_id
        * minutes
            .iter()
            .max_by_key(|&(_, slept)| *slept)
            .map(|(minute, _)| minute.1)
            .unwrap() as usize;

    println!("part1: {}", part1);

    let (guard_id, _, minute_most_frequently) = slept
        .iter()
        .map(|(guard_id, minutes)| {
            minutes
                .iter()
                .max_by_key(|(_, times)| *times)
                .map(move |(minute, times)| (*guard_id, *times, *minute))
        })
        .map(Option::unwrap)
        .max_by_key(|(_, times, _)| *times)
        .unwrap();

    let part2 = guard_id * minute_most_frequently.1 as usize;

    println!("part2: {}", part2);

    assert_eq!(part1, 95199);
    assert_eq!(part2, 7887);

    Ok(())
}

fn minutes_between(start: NaiveTime, end: NaiveTime) -> impl Iterator<Item = NaiveTime> {
    (0..)
        .scan(start, |time, _| {
            let prev = *time;
            time.1 += 1;

            if time.1 >= 60 {
                time.0 += 1;
                time.1 = 0;

                if time.0 >= 24 {
                    time.0 = 0;
                }
            }

            Some(prev)
        })
        .take_while(move |&time| time < end)
}

#[derive(Debug, PartialEq, Eq)]
struct Event {
    ts: Timestamp,
    payload: Payload,
}

impl cmp::PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<cmp::Ordering> {
        Some(self.ts.cmp(&other.ts))
    }
}

impl cmp::Ord for Event {
    fn cmp(&self, other: &Event) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

type NaiveDate = (u16, u8, u8);
type NaiveTime = (u8, u8);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Timestamp {
    date: NaiveDate,
    time: NaiveTime,
}

#[derive(Debug, PartialEq, Eq)]
enum Payload {
    ShiftStart { guard_id: usize },
    FellAsleep,
    WokeUp,
}

#[derive(Debug, PartialEq)]
enum PayloadParsingFailure {
    EmptyInput,
    UnknownInput,
    MissingGuardId,
    InvalidGuardId,
}

impl TryFrom<&str> for Payload {
    type Error = PayloadParsingFailure;

    fn try_from(s: &str) -> Result<Payload, Self::Error> {
        let mut iter = s.split_whitespace();

        let first = iter.next();

        Ok(match first {
            Some("falls") => Payload::FellAsleep,
            Some("wakes") => Payload::WokeUp,
            Some("Guard") => {
                let raw_id = iter.next().ok_or(PayloadParsingFailure::MissingGuardId)?;

                let mut chars = raw_id.chars();

                let hash = chars.next().ok_or(PayloadParsingFailure::InvalidGuardId)?;

                if hash != '#' {
                    return Err(PayloadParsingFailure::InvalidGuardId);
                }

                let guard_id = chars
                    .as_str()
                    .parse::<usize>()
                    .map_err(|_| PayloadParsingFailure::InvalidGuardId)?;

                Payload::ShiftStart { guard_id }
            }
            Some(_) => return Err(PayloadParsingFailure::UnknownInput),
            None => return Err(PayloadParsingFailure::EmptyInput),
        })
    }
}

#[derive(Debug, PartialEq)]
enum EventParsingFailure {
    EmptyInput,
    InvalidStartOfLine,
    Timestamp(TimestampParsingFailure),
    Payload(Timestamp, PayloadParsingFailure),
}

impl From<TimestampParsingFailure> for EventParsingFailure {
    fn from(ts: TimestampParsingFailure) -> EventParsingFailure {
        EventParsingFailure::Timestamp(ts)
    }
}

impl From<(Timestamp, PayloadParsingFailure)> for EventParsingFailure {
    fn from((ctx, p): (Timestamp, PayloadParsingFailure)) -> EventParsingFailure {
        EventParsingFailure::Payload(ctx, p)
    }
}

#[derive(Debug, PartialEq)]
enum TimestampParsingFailure {
    Missing(TimestampField),
    Invalid(TimestampField),
}

#[derive(Debug, PartialEq)]
enum TimestampField {
    Year,
    Month,
    Day,
    Hours,
    Minutes,
}

impl TimestampField {
    fn missing(self) -> TimestampParsingFailure {
        TimestampParsingFailure::Missing(self)
    }
    fn invalid(self) -> TimestampParsingFailure {
        TimestampParsingFailure::Invalid(self)
    }
}

impl TryFrom<&str> for Event {
    type Error = EventParsingFailure;

    fn try_from(s: &str) -> Result<Event, Self::Error> {
        if s.is_empty() {
            return Err(EventParsingFailure::EmptyInput);
        }

        let mut chars = s.char_indices();

        let bracket = chars
            .next()
            .ok_or(EventParsingFailure::InvalidStartOfLine)?;
        if bracket.1 != '[' {
            return Err(EventParsingFailure::InvalidStartOfLine);
        }

        let rest = chars.as_str();

        let (year, rest) = timestamp_part(rest, '-', TimestampField::Year)?;
        let (month, rest) = timestamp_part(rest, '-', TimestampField::Month)?;
        let (day, rest) = timestamp_part(rest, ' ', TimestampField::Day)?;
        let (hours, rest) = timestamp_part(rest, ':', TimestampField::Hours)?;
        let (minutes, rest) = timestamp_part(rest, ']', TimestampField::Minutes)?;

        let ts = Timestamp {
            date: (year, month, day),
            time: (hours, minutes),
        };

        let payload = match Payload::try_from(rest.trim()) {
            Ok(payload) => payload,
            Err(e) => return Err((ts, e).into()),
        };

        Ok(Event { ts, payload })
    }
}

fn split2<'a>(s: &'a str, split: char) -> Option<(&'a str, &'a str)> {
    let mut parts = s.splitn(2, split);

    let first = parts.next().expect("first split element is always present");

    parts.next().map(move |rest| (first, rest))
}

fn timestamp_part<'a, T: FromStr>(
    s: &'a str,
    until: char,
    field: TimestampField,
) -> Result<(T, &'a str), TimestampParsingFailure> {
    let (raw, rest) = match split2(s, until) {
        Some((raw, rest)) => (raw, rest),
        None => return Err(field.missing()),
    };

    let val = raw.parse::<T>().map_err(move |_| field.invalid())?;

    Ok((val, rest))
}

#[test]
fn test_parsing() {
    let example = "\
[1518-06-27 00:21] falls asleep
[1518-11-10 23:52] Guard #881 begins shift
[1518-11-08 00:51] wakes up
";

    let expected = [
        Ok(Event {
            ts: Timestamp {
                date: (1518, 6, 27),
                time: (0, 21),
            },
            payload: Payload::FellAsleep,
        }),
        Ok(Event {
            ts: Timestamp {
                date: (1518, 11, 10),
                time: (23, 52),
            },
            payload: Payload::ShiftStart { guard_id: 881 },
        }),
        Ok(Event {
            ts: Timestamp {
                date: (1518, 11, 8),
                time: (0, 51),
            },
            payload: Payload::WokeUp,
        }),
    ];

    let parsed = example.lines().map(Event::try_from).collect::<Vec<_>>();

    assert_eq!(&expected[..], parsed.as_slice());
}
