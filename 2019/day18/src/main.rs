use intcode::{util::GameDisplay, Word};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

fn main() {
    println!("Hello, world!");
}

fn steps_to_collect_all_keys(m: &Map) -> usize {
    unimplemented!()
}

struct Map {
    gd: GameDisplay<Tile>,
    portal_at: (Word, Word),
}

impl fmt::Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.gd.fmt(fmt)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Portal,
    Key(char),
    Door(char),
}

impl fmt::Display for Tile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ch = match *self {
            Tile::Empty => '.',
            Tile::Wall => '#',
            Tile::Portal => '@',
            Tile::Key(ch) => ch,
            Tile::Door(ch) => ch,
        };

        write!(fmt, "{}", ch)
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

#[derive(Debug)]
struct InvalidTile(char);

impl TryFrom<char> for Tile {
    type Error = InvalidTile;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        Ok(match ch {
            '.' => Tile::Empty,
            '#' => Tile::Wall,
            '@' => Tile::Portal,
            ch @ 'a'..='z' => Tile::Key(ch),
            ch @ 'A'..='Z' => Tile::Door(ch),
            x => return Err(InvalidTile(x)),
        })
    }
}

impl FromStr for Map {
    type Err = InvalidTile;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (gd, portal_at) = s
            .trim()
            .chars()
            .scan((0i64, 0i64), |mut acc, ch| match ch {
                '\n' => {
                    acc.1 += 1;
                    acc.0 = 0;
                    Some((*acc, None))
                }
                _ => {
                    let old = *acc;
                    acc.0 += 1;
                    Some((old, Some(ch)))
                }
            })
            .filter_map(|(pos, ch)| match ch {
                Some(ch) => Some((pos, ch)),
                None => None,
            })
            .map(|(pos, ch)| (pos, Tile::try_from(ch)))
            .fold(Ok((GameDisplay::default(), None)), |gd, (pos, tile)| {
                let (mut gd, mut portal_at) = gd.unwrap();
                let tile = tile?;
                portal_at = match (&tile, portal_at) {
                    (&Tile::Portal, None) => Some(pos),
                    (&Tile::Portal, Some(old)) => {
                        panic!("too many portals found: {:?} and {:?}", old, pos)
                    }
                    (_, x @ _) => x,
                };

                gd.insert(&pos, tile);
                Ok((gd, portal_at))
            })?;

        Ok(Map {
            gd,
            portal_at: portal_at.expect("There must be a portal somewhere in the map"),
        })
    }
}

#[test]
fn parse_first_map() {
    let s = "\
#########
#b.A.@.a#
#########";

    let m = Map::from_str(s).unwrap();

    assert_eq!(s.trim(), format!("{}", m).trim());
    assert_eq!((5, 1), m.portal_at);

    assert_eq!(&Tile::Wall, m.gd.get(&(0, 0)).unwrap());
}

#[test]
fn full_first_example() {
    let s = "\
#########
#b.A.@.a#
#########";

    let m = Map::from_str(s).unwrap();

    assert_eq!(steps_to_collect_all_keys(&m), 8);
}
