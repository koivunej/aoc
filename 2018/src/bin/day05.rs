fn main() {
    use std::io::stdin;
    let mut buffer = String::new();
    let _ = stdin().read_line(&mut buffer).unwrap();

    let part1 = reduce(buffer.trim()).len();

    println!("part1: {}", part1);

    assert_ne!(part1, 17220);
    assert_eq!(part1, 9822);
}

fn reduce(buffer: &str) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(buffer.len());

    let mut last = None;

    for ch in buffer.chars() {
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

    reduced
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
