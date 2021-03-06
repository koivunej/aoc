use std::fmt;
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();

    let (width, map) = process(stdin.lock())?;
    let height = map.len() / width;

    let part_one = gol_until_settled(RuleSet::PartOne, width, height, &map);
    let part_two = gol_until_settled(RuleSet::PartTwo, width, height, &map);

    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(part_one, 2183);
    assert_eq!(part_two, 1990);

    Ok(())
}

fn all_coordinates(width: i32) -> impl Iterator<Item = (i32, i32)> {
    std::iter::successors(Some((0i32, 0i32)), move |(x, y)| {
        let next_x = x + 1;
        if next_x >= width {
            Some((0, y + 1))
        } else {
            Some((next_x, *y))
        }
    })
}

enum RuleSet {
    PartOne,
    PartTwo,
}

impl RuleSet {
    fn count_adjacent_taken(
        &self,
        old: &[Spot],
        coord: (i32, i32),
        width: usize,
        height: usize,
    ) -> usize {
        (match self {
            RuleSet::PartOne => vanilla_taken,
            RuleSet::PartTwo => directional_taken,
        })(old, coord, width, height)
    }

    fn too_many_occupied_seats(&self) -> usize {
        match self {
            RuleSet::PartOne => 4,
            RuleSet::PartTwo => 5,
        }
    }
}

static DIRECTION_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

fn vanilla_taken(old: &[Spot], coord: (i32, i32), width: usize, height: usize) -> usize {
    // just look at each direction
    DIRECTION_OFFSETS
        .iter()
        .zip(std::iter::repeat(coord))
        .filter_map(|(a, b)| (width, height).to_index((a.0 + b.0, a.1 + b.1)))
        .filter(|&idx| old[idx] == Spot::TakenSeat)
        .count()
}

trait MapSize {
    fn to_index(&self, coords: (i32, i32)) -> Option<usize>;
}

impl MapSize for (usize, usize) {
    fn to_index(&self, (x, y): (i32, i32)) -> Option<usize> {
        let width = self.0;
        let height = self.1;

        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let x = x as usize;
            let y = y as usize;

            Some(y * width + x)
        } else {
            None
        }
    }
}

fn directional_taken(old: &[Spot], coord: (i32, i32), width: usize, height: usize) -> usize {
    DIRECTION_OFFSETS
        .iter()
        .inspect(|_dir| {
            #[cfg(test)]
            println!("looking at direction {:?}", _dir);
        })
        .filter_map(|&(dx, dy)| {
            // walk multiple into this direction
            let mut first = (1..)
                .map(|c| (coord.0 + (c * dx), coord.1 + (c * dy)))
                .map(|coord| (width, height).to_index(coord).map(|idx| (idx, coord)))
                // take_while the index is on the map
                .take_while(|maybe| maybe.is_some())
                // unwrap the Option<usize>
                .map(|maybe| maybe.expect("this must be Some because would had stopped on None"))
                .map(|(idx, coord)| (old[idx], coord))
                .inspect(|(_spot, _at_coord)| {
                    #[cfg(test)]
                    println!("  {:?} looking at {:?} at {:?}", coord, _spot, _at_coord)
                })
                // filter out the floors so that the first is either a seat or nothing
                .filter(|&(spot, _)| spot != Spot::Floor);

            // we want to find the number of taken seats in every direction so use one and zero
            // to be able to sum them up
            first.next().map(|(kind, _)| match kind {
                Spot::Floor => unreachable!(),
                Spot::TakenSeat => 1,
                Spot::EmptySeat => 0,
            })
        })
        .inspect(|_count| {
            #[cfg(test)]
            println!("  --> {:?}", _count);
        })
        .sum::<usize>()
}

fn gol_until_settled(rules: RuleSet, width: usize, height: usize, map: &[Spot]) -> usize {
    let mut first = map.to_vec();
    let mut second = map.to_vec();

    let mut old = &mut first;
    let mut new = &mut second;

    for _i in 0.. {
        gol_round(&rules, old, new, width, height);
        #[cfg(test)]
        println!("\n{:?}", MapDebug(new, width));

        if old == new {
            break;
        }

        std::mem::swap(&mut old, &mut new);

        #[cfg(test)]
        if _i > 7 {
            panic!();
        }
    }

    first.iter().filter(|&&s| s == Spot::TakenSeat).count()
}

fn gol_round(rules: &RuleSet, old: &[Spot], new: &mut [Spot], width: usize, height: usize) {
    let seat_adjacent_counts = old
        .iter()
        .zip(all_coordinates(width as i32))
        .map(|(spot, coord)| {
            if *spot == Spot::Floor {
                (Spot::Floor, 0, coord)
            } else {
                let count = rules.count_adjacent_taken(&old, coord, width, height);
                assert!(count <= 8);
                (*spot, count, coord)
            }
        })
        .inspect(|(_spot, _count, _coord)| {
            #[cfg(test)]
            {
                if _coord.0 == 0 && _coord.1 > 0 {
                    println!();
                }

                match _spot {
                    Spot::Floor => print!("."),
                    _ => print!("{}", _count),
                }
            }
        })
        .map(|(spot, count, _)| (spot, count));

    for (target, (current_spot, count)) in new.iter_mut().zip(seat_adjacent_counts) {
        *target = match current_spot {
            Spot::TakenSeat if count >= rules.too_many_occupied_seats() => Spot::EmptySeat,
            Spot::EmptySeat if count == 0 => Spot::TakenSeat,
            Spot::Floor => {
                assert_eq!(count, 0);
                Spot::Floor
            }
            x => x,
        };
    }

    #[cfg(test)]
    println!("\n");
}

