fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buffer = String::new();

    let mut part_one = None;

    loop {
        buffer.clear();
        let read = stdin.read_line(&mut buffer)?;
        if read == 0 {
            break;
        }

        let id = find_seat(buffer.trim().as_bytes()).to_id();

        part_one = part_one.max(Some(id));
    }

    println!("{}", part_one.unwrap());

    Ok(())
}

fn find_seat(ins: &[u8]) -> (u8, u8) {
    let mut main = ins.iter();

    let row = main
        .by_ref()
        .take(7)
        .enumerate()
        .map(|(idx, ch)| (7 - idx, ch))
        .inspect(|&(_, ch)| assert!(ch == &b'F' || ch == &b'B'))
        .map(|(exp, ch)| if ch == &b'B' { 1 << (exp - 1) } else { 0 })
        .fold(0u8, |acc, next| acc | next);

    let seat = main
        .enumerate()
        .map(|(idx, ch)| (3 - idx, ch))
        .inspect(|&(_, ch)| assert!(ch == &b'L' || ch == &b'R'))
        .map(|(exp, ch)| if ch == &b'R' { 1 << (exp - 1) } else { 0 })
        .fold(0u8, |acc, next| acc | next);

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
