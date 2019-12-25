use std::iter::FromIterator;
use std::str::FromStr;
use std::fmt;
use std::io::Read;
use std::collections::{HashSet, VecDeque};

fn main() {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    locked.read_to_string(&mut buffer).unwrap();


    let bt = BugTile::from_str(buffer.as_str()).unwrap();

    {
        let mut seen = HashSet::new();
        let mut bt = bt.clone();

        while !seen.contains(&bt) {
            seen.insert(bt);

            bt = bt.next();
        }

        println!("stage1: {}", bt.biodiversity_value());
    }

    let mut tiles = RecursiveBugTiles::from(bt);
    for _ in 0..200 {
        tiles.next_mut();
    }

    println!("stage2: {}", tiles.bug_count());
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct BugTile(u32);

/// 5x5 tile where LSB is (0,0) and 25th bit is (4,4)
impl BugTile {
    fn next(&self) -> BugTile {
        let w = 5i8;
        let h = 5i8;

        let d = &[
            ( 0, 1),
            ( 1, 0),
            ( 0,-1),
            (-1, 0),
        ];

        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            //.inspect(|x| println!("coord: {:?}", x))
            .map(|p1| {
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

    fn next_recursive(&self, inner: Option<&BugTile>, outer: Option<&BugTile>) -> BugTile {
        let w = 5i8;
        let h = 5i8;

        let d = &[
            ( 0, 1),
            ( 1, 0),
            ( 0,-1),
            (-1, 0),
        ];

        // out of bounds coordinates could be just mapped to outers?
        //
        // (0,0) | (1,0) | (2,0) | (3,0) | (4,0)
        // (0,1) | (1,1) | (2,1) | (3,1) | (4,1)
        // (0,2) | (1,2) | (2,2) | (3,2) | (4,2)
        // (0,3) | (1,3) | (2,3) | (3,3) | (4,3)
        // (0,4) | (1,4) | (2,4) | (3,4) | (4,4)

        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|p1| p1 != &(2, 2))
            //.inspect(|x| println!("coord: {:?}", x))
            .map(|p1| {
                let neighbours = d.iter()
                    .map(|d| (p1.0 + d.0, p1.1 + d.1))
                    .map(|p2| (p2, match p2 {
                        (x, y) if x < 0 || x > 4 || y < 0 || y > 4 => {

                            let q = match (x, y) {
                                ( _,-1) => ( 2, 1),
                                ( 5, _) => ( 3, 2),
                                ( _, 5) => ( 2, 3),
                                (-1, _) => ( 1, 2),
                                p2 => unreachable!("attempted to query OOB coord {:?}", p2),
                            };

                            //println!("out-of-bounds: querying {:?} as {:?} -> {:?}", q, p1, p2);

                            let bug = outer.and_then(|bt| bt.bug_at(q)).unwrap_or(false);

                            bug as usize
                        },
                        (2, 2) => {
                            match p1 {
                                (2, 1) => inner.map(|bt| bt.bugs_at_edge(Edge::Top)).unwrap_or(0),
                                (1, 2) => inner.map(|bt| bt.bugs_at_edge(Edge::Left)).unwrap_or(0),
                                (3, 2) => inner.map(|bt| bt.bugs_at_edge(Edge::Right)).unwrap_or(0),
                                (2, 3) => inner.map(|bt| bt.bugs_at_edge(Edge::Bottom)).unwrap_or(0),
                                p1 => unreachable!("{:?} not adjacent to (2, 2)", p1),
                            }
                        },
                        p2 => self.bug_at(p2).unwrap() as usize,
                    }))
                    //.inspect(|(p2, bugs)| println!("{:?} -> {:?} bugs: {}", p1, p2, bugs))
                    .map(|(_, bugs)| bugs)
                    .sum();

                (p1, (self.bug_at(p1).unwrap(), neighbours))
            })
            //.inspect(|x| println!("state: {:?}", x))
            .map(|(p1, x)| (p1.0 as usize + (p1.1 * w) as usize, x))
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

    fn bugs_at_edge(&self, edge: Edge) -> usize {
        let indices = match edge {
            Edge::Top => [0, 1, 2, 3, 4],
            Edge::Right => [4, 9, 14, 19, 24],
            Edge::Bottom => [20, 21, 22, 23, 24],
            Edge::Left => [0, 5, 10, 15, 20],
        };

        indices.into_iter().filter(|i| self.bug_at_nth(**i).unwrap()).count()
    }

    fn biodiversity_value(&self) -> usize {
        (0..25)
            .map(|i| if self.bug_at_nth(i as i8).unwrap() { 2usize.pow(i) } else { 0 })
            .sum()
    }

    fn create_inner(&self) -> Option<BugTile> {
        //println!("===\ncreating inner\n===");
        let bt = BugTile(0).next_recursive(None, Some(self));
        if bt.bug_count() > 0 {
            Some(bt)
        } else {
            None
        }
    }

    fn create_outer(&self) -> Option<BugTile> {
        //println!("===\ncreating outer\n===");
        let bt = BugTile(0).next_recursive(Some(self), None);
        if bt.bug_count() > 0 {
            Some(bt)
        } else {
            None
        }
    }

    fn bug_count(&self) -> usize {
        self.0.count_ones() as usize
    }

    fn into_recursive(self) -> BugTile {
        BugTile(self.0 & !(1 << 12))
    }

    fn display_as_recursive(&self) -> RecursiveBugTile {
        RecursiveBugTile(*self)
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

            //println!("final: ({:?}, {})", (i % 5, i / 5), alive);

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

struct RecursiveBugTile(BugTile);

impl fmt::Display for RecursiveBugTile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..25 {
            if i > 0 && i % 5 == 0 {
                writeln!(fmt)?;
            }

            let ch = if i == 12 {
                '?'
            } else if self.0.bug_at_nth(i).unwrap() {
                '#'
            } else {
                '.'
            };

            write!(fmt, "{}", ch)?;
        }
        Ok(())
    }
}

struct RecursiveBugTiles {
    deq: VecDeque<BugTile>,
    init_pos: usize,
}

impl From<BugTile> for RecursiveBugTiles {
    fn from(init: BugTile) -> Self {
        let mut deq = VecDeque::with_capacity(3 * 6);
        deq.push_back(init.into_recursive());
        RecursiveBugTiles { deq, init_pos: 0 }
    }
}

impl RecursiveBugTiles {
    fn next_mut(&mut self) {

        let drained = self.deq.drain(..).collect::<Vec<_>>();

        for (i, bt) in drained.iter().enumerate() {
            let outer = i.checked_sub(1).and_then(|i| drained.get(i));
            let inner = i.checked_add(1).and_then(|i| drained.get(i));
            let next = bt.next_recursive(inner, outer);

            if outer.is_none() {
                if let Some(created) = drained[i].create_outer() {
                //if let Some(created) = next.create_outer() {
                    assert_eq!(self.deq.len(), 0);
                    self.deq.push_back(created);
                    self.init_pos += 1;
                }
            }

            self.deq.push_back(next);
        }

        if let Some(created) = drained.last().unwrap().create_inner() {
        //if let Some(created) = self.deq.back().unwrap().create_inner() {
            self.deq.push_back(created);
        }
    }

    fn len(&self) -> usize {
        self.deq.len()
    }

    fn bug_count(&self) -> usize {
        self.deq.iter()
            .map(|bt| bt.bug_count())
            .sum()
    }
}

impl fmt::Display for RecursiveBugTiles {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (i, bt) in self.deq.iter().enumerate() {
            write!(fmt, "Depth {}:\n{}\n\n", i as isize - self.init_pos as isize, bt.display_as_recursive())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
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

#[test]
fn part2_full_example() {
    let mut tiles = RecursiveBugTiles::from(BugTile::from_str(FIRST_EXAMPLE).unwrap());

    for _ in 0..10 {
        tiles.next_mut();
    }

    assert_eq!(tiles.len(), 11);
    assert_eq!(tiles.bug_count(), 99);
}
