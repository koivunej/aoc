use std::ops::RangeInclusive;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let re = regex::Regex::new(r"^(\d+)-(\d+) (\S): (\S+)$").unwrap();
    let mut buf = String::new();

    let mut part_one = 0;
    let mut part_two = 0;
    loop {
        buf.clear();
        let read = stdin.read_line(&mut buf)?;
        if read == 0 {
            break;
        }

        let buf = buf.trim();

        if buf.is_empty() {
            continue;
        }

        for cap in re.captures_iter(buf) {
            let start = cap[1].parse::<u8>().expect("matched with re already");
            let end = cap[2].parse::<u8>().expect("matched with re already");
            let ch = cap[3].chars().next().unwrap();

            let p = Policy(start..=end, ch);

            let pw = &cap[4];

            if p.verify_contains(pw) {
                part_one += 1;
            }

            if p.verify_at_indices(pw) {
                part_two += 1;
            }
        }
    }

    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(519, part_one);
    assert_eq!(708, part_two);

    Ok(())
}

struct Policy(RangeInclusive<u8>, char);

impl Policy {
    fn verify_contains(&self, s: &str) -> bool {
        let found = s.chars().filter(|&x| x == self.1).count();

        assert!(found < 256);

        self.0.contains(&(found as u8))
    }

    fn verify_at_indices(&self, s: &str) -> bool {
        assert!(s.len() < 255); // because of +1
        let found = s
            .char_indices()
            .filter(|&(_, ch)| ch == self.1)
            // tripped on this quite confusingly: was using RangeInclusive::contains
            .filter(|&(index, _)| {
                let index = (index as u8) + 1;
                &index == self.0.start() || &index == self.0.end()
            })
            .count();

        // exactly one match at the set of indices
        found == 1
    }
}

#[test]
fn part2_examples() {
    assert!(Policy(1..=3, 'a').verify_at_indices("abcde"));
    assert!(!Policy(1..=3, 'b').verify_at_indices("cdefg"));
    assert!(!Policy(2..=9, 'c').verify_at_indices("ccccccccc"));
    assert!(Policy(2..=9, 'c').verify_at_indices("bbbbbbbbc"));
    assert!(!Policy(2..=9, 'c').verify_at_indices("bcbbbbbbc"));
    assert!(Policy(2..=9, 'c').verify_at_indices("bcbbbbbbb"));
}
