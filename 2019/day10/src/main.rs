use std::convert::TryFrom;
use std::fmt::{Display, self};
use std::collections::HashSet;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    let mut buffer = String::new();
    let mut all = String::new();

    let mut width = None;
    let mut height = 0;

    loop {
        buffer.clear();
        match locked.read_line(&mut buffer).unwrap() {
            0 => break,
            _ => {
                width = width.take().or_else(|| Some(buffer.trim().len()));
                all += buffer.trim();
                height += 1;
            }
        }
    }


    println!("stage1: {}", best_asteroid_for_monitoring(&[&all], (width.unwrap(), height)).1);
}

type Size = (usize, usize);
type Point = (isize, isize);

fn best_asteroid_for_monitoring(map: &[&str], (width, height): Size) -> (Point, usize) {
    let map = Map::parse(map, (width, height)).unwrap();

    let mut max = None;

    let mut uniq = HashSet::new();

    for (x0, y0) in map.asteroid_points() {
        uniq.clear();

        // not sure if there is a better than O(n²) for this
        for (x1, y1) in map.asteroid_points() {
            // invert y
            let (dx, dy) = (x1 - x0, -(y1 - y0));

            let degrees = f64::atan2(dx as f64, dy as f64).to_degrees();

            // well this is sketchy
            uniq.insert((100.0 * degrees) as i64);
        }

        // poor mans max_by_key
        max = max.take()
            .map(|(p, asteroids)| if asteroids > uniq.len() { (p, asteroids) } else { ((x0, y0), uniq.len()) })
            .or_else(|| Some(((x0, y0), uniq.len())));
    }

    max.unwrap()
}

#[derive(Debug, PartialEq, Clone)]
enum Element {
    Empty,
    Asteroid,
}

impl TryFrom<char> for Element {
    type Error = ();

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        Ok(match ch {
            '#' => Element::Asteroid,
            '.' => Element::Empty,
            _ => return Err(()),
        })
    }
}

struct Map {
    map: Vec<Element>,
    size: Size,
}

impl Map {
    fn parse(parts: &[&str], size: Size) -> Result<Self, <Element as TryFrom<char>>::Error> {
        let map = parts.iter()
            .flat_map(|p| p.chars())
            .filter(|ch| *ch == '#' || *ch == '.')
            .map(|ch| Element::try_from(ch))
            .collect::<Result<Vec<Element>, _>>()?;

        assert_eq!(map.len(), size.0 * size.1);
        Ok(Map {
            map,
            size,
        })
    }

    fn iter(&self) -> impl Iterator<Item = &Element> + Clone {
        self.map.iter()
    }

    fn asteroid_points<'a>(&'a self) -> impl Iterator<Item = Point> + 'a {
        let size = self.size;
        self.iter()
            .enumerate()
            .filter(|(_, e)| **e == Element::Asteroid)
            .map(move |(i, _)| Self::offset_to_point(size, i))
    }

    fn offset_to_point(size: Size, offset: usize) -> Point {
        ((offset % size.0) as isize, (offset / size.0) as isize)
    }
}

impl std::ops::Index<Point> for Map {
    type Output = Element;

    fn index(&self, (x, y): Point) -> &Self::Output {
        &self.map[y as usize * self.size.0 + x as usize]
    }
}

impl Display for Element {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", if *self == Element::Asteroid { '#' } else { '.' })
    }
}

impl Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        for y in 0..(self.size.1) {
            for x in 0..(self.size.0) {
                write!(fmt, "{}", self[(x as isize, y as isize)])?;
            }
            writeln!(fmt, "")?;
        }

        Ok(())
    }
}

#[test]
fn stage1_first_example() {
    let map = &".#..#\
.....\
#####\
....#\
...##";

    assert_eq!(best_asteroid_for_monitoring(&[map], (5, 5)), ((3, 4), 8));
}

#[test]
fn stage1_second_example() {
    let map = &[
        "......#.#.",
        "#..#.#....",
        "..#######.",
        ".#.#.###..",
        ".#..#.....",
        "..#....#.#",
        "#..#....#.",
        ".##.#..###",
        "##...#..#.",
        ".#....####",
    ];

    assert_eq!(best_asteroid_for_monitoring(&map[..], (map[0].len(), map.len())), ((5, 8), 33));
}

#[test]
fn stage1_third_example() {
    let map = &[
        "#.#...#.#.",
        ".###....#.",
        ".#....#...",
        "##.#.#.#.#",
        "....#.#.#.",
        ".##..###.#",
        "..#...##..",
        "..##....##",
        "......#...",
        ".####.###.",
    ];

    assert_eq!(best_asteroid_for_monitoring(&map[..], (map[0].len(), map.len())), ((1, 2), 35));
}

#[test]
fn stage1_fourth_example() {
    let map = &[
        ".#..#..###",
        "####.###.#",
        "....###.#.",
        "..###.##.#",
        "##.##.#.#.",
        "....###..#",
        "..#.#..#.#",
        "#..#.#.###",
        ".##...##.#",
        ".....#.#..",
    ];

    assert_eq!(best_asteroid_for_monitoring(&map[..], (map[0].len(), map.len())), ((6, 3), 41));
}

#[test]
fn stage1_fifth_example() {
    let map = &[
        ".#..##.###...#######",
        "##.############..##.",
        ".#.######.########.#",
        ".###.#######.####.#.",
        "#####.##.#.##.###.##",
        "..#####..#.#########",
        "####################",
        "#.####....###.#.#.##",
        "##.#################",
        "#####.##.###..####..",
        "..######..##.#######",
        "####.##.####...##..#",
        ".#####..#.######.###",
        "##...#.##########...",
        "#.##########.#######",
        ".####.#.###.###.#.##",
        "....##.##.###..#####",
        ".#.#.###########.###",
        "#.#.#.#####.####.###",
        "###.##.####.##.#..##",
    ];

    assert_eq!(best_asteroid_for_monitoring(&map[..], (map[0].len(), map.len())), ((11, 13), 210));

}
