use std::convert::TryInto;
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
    assert_eq!(part_two, 2183);

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

fn vanilla_taken(old: &[Spot], coord: (i32, i32), width: usize, height: usize) -> usize {
    let adjacent = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    adjacent
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
    let directions = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    directions
        .iter()
        .filter(|&&(dx, dy)| {
            (1..)
                .map(|c| (coord.0 + (c * dx), coord.1 + (c * dy)))
                .map(|coord| (width, height).to_index(coord))
                .take_while(|maybe| maybe.is_some())
                .filter_map(|maybe| maybe)
                .any(|idx| old[idx] == Spot::TakenSeat)
        })
        .count()
}

fn gol_until_settled(rules: RuleSet, width: usize, height: usize, map: &[Spot]) -> usize {
    let mut first = map.to_vec();
    let mut second = map.to_vec();

    let mut old = &mut first;
    let mut new = &mut second;

    const DUMP: (bool, bool) = (false, true);

    for i in 0.. {
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
            .inspect(|(spot, count, coord)| {
                if DUMP.0 {
                    if coord.0 == 0 {
                        println!();
                    }

                    match spot {
                        Spot::Floor => print!("."),
                        _ => print!("{}", count),
                    }
                }
            })
            .map(|(spot, count, _)| (spot, count));

        {
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
        }

        if DUMP.1 {
            println!();

            new.iter()
                .zip(all_coordinates(width as i32))
                .for_each(|(spot, (x, _))| {
                    if x == 0 {
                        println!();
                    }
                    print!(
                        "{}",
                        match spot {
                            Spot::Floor => '.',
                            Spot::TakenSeat => '#',
                            Spot::EmptySeat => 'L',
                        }
                    );
                });

            println!();
        }

        if old == new {
            break;
        }

        std::mem::swap(&mut old, &mut new);

        if i > 7 && (DUMP.0 || DUMP.1) {
            panic!();
        }
    }

    first.iter().filter(|&&s| s == Spot::TakenSeat).count()
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
            x => unreachable!("invalid byte {} in {:?}", x, buffer),
        }));
    }

    Ok((width.unwrap(), spots))
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

    assert_eq!(gol_until_settled(RuleSet::PartTwo, width, height, &map), 26);
}
