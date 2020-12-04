use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};
#[macro_use]
extern crate lazy_static;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();

    let (part_one, part_two) = process(stdin.lock())?;

    println!("{}", part_one);
    println!("{}", part_two);

    assert_ne!(89, part_one);
    assert_ne!(191, part_one);
    assert_ne!(21, part_one);
    assert_eq!(192, part_one);
    assert_ne!(19, part_two);
    assert_eq!(101, part_two);

    Ok(())
}

fn process<R: BufRead>(input: R) -> Result<(usize, usize), Box<dyn std::error::Error + 'static>> {
    let required = [
        ("byr", validate_birth_year as fn(&str) -> bool),
        ("iyr", validate_issue_year),
        ("eyr", validate_exp_year),
        ("hgt", validate_height),
        ("hcl", validate_hair_color),
        ("ecl", validate_eye_color),
        ("pid", validate_passport_id),
    ]
    .iter()
    .map(|&s| s)
    .collect::<HashMap<&str, fn(&str) -> bool>>();

    assert_eq!(required.len(), 7);
    let mut part_one = 0;
    let mut part_two = 0;

    let mut splitter = EmptyLineSeparated::new(input);

    while let Some(record_buffer) = splitter.read_next()? {
        let (has_all, valid) = inspect_record(&record_buffer, &required);

        if has_all {
            part_one += 1;
        }

        if valid {
            part_two += 1;
        }
    }

    Ok((part_one, part_two))
}

struct EmptyLineSeparated<R: BufRead> {
    in_record: bool,
    buffer: String,
    inner: R,
    eof: bool,
}

impl<R: BufRead> EmptyLineSeparated<R> {
    fn new(input: R) -> Self {
        Self {
            in_record: true,
            buffer: String::new(),
            inner: input,
            eof: false,
        }
    }

    fn read_next(&mut self) -> Result<Option<&str>, std::io::Error> {
        if self.eof {
            // originally thought might still have some unprocessed
            // but the only way we get here is to have an read == 0
            // which in turn would not have put anything in the buffer.
            return Ok(None);
        }

        loop {
            let before = if !self.in_record {
                self.buffer.clear();
                0
            } else {
                self.buffer.len()
            };

            let read = self.inner.read_line(&mut self.buffer)?;
            let pre_trim = &self.buffer[before..];
            let buf = pre_trim.trim();

            if buf.is_empty() {
                assert!(self.in_record);
                self.in_record = false;
                self.eof = read == 0;

                // this seemed a bit tricky but for our input, this will hold because of this there
                // does not need to be a buffer.drain(..before) in the else branch
                assert_eq!(pre_trim, if self.eof { "" } else { "\n" });
                return Ok(Some(&self.buffer[..before]));
            } else {
                self.in_record = true;
            }
        }
    }
}

fn inspect_record(
    record_buffer: &str,
    required: &HashMap<&str, for<'s> fn(&'s str) -> bool>,
) -> (bool, bool) {
    let found = record_buffer.split_whitespace().map(|field| {
        let mut split = field.splitn(2, ':');
        let key = split.next().unwrap();
        let value = split.next().unwrap();
        (key, value)
    });

    let mut valid = true;
    let mut found_keys = HashSet::new();

    for (key, value) in found {
        if key == "cid" {
            // optional
            continue;
        }

        if !required[key](value) {
            valid = false;
        }

        assert!(found_keys.insert(key));
    }

    let has_all = found_keys.len() == required.len();

    return (has_all, has_all && valid);
}

fn validate_birth_year(s: &str) -> bool {
    if let Ok(y) = s.parse::<usize>() {
        (1920..=2002).contains(&y)
    } else {
        false
    }
}

fn validate_issue_year(s: &str) -> bool {
    if let Ok(y) = s.parse::<usize>() {
        (2010..=2020).contains(&y)
    } else {
        false
    }
}

#[test]
fn issue_year() {
    assert!(validate_issue_year("2010"));
    assert!(validate_issue_year("2015"));
    assert!(validate_issue_year("2020"));
    assert!(!validate_issue_year("2009"));
    assert!(!validate_issue_year("2021"));
    assert!(!validate_issue_year("2021abc"));
}

fn validate_exp_year(s: &str) -> bool {
    if let Ok(y) = s.parse::<usize>() {
        (2020..=2030).contains(&y)
    } else {
        false
    }
}

fn validate_height(s: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("^([0-9]+)(in|cm)$").unwrap();
    }
    for cap in RE.captures_iter(s) {
        let amount = cap[1].parse::<usize>();
        let unit = &cap[2];

        return match (unit, amount) {
            ("cm", Ok(ref h)) if (150..=193).contains(h) => true,
            ("in", Ok(ref h)) if (59..=76).contains(h) => true,
            _ => false,
        };
    }

    false
}

#[test]
fn height() {
    assert!(validate_height("59in"));
    assert!(validate_height("60in"));
    assert!(validate_height("70in"));
    assert!(validate_height("150cm"));
    assert!(validate_height("193cm"));
    assert!(!validate_height("190in"));
    assert!(!validate_height("190"));
}

fn validate_hair_color(s: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("^#[0-9a-f]{6}$").unwrap();
    }
    RE.is_match(s)
}

fn validate_eye_color(s: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("^(amb|blu|brn|gry|grn|hzl|oth)$").unwrap();
    }
    RE.is_match(s)
}

fn validate_passport_id(s: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("^[0-9]{9}$").unwrap();
    }
    RE.is_match(s)
}

#[test]
fn part_one_example() {
    let (count, _) = process(std::io::BufReader::new(std::io::Cursor::new(
        b"ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in",
    )))
    .unwrap();

    assert_eq!(2, count);
}

#[test]
fn part_two_examples() {
    let (_, count) = process(std::io::BufReader::new(std::io::Cursor::new(
        b"eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007

pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719",
    )))
    .unwrap();
    assert_eq!(4, count);
}
