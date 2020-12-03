fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();
    let mut width = None;

    let slopes = &[(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];

    let mut counters = slopes
        .iter()
        .map(|slope| TreeCounter {
            slope,
            x: 0,
            skip: 1,
            trees: 0,
        })
        .collect::<Vec<_>>();

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

        counters.iter_mut().for_each(|ctr| ctr.process(buf))
    }

    let counts = counters
        .into_iter()
        .map(|ctr| ctr.into_count())
        .collect::<Vec<_>>();

    let part_one = counts[1];
    let part_two = counts.iter().product::<usize>();

    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(153, part_one);
    assert_eq!(2_421_944_712, part_two);

    Ok(())
}

struct TreeCounter {
    slope: &'static (u16, u8),
    x: u16,
    skip: u8,
    trees: usize,
}

impl TreeCounter {
    fn process(&mut self, map: &str) {
        self.skip -= 1;
        if self.skip > 0 {
            return;
        }
        let tree = map.as_bytes()[self.x as usize] == b'#';

        if tree {
            self.trees += 1;
        }

        self.x = (self.x + self.slope.0) % map.len() as u16;
        self.skip = self.slope.1;
    }

    fn into_count(self) -> usize {
        self.trees
    }
}
