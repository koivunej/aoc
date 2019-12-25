use std::iter::FromIterator;
use std::str::FromStr;
use std::fmt;
use std::io::Read;
use std::collections::HashSet;

fn main() {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    locked.read_to_string(&mut buffer).unwrap();

    let mut seen = HashSet::new();

    let mut bt = BugTile::from_str(buffer.as_str()).unwrap();

    while !seen.contains(&bt) {
        seen.insert(bt);

        bt = bt.next();
    }

    println!("stage1: {}", bt.biodiversity_value());
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct BugTile(u32);

/// 5x5 tile where LSB is (0,0) and 25th bit is (4,4)
impl BugTile {
    fn next(&self) -> BugTile {
        let w = 5i8;
        let h = 5i8;

        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            //.inspect(|x| println!("coord: {:?}", x))
            .map(|p1| {
                let d = &[
                    ( 0, 1),
                    ( 1, 0),
                    ( 0,-1),
                    (-1, 0),
                ];

                let neighbours = d.iter()
                    .map(|d| (p1.0 + d.0, p1.1 + d.1))
                    .filter_map(|p2| self.bug_at(p2).map(|bug| (p2, bug)))
                    .filter(|(_, bug)| *bug)
                    //.inspect(|x| println!("around {:?}: {:?}", p1, x))
                    .count();

                (self.bug_at_nth(p1.1 * w + p1.0).unwrap(), neighbours)
            })
            .enumerate()
            //.inspect(|x| println!("before: {:?}", x))
            .collect()
    }

    fn bug_at(&self, (x, y): (i8, i8)) -> Option<bool> {
        match (x, y) {
            (x, y) if 0 <= x && x <= 4
                   && 0 <= y && y <= 4 => self.bug_at_nth(y * 5 + x),
            _ => None,
        }
    }

    fn bug_at_nth(&self, nth: i8) -> Option<bool> {
        match nth {
            nth if 0 <= nth && nth < 25 => Some(self.0 & (1 << nth) != 0),
            _ => None
        }
    }

    fn biodiversity_value(&self) -> usize {
        (0..25)
            .map(|i| if self.bug_at_nth(i as i8).unwrap() { 2usize.pow(i) } else { 0 })
            .sum()
    }
}

impl FromIterator<(usize, (bool, usize))> for BugTile {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = (usize, (bool, usize))>
    {
        let val = iter.into_iter().fold(0u32, |acc, (i, (bug, neighbours))| {
            let alive = match (bug, neighbours) {
                (true, 1) => true,
                (true, _) => false,
                (false, 1)
                | (false, 2) => true,
                (x, _) => x
            };

            if alive {
                acc | (1 << i)
            } else {
                acc
            }
        });

        BugTile(val)
    }
}

impl FromIterator<(usize, bool)> for BugTile {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = (usize, bool)>
    {
        let inner = iter.into_iter()
            .fold(0u32, |acc, (index, alive)| acc | ((alive as u32) << index));
        BugTile(inner)
    }
}

impl FromStr for BugTile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars()
            .filter(|ch| *ch != '\n')
            .enumerate()
            .map(|(i, ch)| match ch {
                '.' => Ok((i, false)),
                '#' => Ok((i, true)),
                _ => Err(()),
            })
            .collect()
    }
}

impl fmt::Display for BugTile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..25 {
            if i > 0 && i % 5 == 0 {
                writeln!(fmt)?;
            }

            let ch = if self.bug_at_nth(i).unwrap() {
                '#'
            } else {
                '.'
            };

            write!(fmt, "{}", ch)?;
        }
        Ok(())
    }
}

#[cfg(test)]
const FIRST_EXAMPLE: &str = "\
....#
#..#.
#..##
..#..
#....";

#[test]
fn first_example_roundtrip() {
    let bt = BugTile::from_str(FIRST_EXAMPLE).unwrap();

    assert_eq!(FIRST_EXAMPLE.trim(), format!("{}", bt).trim());
}

#[test]
fn first_example_minutes() {
    let each_minute = &["\
#..#.
####.
###.#
##.##
.##..",
    "\
#####
....#
....#
...#.
#.###",
    "\
#....
####.
...##
#.##.
.##.#",
    "\
####.
....#
##..#
.....
##..."];

    let mut bt = BugTile::from_str(FIRST_EXAMPLE).unwrap();

    for (i, expected) in each_minute.iter().enumerate() {
        bt = bt.next();
        let actual = format!("{}", bt);
        println!("actual:");
        println!("{}", actual);

        assert_eq!(expected.trim(), actual.trim(), "{}min", i + 1);
    }
}
