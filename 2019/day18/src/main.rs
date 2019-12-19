use intcode::util::GameDisplay;
use std::convert::TryFrom;
use std::str::FromStr;
use std::fmt;

fn main() {
    println!("Hello, world!");
}

fn steps_to_collect_all_keys(m: &Map) -> usize {
    unimplemented!()
}

struct Map {
    gd: GameDisplay<Tile>,
}

impl fmt::Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.gd.fmt(fmt)
    }
}

#[derive(Debug, Clone)]
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
        let gd = s.chars()
            .scan((0i64, 0i64), |mut acc, ch| {
                match ch {
                    '\n' => {
                        acc.1 += 1;
                        acc.0 = 0;
                        Some((*acc, None))
                    },
                    _ => {
                        acc.0 += 1;
                        Some((*acc, Some(ch)))
                    }
                }
            })
            .filter_map(|(pos, ch)| match ch {
                Some(ch) => Some((pos, ch)),
                None => None,
            })
            .map(|(pos, ch)| (pos, Tile::try_from(ch)))
            .fold(Ok(GameDisplay::default()), |gd, (pos, tile)| {
                let mut gd = gd.unwrap();
                gd.insert(&pos, tile?);
                Ok(gd)
            })?;

        Ok(Map { gd })
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
}
