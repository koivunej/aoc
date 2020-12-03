fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buf = String::new();

    let mut width = None;
    let mut pos = Position { x: 0, y: 0 };

    let mut part_one = 0;
    let mut part_two = 0;
    loop {
        buf.clear();
        let read = stdin.read_line(&mut buf)?;
        if read == 0 {
            break;
        }

        let buf = buf.trim();

        if let Some(width) = width.as_ref().copied() {
            assert_eq!(buf.len() as i64, width);
        } else {
            width = Some(buf.len() as i64);
        }

        assert!(pos.x >= 0);
        // input is ascii
        let tree = buf.as_bytes()[pos.x as usize] == b'#';

        if tree {
            part_one += 1;
        }

        pos += Position { x: 3, y: 1 };
        pos.wrap_around_width(width.as_ref().copied().unwrap());
    }

    println!("{}", part_one);

    Ok(())
}

struct Position {
    x: i64,
    y: i64,
}

impl std::ops::AddAssign<Position> for Position {
    fn add_assign(&mut self, other: Position) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Position {
    fn wrap_around_width(&mut self, width: i64) {
        self.x %= width;
    }
}
