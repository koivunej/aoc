use intcode::{util::GameDisplay, Word};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
}

fn steps_to_collect_all_keys(m: &Map) -> usize {
    let mut last_key: Option<char> = None;
    let mut last_door: Option<char> = None;

    unimplemented!()
}

struct Map {
    gd: GameDisplay<Tile>,
    poi: HashMap<Tile, (Word, Word)>,
}

impl fmt::Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.gd.fmt(fmt)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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
        use std::collections::hash_map::Entry;
        let (gd, poi) = s
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
            .fold(Ok((GameDisplay::default(), HashMap::new())), |gd, (pos, tile)| {
                let (mut gd, mut poi) = gd.unwrap();
                let tile = tile?;

                match &tile {
                    &Tile::Portal
                    | &Tile::Door(_)
                    | &Tile::Key(_) => {
                        let old = poi.insert(tile.clone(), pos.clone());
                        if let Some(x) = old {
                            panic!("duplicate coordinates for tile {:?}: {:?} and {:?}", tile, pos, x);
                        }
                    },
                    _ => {}
                }

                gd.insert(&pos, tile);
                Ok((gd, poi))
            })?;

        Ok(Map {
            gd,
            poi,
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
    assert_eq!(Some(&(5, 1)), m.poi.get(&Tile::Portal));
    assert_eq!(Some(&(7, 1)), m.poi.get(&Tile::Key('a')));
    assert_eq!(Some(&(3, 1)), m.poi.get(&Tile::Door('A')));
    assert_eq!(Some(&(1, 1)), m.poi.get(&Tile::Key('b')));

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
