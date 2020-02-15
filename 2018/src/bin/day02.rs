fn main() {
    let ids = parse_serial_from_stdin();

    let part1 = part1(&ids);
    println!("part1: {}", part1);

    let part2 = part2(&ids);
    println!("part2: {}", part2);

    assert_eq!(part1, 4920);
    assert_eq!(part2, "fonbwmjquwtapeyzikghtvdxl");
}

fn parse_serial_from_stdin() -> Vec<String> {
    use std::io::BufRead;
    let stdin = std::io::stdin();

    stdin.lock().lines().collect::<Result<Vec<_>, _>>().unwrap()
}

fn part1<S: AsRef<str>>(ss: &[S]) -> i64 {
    let mut repeated_counts = [0i64; 2];

    for serial in ss {
        let mut char_counts = [0i64; 30];

        for ch in serial.as_ref().chars() {
            char_counts[(ch as u8 - b'a') as usize] += 1;
        }

        let mut visited = [false; 2];

        for &counter in char_counts.iter() {
            let index = match counter {
                2 => 0,
                3 => 1,
                _ => continue,
            };

            if !visited[index] {
                repeated_counts[index] += 1;
                visited[index] = true;
            }
        }
    }

    repeated_counts.iter().product()
}

fn part2<S: AsRef<str>>(ss: &[S]) -> String {
    for (y, left) in ss.iter().enumerate() {
        let left = left.as_ref();

        for (x, right) in ss.iter().enumerate() {
            if x == y {
                continue;
            }

            let right = right.as_ref();
            assert_eq!(left.len(), right.len());

            let mut differences = left
                .chars()
                .zip(right.chars())
                .enumerate()
                .filter(|&(_, (a, b))| a != b)
                .map(|(i, _)| i);

            let first = differences.next();

            if let Some(i) = first {
                if differences.next().is_none() {
                    let mut ret = String::new();
                    ret.push_str(&left[..i]);
                    ret.push_str(&left[i + 1..]);
                    return ret;
                }
            }

            continue;
        }
    }

    panic!("couldn't find an id with edit distance of 1");
}

#[test]
fn part1_example() {
    let input: Vec<&'static str> = "abcdef,bababc,abbcde,abcccd,aabcdd,abcdee,ababab"
        .split(',')
        .collect::<Vec<_>>();

    assert_eq!(part1(&input), 12);
}

#[test]
fn part1_accepts_vec_of_strings() {
    let vec_of_strings = vec![String::from("a"), String::from("b")];
    part1(&vec_of_strings);
}

#[test]
fn part2_example() {
    let input = &[
        "abcde", "fghij", "klmno", "pqrst", "fguij", "axcye", "wvxyz",
    ];

    assert_eq!(part2(&input[..]), "fgij");
}
