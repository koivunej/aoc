fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buf = String::new();

    // true for coordinates with trees, false otherwise
    let mut map = Vec::new();

    let mut width = None;
    let mut height = 0;

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

        map.extend(buf.as_bytes().iter().map(|&ch| ch == b'#'));
        height += 1;
    }

    let width = width.unwrap();
    let height = height;

    let slopes = [
        Position { x: 1, y: 1 },
        Position { x: 3, y: 1 },
        Position { x: 5, y: 1 },
        Position { x: 7, y: 1 },
        Position { x: 1, y: 2 },
    ];

    let trees_hit = slopes
        .iter()
        .map(|slope| {
            let mut pos = Position { x: 0, y: 0 };

            let mut trees = 0;

            loop {
                assert!(pos.x >= 0);
                if pos.y >= height {
                    break;
                }

                let tree = map[(pos.x + width * pos.y) as usize];

                if tree {
                    trees += 1;
                }

                pos += &slope;
                pos.wrap_around_width(width);
            }

            trees
        })
        .collect::<Vec<_>>();

    let part_one = trees_hit[1];
    let part_two = trees_hit.into_iter().product::<usize>();

    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(153, part_one);
    assert_eq!(2_421_944_712, part_two);

    Ok(())
}

struct Position {
    x: i64,
    y: i64,
}

impl std::ops::AddAssign<&Position> for Position {
    fn add_assign(&mut self, other: &Position) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Position {
    fn wrap_around_width(&mut self, width: i64) {
        self.x %= width;
    }
}
