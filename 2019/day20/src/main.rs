use std::fmt;
use std::collections::HashMap;
use std::convert::TryFrom;
use intcode::{Word, util::{GameDisplay, Position}};

fn main() {
    let stdin = std::io::stdin();
    let mut output: GameDisplay<ParsedTile> = GameDisplay::default();
    output.parse_from_reader((0, 0), stdin.lock()).unwrap();

    let part1 = shortest_path(&output, ('A', 'A'), ('Z', 'Z'), false);
    println!("part1: {}", part1);
    let part2 = shortest_path(&output, ('A', 'A'), ('Z', 'Z'), true);
    println!("part2: {}", part2);

    assert_eq!(part1, 600);
    assert_eq!(part2, 6666);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedTile {
    Dot,
    Space,
    Wall,
    Key(char),
}

impl Default for ParsedTile {
    fn default() -> ParsedTile {
        ParsedTile::Space
    }
}

impl fmt::Display for ParsedTile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use ParsedTile::*;
        let ch = match *self {
            Dot => '.',
            Space => ' ',
            Wall => '#',
            Key(ch) => ch,
        };

        write!(fmt, "{}", ch)
    }
}

impl TryFrom<char> for ParsedTile {
    type Error = char;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        Ok(match ch {
            ' ' => ParsedTile::Space,
            '.' => ParsedTile::Dot,
            '#' => ParsedTile::Wall,
            ch @ 'A'..='Z' => ParsedTile::Key(ch),
            ch => return Err(ch),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Portal {
    Outer(Position),
    Inner(Position),
}

impl Into<Position> for Portal {
    fn into(self) -> Position {
        match self {
            Portal::Outer(x) | Portal::Inner(x) => x
        }
    }
}

impl Portal {
    fn into_position(self) -> Position {
        self.into()
    }
}

fn find_portal_exits(gd: &GameDisplay<ParsedTile>) -> HashMap<(char, char), Vec<Portal>> {

    let mut min_dot: Option<(Word, Word)> = None;
    let mut max_dot: Option<(Word, Word)> = None;

    // this seems quite wasteful, but couldn't think of a simpler way
    for (x, y) in gd.cells().iter().enumerate().filter_map(|(i, t)| if t == &ParsedTile::Dot { Some(i) } else { None }).map(|i| gd.to_coordinates(i)) {
        min_dot = min_dot.map(|(mx, my)| (mx.min(x), my.min(y))).or(Some((x, y)));
        max_dot = max_dot.map(|(mx, my)| (mx.max(x), my.max(y))).or(Some((x, y)));
    }

    let min_dot = min_dot.unwrap();
    let max_dot = max_dot.unwrap();

    let mut ret = HashMap::new();

    let keys = gd.cells()
        .iter()
        .enumerate()
        .filter_map(|(i, x)| match *x { ParsedTile::Key(ch) => Some((gd.to_coordinates(i), ch)), _ => None });

    let forwards = [(0, 1), (1, 0)];
    let backwards = [(0, -1), (-1, 0)];

    for (p1, key) in keys {
        // here's probably a complication... for which of the two consecutive keys to find the
        // dot...
        //
        // (1)       (2)       (3)      (4)
        //  A                   .
        //  A   or   .AA   or   A   or  AA.
        //  .                   A
        //
        // so maybe just check the backwards first and then depending on a dot recheck forwards...?
        // recheck is not probably needed. probably the same. but forwards needs to first find the
        // key name left to right.

        let mut p2_offsets = forwards
            .iter()
            .map(|p| (p, false))
            .chain(backwards.iter().map(|p| (p, true)))
            .map(|(p, reverse)| (Position::from(p), reverse))
            .map(|(offset, reverse)| (offset + p1, offset, reverse))
            .filter_map(|(coord, offset, reverse)| gd.get(&coord.into()).and_then(|t| match t {
                ParsedTile::Key(ch) => Some((Some((ch, reverse)), offset)),
                ParsedTile::Dot => Some((None, offset)),
                _ => None,
            }));

        let (second_tile, p2_offset) = p2_offsets.next()
            .expect("there should be two adjacent (first)");

        let (opposite_tile, p3_offset) = if let Some(tuple) = p2_offsets.next() {
            tuple
        } else {
            // if we are not looking at the center of the three interesting, just skip
            continue;
        };

        assert_eq!(p2_offsets.next(), None);

        let (other, reverse) = second_tile.or(opposite_tile)
            .expect("did not find other key?");

        let key = if !reverse {
            (key, *other)
        } else {
            (*other, key)
        };

        let dotty_offset = if second_tile.is_none() { p2_offset } else { p3_offset };

        let dot = dotty_offset + p1;

        debug_assert_eq!(gd.get(&dot.into()), Some(&ParsedTile::Dot));

        let at_bounds = dot.x() == min_dot.0 || dot.x() == max_dot.0
            || dot.y() == min_dot.1 || dot.y() == max_dot.1;

        let dot = if at_bounds { Portal::Outer(dot) } else { Portal::Inner(dot) };

        ret.entry(key).or_insert_with(|| Vec::new()).push(dot);
    }

    ret
}

fn shortest_path(gd: &GameDisplay<ParsedTile>, start: (char, char), end_key: (char, char), nested: bool) -> usize {
    use std::collections::BinaryHeap;
    use std::collections::hash_map::Entry;
    use std::cmp;

    let exits = find_portal_exits(gd);

    let start = exits.get(&start)
        .map(|v| { assert_eq!(v.len(), 1); v[0] })
        .expect("AA not found")
        .into_position();

    let end = exits.get(&end_key)
        .map(|v| { assert_eq!(v.len(), 1); v[0] })
        .expect("ZZ not found")
        .into_position();

    let mut work = BinaryHeap::new();
    let mut dist: HashMap<(usize, Position), _> = HashMap::new();
    let mut prev: HashMap<(usize, Position), _> = HashMap::new();

    let forwards = [
        ( 0, 1),
        ( 1, 0),
    ];

    let backwards = [
        ( 0,-1),
        (-1, 0),
    ];

    work.push(cmp::Reverse((0, (0, start))));

    while let Some(cmp::Reverse((steps_here, (level, p)))) = work.pop() {
        if (level, p) == (0, end) {

            /*
            let mut exits_index = HashMap::new();

            for (key, portals) in exits {
                for p in portals {
                    exits_index.insert(p.into_position(), format!("{}{}", key.0, key.1));
                }
            }

            let mut backwards = (level, p);

            let mut path = Vec::new();
            path.push((0, end));

            println!("{:?}", (0, start));
            while backwards != (0, start) {
                //println!("removing {:?}", backwards);
                let previous = prev.remove(&backwards).unwrap_or_else(|| panic!("found nothing for {:?}", backwards));
                path.push(previous);
                backwards = previous;
            }

            path.reverse();
            let mut skipped = 0;

            for (level, pos) in path {
                if let Some(ref name) = exits_index.get(&pos) {
                    println!("{}@{} with {} steps", name, level, skipped);
                    skipped = 0;
                } else {
                    skipped += 1;
                }
            }*/

            // not sure if this is a good place to exit
            return steps_here;
        }

        match dist.entry((level, p)) {
            Entry::Vacant(vcnt) => {
                vcnt.insert(steps_here);
            },
            Entry::Occupied(mut o) => {
                if *o.get() >= steps_here {
                    *o.get_mut() = steps_here;
                } else {
                    println!("already visited {:?} with lower dist {} than {} from {:?}", p, o.get(), steps_here, prev[&(level, p)]);
                    continue;
                }
            }
        }

        let adjacent = forwards.iter()
            .chain(backwards.iter())
            .map(|offset| (Position::from(p) + offset, Position::from(offset)))
            .filter_map(|(p2, offset)| gd.get(&p2.into()).map(|t| (p2, offset, t)))
            .filter_map(|(p2, offset, tile)| match tile {
                ParsedTile::Dot => Some((p2, offset, None)),
                ParsedTile::Key(ch) => Some((p2, offset, Some(ch))), // not sure
                _ => None,
            });

        for (p2, offset, key_part) in adjacent {
            let p2: Position = p2;

            let ((next_level, p2), alt) = match key_part {
                None => ((level, p2), steps_here + 1),
                Some(first) => {
                    let reverse = offset.x() < 0 || offset.y() < 0;

                    let other = match gd.get(&(p2 + offset).into()) {
                        Some(&ParsedTile::Key(ch)) => ch,
                        x => panic!("Expected key on {:?} but found {:?}", p2 + offset, x),
                    };

                    let key = if !reverse {
                        (*first, other)
                    } else {
                        (other, *first)
                    };

                    if nested && (key == ('A', 'A') || key == ('Z', 'Z')) && level != 0 {
                        continue;
                    }

                    // cannot unwrap because of AA which have only one dot
                    let next = exits.get(&key)
                        .and_then(|v| v.iter().filter(|p3| p3.into_position() != p).next())
                        .cloned();

                    if let Some(next) = next {
                        if nested {
                            // next is the **other** side of this ... this took a while to
                            // understand. what a bad decision to use vec for the right hand side
                            // of the hashmap. I even considered using an enum { Outest(Position),
                            // Maze(Position, Position) } but... didn't think I'd need it.
                            //
                            // i was first comparing the next here... and still am.
                            match next {
                                Portal::Inner(_) if level > 0 => {
                                    //println!("allowing breakingout {}{} at {:?}@{}", key.0, key.1, p, level);
                                    ((level - 1, next.into_position()), steps_here + 1)
                                },
                                Portal::Inner(_) => {
                                    //println!("filtering outer {}{} @ {} at {:?}", key.0, key.1, level, p);
                                    continue
                                },
                                Portal::Outer(_) => {
                                    //println!("allowing nesting     {}{} at {:?}@{}", key.0, key.1, p, level);
                                    ((level + 1, next.into_position()), steps_here + 1)
                                },
                            }
                        } else {
                            ((level, next.into_position()), steps_here + 1)
                        }
                    } else {
                        // not a valid adjacent
                        continue;
                    }
                },
            };

            //println!("adjacent({}, {:?}) discovered {}, {:?}", level, p, next_level, p2);

            if alt < *dist.get(&(next_level, p2)).unwrap_or(&usize::max_value()) {
                // println!("  {:?}@{} ---> {:?}@{}", p, level, p2, next_level);
                dist.insert((next_level, p2), alt);
                prev.insert((next_level, p2), (level, p));

                work.push(cmp::Reverse((alt, (next_level, p2))));
            }
        }
    }

    panic!("should have found a path from {:?} to {:?}", start, end)
}

#[cfg(test)]
fn read_first_example() -> GameDisplay<ParsedTile> {
    let first = "\
_________A_________
         A_________
  #######.#########
  #######.........#
  #######.#######.#
  #######.#######.#
  #######.#######.#
  #####  B    ###.#
BC...##  C    ###.#
  ##.##       ###.#
  ##...DE  F  ###.#
  #####    G  ###.#
  #########.#####.#
DE..#######...###.#
  #.#########.###.#
FG..#########.....#
  ###########.#####
             Z_____
             Z_____".replace("_", " ");

    let reader = std::io::Cursor::new(first.as_bytes());

    let mut output: GameDisplay<ParsedTile> = GameDisplay::default();
    output.parse_from_reader((0, 0), reader).unwrap();

    assert_eq!(first, format!("{}", output));

    output
}

#[test]
fn parse_first_map() {
    read_first_example();
}

#[test]
fn first_map_exits() {
    let exits = find_portal_exits(&read_first_example());
    assert_eq!(exits.get(&('A', 'A')), Some(&vec![Portal::Outer((9, 2).into())]));
    assert_eq!(exits.get(&('Z', 'Z')), Some(&vec![Portal::Outer((13, 16).into())]));
    assert_eq!(exits.get(&('B', 'C')), Some(&vec![Portal::Inner((9, 6).into()), Portal::Outer((2, 8).into())]));
}

#[test]
fn first_example_cost() {
    assert_eq!(shortest_path(&read_first_example(), ('A', 'A'), ('Z', 'Z'), false), 23);
}

#[cfg(test)]
fn read_second_example() -> GameDisplay<ParsedTile> {
    let first = "\
___________________A_______________
                   A_______________
  #################.#############__
  #.#...#...................#.#.#__
  #.#.#.###.###.###.#########.#.#__
  #.#.#.......#...#.....#.#.#...#__
  #.#########.###.#####.#.#.###.#__
  #.............#.#.....#.......#__
  ###.###########.###.#####.#.#.#__
  #.....#        A   C    #.#.#.#__
  #######        S   P    #####.#__
  #.#...#                 #......VT
  #.#.#.#                 #.#####__
  #...#.#               YN....#.#__
  #.###.#                 #####.#__
DI....#.#                 #.....#__
  #####.#                 #.###.#__
ZZ......#               QG....#..AS
  ###.###                 #######__
JO..#.#.#                 #.....#__
  #.#.#.#                 ###.#.#__
  #...#..DI             BU....#..LF
  #####.#                 #.#####__
YN......#               VT..#....QG
  #.###.#                 #.###.#__
  #.#...#                 #.....#__
  ###.###    J L     J    #.#.###__
  #.....#    O F     P    #.#...#__
  #.###.#####.#.#####.#####.###.#__
  #...#.#.#...#.....#.....#.#...#__
  #.#####.###.###.#.#.#########.#__
  #...#.#.....#...#.#.#.#.....#.#__
  #.###.#####.###.###.#.#.#######__
  #.#.........#...#.............#__
  #########.###.###.#############__
           B   J   C             __
           U   P   P             __".replace("_", " ");

    let reader = std::io::Cursor::new(first.as_bytes());

    let mut output: GameDisplay<ParsedTile> = GameDisplay::default();
    output.parse_from_reader((0, 0), reader).unwrap();

    assert_eq!(first, format!("{}", output));

    output
}


#[test]
fn parse_second_map() {
    read_second_example();
}

#[test]
fn second_example_cost() {
    assert_eq!(shortest_path(&read_second_example(), ('A', 'A'), ('Z', 'Z'), false), 58);
}

#[test]
fn recursive_path() {
    let first = "\
_____________Z L X W       C               __
             Z P Q B       K               __
  ###########.#.#.#.#######.###############__
  #...#.......#.#.......#.#.......#.#.#...#__
  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###__
  #.#...#.#.#...#.#.#...#...#...#.#.......#__
  #.###.#######.###.###.#.###.###.#.#######__
  #...#.......#.#...#...#.............#...#__
  #.#########.#######.#.#######.#######.###__
  #...#.#    F       R I       Z    #.#.#.#__
  #.###.#    D       E C       H    #.#.#.#__
  #.#...#                           #...#.#__
  #.###.#                           #.###.#__
  #.#....OA                       WB..#.#..ZH
  #.###.#                           #.#.#.#__
CJ......#                           #.....#__
  #######                           #######__
  #.#....CK                         #......IC
  #.###.#                           #.###.#__
  #.....#                           #...#.#__
  ###.###                           #.#.#.#__
XF....#.#                         RF..#.#.#__
  #####.#                           #######__
  #......CJ                       NM..#...#__
  ###.#.#                           #.###.#__
RE....#.#                           #......RF
  ###.###        X   X       L      #.#.#.#__
  #.....#        F   Q       P      #.#.#.#__
  ###.###########.###.#######.#########.###__
  #.....#...#.....#.......#...#.....#.#...#__
  #####.#.###.#######.#######.###.###.#.#.#__
  #.......#.......#.#.#.#.#...#...#...#.#.#__
  #####.###.#####.#.#.#.#.###.###.#.###.###__
  #.......#.....#.#...#...............#...#__
  #############.#.#.###.###################__
               A O F   N                   __
               A A D   M                   __".replace("_", " ");

    let reader = std::io::Cursor::new(first.as_bytes());

    let mut output: GameDisplay<ParsedTile> = GameDisplay::default();
    output.parse_from_reader((0, 0), reader).unwrap();

    assert_eq!(first, format!("{}", output));

    let exits = find_portal_exits(&output);

    assert_eq!(exits[&('O', 'A')], &[Portal::Inner((8, 13).into()), Portal::Outer((17, 34).into())]);
    assert_eq!(exits[&('X', 'F')], &[Portal::Outer((2, 21).into()), Portal::Inner((17, 28).into())]);

    assert_eq!(shortest_path(&output, ('A', 'A'), ('Z', 'Z'), true), 396);
}
