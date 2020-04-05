fn main() {
    let numbers: Vec<i64> = parse_from_stdin().unwrap();

    let part1 = numbers.iter().sum::<i64>();
    println!("part1: {}", part1);

    let part2 = part2(&numbers).expect("there should be one repeating in an infinite cycle");
    println!("part2: {}", part2);

    assert_eq!(part1, 406);
}

fn part2(freqs: &[i64]) -> Option<i64> {
    use std::collections::HashSet;

    let calibrated = freqs.iter().cycle().scan(0, |acc, next| {
        *acc += next;
        Some(*acc)
    });

    let mut seen = HashSet::new();

    for freq in calibrated {
        if !seen.insert(freq) {
            return Some(freq);
        }
    }

    None
}

fn parse_from_stdin() -> Result<Vec<i64>, std::io::Error> {
    aoc2018::try_fold_stdin(Vec::new(), |values, buffer| {
        let num = buffer
            .trim()
            .parse::<i64>()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        values.push(num);

        Ok(())
    })
}

#[test]
fn scan_works_as_expected() {
    assert_eq!(part2(&[1i64, -2, 3, 1]), Some(2));
}
