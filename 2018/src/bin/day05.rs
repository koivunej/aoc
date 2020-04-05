fn main() {
    use std::io::stdin;
    let mut buffer = String::new();
    let _ = stdin().read_line(&mut buffer).unwrap();

    let mut reduced = reduce(buffer.trim().chars());
    let part1 = reduced.len();

    println!("part1: {}", part1);

    let part2 = (b'a'..b'z')
        .map(move |removed| {
            reduced.clear();
            let filtered = buffer
                .trim()
                .chars()
                .filter(|ch| ch.to_ascii_lowercase() as u8 != removed);
            reduce_into(filtered, &mut reduced);
            reduced.len()
        })
        .min()
        .unwrap();

    println!("part2: {}", part2);

    assert_ne!(part1, 17220);
    assert_eq!(part1, 9822);

    assert_eq!(part2, 5726);
}

fn reduce(buffer: impl Iterator<Item = char>) -> Vec<u8> {
    let sz = buffer.size_hint();
    let mut reduced = Vec::with_capacity(sz.1.unwrap_or(sz.0));

    reduce_into(buffer, &mut reduced);

    reduced
}

fn reduce_into(buffer: impl Iterator<Item = char>, reduced: &mut Vec<u8>) {
    let mut last = None;

    for ch in buffer {
        let prev = std::mem::replace(&mut last, Some(ch));

        match (prev, ch) {
            (None, ch) => reduced.push(ch as u8),
            (Some(a), b) if !reaction(a, b) => reduced.push(b as u8),
            (Some(_), _) => {
                reduced.pop().unwrap();

                while reduced.len() > 2 {
                    let last = &reduced[reduced.len() - 1];
                    let prev = &reduced[reduced.len() - 2];

                    if !reaction(*last as char, *prev as char) {
                        break;
                    }

                    reduced.truncate(reduced.len() - 2);
                }

                last = reduced.last().map(|&byte| byte as char);
            }
        }
    }
}

fn reaction(a: char, b: char) -> bool {
    a != b && a.to_ascii_lowercase() == b.to_ascii_lowercase()
}

#[test]
fn part1_example() {
    let input = "dabAcCaCBAcCcaDA";

    let reduced = reduce(input);

    assert_eq!(reduced, b"dabCBAcaDA");
}
