use std::collections::HashMap;
#[macro_use]
extern crate lazy_static;
use regex::Regex;

fn validate_birth_year(s: &str) -> bool {
    if let Ok(y) = s.parse::<usize>() {
        1920 <= y && y <= 2002
    } else {
        false
    }
}

fn validate_issue_year(s: &str) -> bool {
    if let Ok(y) = s.parse::<usize>() {
        2010 <= y && y <= 2020
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
        2020 <= y && y <= 2030
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
            ("cm", Ok(h)) if 150 <= h && h <= 193 => true,
            ("in", Ok(h)) if 59 <= h && h <= 76 => true,
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

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();

    let required = [
        ("byr", validate_birth_year as for<'s> fn(&'s str) -> bool),
        ("iyr", validate_issue_year),
        ("eyr", validate_exp_year),
        ("hgt", validate_height),
        ("hcl", validate_hair_color),
        ("ecl", validate_eye_color),
        ("pid", validate_passport_id),
    ]
    .iter()
    .map(|&s| s)
    .collect::<HashMap<&str, for<'s> fn(&'s str) -> bool>>();

    assert_eq!(required.len(), 7);

    // let mut found = HashSet::with_capacity(expected.len());
    let mut in_record = false;
    let mut record_buffer = String::new();

    let mut part_one = 0;
    let mut part_two = 0;

    loop {
        buf.clear();

        let read = stdin.read_line(&mut buf)?;
        let buf = buf.trim();

        if buf.is_empty() {
            assert!(in_record, "empty line outside of record?");

            println!("{:?}", record_buffer);

            let (has_all, valid) = inspect_record(&record_buffer, &required);

            if has_all {
                part_one += 1;
            }

            if valid {
                part_two += 1;
            }

            in_record = false;
            record_buffer.clear();

            if read == 0 {
                // I was originally of course just breaking with this, leaving the last element
                // unprocessed...
                break;
            } else {
                continue;
            }
        }

        in_record = true;
        if !record_buffer.is_empty() {
            record_buffer.push(' ');
        }
        record_buffer.push_str(buf);
    }

    assert!(record_buffer.is_empty());

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

fn inspect_record(
    record_buffer: &str,
    required: &HashMap<&str, for<'s> fn(&'s str) -> bool>,
) -> (bool, bool) {
    let found = record_buffer
        .split(' ')
        .map(|field| {
            let mut split = field.splitn(2, ':');
            let key = split.next().unwrap();
            let value = split.next().unwrap();
            (key, value)
        })
        .collect::<HashMap<&str, &str>>();
    /*.for_each(|key| {
        if !found.insert(key) {
            println!("duplicate field '{}'", key);
        }
    });*/

    if found.len() >= required.len() {
        let mut valid = true;
        let mut found_required = 0;

        for (key, value) in found {
            if key == "cid" {
                // optional
                continue;
            }

            if !required[key](value) {
                valid = false;
                println!("invalid field {}: {}", key, value);
            }
            found_required += 1;
        }

        let has_all = found_required == required.len();

        return (has_all, has_all && valid);
    } else {
        println!(
            "bad after found {} needed {}: {:?}",
            found.len(),
            required.len(),
            found
        );
    }

    (false, false)
}
