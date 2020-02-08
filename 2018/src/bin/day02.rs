fn main() {
    let ids = parse_serial_from_stdin();

    let part1 = part1(&ids);
    println!("part1: {}", part1);

    assert_eq!(part1, 4920);
}

fn parse_serial_from_stdin() -> Vec<String> {
    use std::io::BufRead;
    let stdin = std::io::stdin();

    stdin.lock().lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn part1<S: AsRef<str>>(ss: &[S]) -> i64 {
    let mut repeated_counts = [0i64; 2];

    for serial in ss {
        let mut char_counts = [0i64; 30];

        for ch in serial.as_ref().chars() {
            char_counts[(ch as u8 - 'a' as u8) as usize] += 1;
        }

        let mut visited = [false; 2];

        for &counter in char_counts.iter() {
            let index = match counter {
                2 => 0,
                3 => 1,
                _ => continue
            };

            if !visited[index] {
                repeated_counts[index] += 1;
                visited[index] = true;
            }
        }
    }

    repeated_counts.iter().product()
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
