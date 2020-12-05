fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buffer = String::new();
    let mut part_one = None;
    let mut all = vec![];

    loop {
        buffer.clear();
        let read = stdin.read_line(&mut buffer)?;
        if read == 0 {
            break;
        }

        let id = find_seat(buffer.trim().as_bytes()).to_id();

        part_one = part_one.max(Some(id));

        // collect in order to sort, and find the missing one in between
        all.push(id);
    }

    all.sort();

    let missing_one_in_between = all
        .windows(2)
        .filter_map(|w| {
            let earlier = w[0];
            let later = w[1];
            assert!(earlier < later);
            if later - earlier == 2 {
                Some(later - 1)
            } else {
                None
            }
        })
        .fold(None, |acc, next| {
            if acc.is_none() {
                Some(next)
            } else {
                panic!("multiple results: {} and {}", acc.unwrap(), next);
            }
        });

    let part_one = part_one.unwrap();
    let part_two = missing_one_in_between.unwrap();
    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(892, part_one);
    assert_eq!(625, part_two);

    Ok(())
}

fn find_seat(ins: &[u8]) -> (u8, u8) {
    let mut main = ins.iter();

    // just a binary number encoded with FB and LR for 01
    // rewriting to use u8::from_str_radix would probably take too long

    let row = main
        .by_ref()
        .copied()
        .take(7)
        .enumerate()
        .inspect(|&(_, ch)| assert!(ch == b'F' || ch == b'B'))
        .map(|(idx, ch)| if ch == b'B' { 1 << (7 - idx - 1) } else { 0 })
        .fold(0u8, |acc, next| acc | next);

    let seat = main
        .by_ref()
        .copied()
        .take(3)
        .enumerate()
        .inspect(|&(_, ch)| assert!(ch == b'L' || ch == b'R'))
        .map(|(idx, ch)| if ch == b'R' { 1 << (3 - idx - 1) } else { 0 })
        .fold(0u8, |acc, next| acc | next);

    assert_eq!(main.next(), None);

    (row, seat)
}

trait SeatId {
    fn to_id(&self) -> u16;
}

impl SeatId for (u8, u8) {
    fn to_id(&self) -> u16 {
        self.0 as u16 * 8 + self.1 as u16
    }
}

#[test]
fn seat_finding_example() {
    let ins = b"FBFBBFFRLR";
    assert_eq!(find_seat(ins), (44, 5));
}

#[test]
fn seat_id_examples() {
    let examples = [
        (b"BFFFBBFRRR", 70, 7, 567),
        (b"FFFBBBFRRR", 14, 7, 119),
        (b"BBFFBBFRLL", 102, 4, 820),
    ];

    for (example, row, column, id) in &examples {
        let pos = find_seat(&example[..]);
        assert_eq!((*row, *column), pos);
        assert_eq!(*id, pos.to_id());
    }
}