fn process<I: BufRead>(
    mut input: I,
) -> Result<(usize, Vec<Spot>), Box<dyn std::error::Error + 'static>> {
    let mut buffer = String::new();

    let mut spots = Vec::new();

    let mut width = None;

    loop {
        buffer.clear();
        let read = input.read_line(&mut buffer)?;

        if read == 0 {
            break;
        }

        let buffer = buffer.trim();

        if let Some(width) = width.as_ref() {
            assert_eq!(buffer.len(), *width);
        } else {
            width = Some(buffer.len());
        }

        spots.extend(buffer.as_bytes().iter().map(|ch| match ch {
            b'.' => Spot::Floor,
            b'L' => Spot::EmptySeat,
            #[cfg(test)]
            b'#' => Spot::TakenSeat,
            x => unreachable!("invalid byte {} in {:?}", x, buffer),
        }));
    }

    Ok((width.unwrap(), spots))
}

struct MapDebug<'a>(&'a [Spot], usize);

impl<'a> fmt::Debug for MapDebug<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .zip(all_coordinates(self.1 as i32))
            .try_for_each(|(spot, (x, _))| {
                if x == 0 {
                    writeln!(fmt)?;
                }
                write!(
                    fmt,
                    "{}",
                    match spot {
                        Spot::Floor => '.',
                        Spot::TakenSeat => '#',
                        Spot::EmptySeat => 'L',
                    }
                )
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spot {
    Floor,
    EmptySeat,
    TakenSeat,
}

#[test]
fn first_example() {
    let input = b"L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL";

    let (width, map) = process(std::io::BufReader::new(std::io::Cursor::new(input))).unwrap();
    let height = map.len() / width;

    assert_eq!(gol_until_settled(RuleSet::PartOne, width, height, &map), 37);
}

#[test]
fn second_example() {
    let input = b"L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL";

    let (width, map) = process(std::io::BufReader::new(std::io::Cursor::new(input))).unwrap();
    let height = map.len() / width;

    let mut first = map.to_vec();
    let mut second = map.to_vec();

    let mut old = &mut first;
    let mut new = &mut second;

    let phases = [
        "#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##",
        "#.LL.LL.L#
#LLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLL#
#.LLLLLL.L
#.LLLLL.L#",
        "#.L#.##.L#
#L#####.LL
L.#.#..#..
##L#.##.##
#.##.#L.##
#.#####.#L
..#.#.....
LLL####LL#
#.L#####.L
#.L####.L#",
        "#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##LL.LL.L#
L.LL.LL.L#
#.LLLLL.LL
..L.L.....
LLLLLLLLL#
#.LLLLL#.L
#.L#LL#.L#",
        "#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##L#.#L.L#
L.L#.#L.L#
#.L####.LL
..#.#.....
LLL###LLL#
#.LLLLL#.L
#.L#LL#.L#",
        "#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##L#.#L.L#
L.L#.LL.L#
#.LLLL#.LL
..#.L.....
LLL###LLL#
#.LLLLL#.L
#.L#LL#.L#",
    ];

    for (i, &phase) in phases.iter().enumerate() {
        gol_round(&RuleSet::PartTwo, old, new, width, height);

        let got = format!("{:?}", MapDebug(new, width));
        let got = got.trim();

        for (a, b) in got.lines().zip(phase.lines()) {
            println!("{}\t{}", a, b);
        }
        println!();

        if got != phase {
            panic!("round {} failed", i);
        }

        std::mem::swap(&mut old, &mut new);
    }

    assert_eq!(gol_until_settled(RuleSet::PartTwo, width, height, &map), 26);
}

#[test]
fn part_two_directional_examples() {
    let map0 = b".......#.
...#.....
.#.......
.........
..#L....#
....#....
.........
#........
...#.....";
    let map1 = b".............
.L.L.#.#.#.#.
.............";
    let map2 = b".##.##.
#.#.#.#
##...##
...L...
##...##
#.#.#.#
.##.##.";

    let examples = [
        (&map0[..], (3, 4), 8),
        (&map1[..], (1, 1), 0),
        (&map2[..], (3, 3), 0),
    ];

    for &(input, pos, occupied_seats) in &examples {
        let (width, map) = process(std::io::BufReader::new(std::io::Cursor::new(input))).unwrap();
        let height = map.len() / width;

        let found = directional_taken(&map, pos, width, height);

        if found != occupied_seats {
            println!("{:?}", MapDebug(&map, width));
        }

        assert_eq!(found, occupied_seats);
    }
}
