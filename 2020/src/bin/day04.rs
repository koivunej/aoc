use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();

    let expected = ["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid", "cid"]
        .iter()
        .map(|&s| s)
        .collect::<HashSet<&str>>();

    let part_one_expected = {
        let mut expected = expected.clone();
        expected.remove("cid");
        expected
    };

    assert_eq!(expected.len(), 8);
    assert_eq!(part_one_expected.len(), 7);

    // let mut found = HashSet::with_capacity(expected.len());
    let mut in_record = false;
    let mut record_buffer = String::new();

    let mut part_one = 0;

    loop {
        buf.clear();

        let read = stdin.read_line(&mut buf)?;

        let buf = buf.trim();

        if buf.is_empty() {
            assert!(in_record, "empty line outside of record?");

            println!("{:?}", record_buffer);

            if inspect_record(&record_buffer, &part_one_expected) {
                part_one += 1;
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

    assert_ne!(89, part_one);
    assert_ne!(191, part_one);
    assert_ne!(21, part_one);

    Ok(())
}

fn inspect_record(record_buffer: &str, part_one_expected: &HashSet<&str>) -> bool {
    let found = record_buffer
        .split(' ')
        .flat_map(|field| field.split(':').take(1))
        .collect::<HashSet<_>>();
    /*.for_each(|key| {
        if !found.insert(key) {
            println!("duplicate field '{}'", key);
        }
    });*/

    if found.len() >= part_one_expected.len() {
        let diff = part_one_expected.difference(&found);

        let mut bad = false;

        for &elem in diff {
            if elem == "cid" {
                continue;
            }

            bad = true;
            println!("bad after missing '{}': {:?}", elem, found);
            break;
        }

        return !bad;
    } else {
        println!(
            "bad after found {} needed {}: {:?}",
            found.len(),
            part_one_expected.len(),
            found
        );
    }

    false
}

#[test]
fn example_records() {
    let expected = ["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"]
        .iter()
        .map(|&s| s)
        .collect::<HashSet<&str>>();
    assert!(inspect_record(
        "ecl:gry pid:860033327 eyr:2020 hcl:#fffffd byr:1937 iyr:2017 cid:147 hgt:183cm",
        &expected
    ));
    assert!(!inspect_record(
        "iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884 hcl:#cfa07d byr:1929",
        &expected
    ));
    assert!(inspect_record(
        "hcl:#ae17e1 iyr:2013 eyr:2024 ecl:brn pid:760753108 byr:1931 hgt:179cm",
        &expected
    ));
    assert!(!inspect_record(
        "hcl:#cfa07d eyr:2025 pid:166559648 iyr:2011 ecl:brn hgt:59in",
        &expected
    ));
}
