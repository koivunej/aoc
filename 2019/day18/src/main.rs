use intcode::{util::GameDisplay, Word};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use std::collections::{HashMap, HashSet, VecDeque};
use ndarray::{s, Array2};
use itertools::Either;

fn main() {
    println!("Hello, world!");
}

fn steps_to_collect_all_keys(m: &mut Map) -> usize {

    let vertices = m.gd.cells().iter()
        .enumerate()
        .filter_map(|(i, t)| match t {
            Tile::Wall => None,
            t => Some((m.gd.to_coordinates(i), t.clone())),
        })
        .collect::<Vec<_>>();

    let vertices_index = vertices.iter()
        .enumerate()
        .map(|(i, (coords, tile))| (coords, i))
        .collect::<HashMap<_, _>>();

    let edges = vertices.iter()
        .flat_map(|(c, t)| m.frontier_where(*c, t).map(move |(target, _tile)| (*c, target)))
        .map(|(from, to)| (vertices_index[&from], vertices_index[&to]));

    let mut dist = ndarray::Array2::<Option<u32>>::from_elem((vertices.len(), vertices.len()), None);
    let mut next = ndarray::Array2::<Option<usize>>::from_elem((vertices.len(), vertices.len()), None);

    //let edges = vertices.iter().enumerate().flat_map(|(i, (pos, _))| m.frontier(&pos).map(move |(next, _)| (pos, next, 1)));

    for (from, to) in edges {
        dist[(from, to)] = Some(1);
        dist[(to, from)] = Some(1);
        next[(from, to)] = Some(to);
        next[(to, from)] = Some(from);
    }

    for (i, _) in vertices.iter().enumerate() {
        assert_eq!(dist[(i, i)], None);
        next[(i, i)] = Some(i);
        //dist[(i, i)] = 0;
    }

    for k in 0..vertices.len() {
        for i in 0..vertices.len() {
            for j in 0..vertices.len() {

                if i == j || i == k || k == j {
                    continue;
                }

                let rhs = dist[(i, k)]
                    .and_then(|ik| dist[(k, j)]
                    .map(|kj| ik + kj));

                if rhs.is_none() {
                    continue;
                }

                let lhs = &mut dist[(i, j)];

                if rhs.is_some() && (lhs.is_none() || lhs.unwrap() > rhs.unwrap()) {
                    let rhs = rhs.unwrap();
                    if lhs.unwrap_or(u32::max_value()) > rhs {
                        *lhs = Some(rhs);
                        next[(i, j)] = next[(i, k)];
                    }
                }
            }
        }
    }

    let all_paths = AllPaths {
        dist: &dist,
        next: &next,
        vertices: vertices.as_slice(),
        index: &vertices_index
    };

    let mut pos = m.initial_position;
    let mut keys = KeySet::default();
    let mut steps = 0;

    let mut all_keys = m.poi.iter().filter(|(t, _)| if let &Tile::Key(_) = t { true } else { false }).collect::<Vec<_>>();
    let mut choices = Vec::new();

    while !all_keys.is_empty() {

        //let chosen_index = None;

        choices.clear();
        choices.extend(
            all_keys.iter()
                .enumerate()
                .map(|(i, (tile, coord))| (i, *tile, all_paths.find_path(coord, &pos, None, &keys)))
                .filter_map(|(i, t, p)| p.map(|(path, keys)| (i, path.len() - 1, *path.first().unwrap(), t, keys)))
        );

        while choices.len() > 1 {

            choices.sort_by_key(|(_, more_steps, _, _, _)| *more_steps);

            let mut purge = Vec::new();

            for (i, x) in choices.iter().enumerate().rev() {
                purge.extend(choices.iter().enumerate().rev().skip(i).filter_map(|(j, y)| if y.4.subset_of(&x.4) && i != j { Some(j) } else { None }));
            }

            for x in purge {
                choices.swap_remove(x);
            }

            /*if choices.len() > 1 {
                unimplemented!("unexpect amount of choices with keys={:?}, steps={}, remaining={:?}: {:?}\n{}", keys, steps, all_keys, choices, m);
            }*/
            break;
            // would need to take a look if any of the paths is a subpath of another... not sure if
            // that is easy... perhaps through keys?
        }

        //if choices.len() == 1 {
            let (i, more_steps, end_up_at, tile, _) = choices.pop().unwrap();
            all_keys.swap_remove(i);

            let old_pos = pos;
            pos = end_up_at;
            keys += tile;
            steps += more_steps;
            println!("moved to {:?}...{:?} at {:?} total {} steps", old_pos, pos, tile, steps);
            continue;
        //}

        unimplemented!("unexpect amount of choices with keys={:?}, steps={}, remaining={:?}: {:?}\n{}", keys, steps, all_keys, choices, m);
    }

    steps
}

struct InterestingPath {
    steps: usize,
    end_up_at: (i64, i64),
    req_prov: KeySet,
}

impl From<(Vec<(Word, Word)>, KeySet)> for InterestingPath {
    fn from((path, req_prov): (Vec<(Word, Word)>, KeySet)) -> Self {
        InterestingPath {
            steps: path.len(),
            end_up_at: *path.last().unwrap(),
            req_prov
        }
    }
}

struct AllPaths<'a, 'b> {
    dist: &'a Array2<Option<u32>>,
    next: &'a Array2<Option<usize>>,
    vertices: &'a [((Word, Word), Tile)],
    index: &'a HashMap<&'b (Word, Word), usize>,
}

impl<'a, 'b> AllPaths<'a, 'b> {
    fn find_path(&self, a: &(Word, Word), b: &(Word, Word), max_len: Option<usize>, keys: &KeySet) -> Option<(Vec<(Word, Word)>, KeySet)> {
        let u = self.index[a];
        let v = self.index[b];

        assert_ne!(max_len, Some(0));

        let mut path_keys = KeySet::default();
        path_keys += &self.vertices[u].1;
        let mut path = vec![self.vertices[u].0];

        let mut u = match self.next[(u, v)] {
            Some(u) => u,
            None => {
                return None;
            }
        };

        path_keys += &self.vertices[u].1;
        path.push(self.vertices[u].0);

        loop {
            match (path.len(), max_len) {
                (x, Some(y)) if x >= y => {
                    return None;
                },
                _ => {},
            }

            u = match self.next[(u, v)] {
                Some(u) => u,
                None => {
                    panic!("no subpath from {:?} to {:?}", self.vertices[u], self.vertices[v]);
                }
            };
            path_keys += &self.vertices[u].1;
            if !keys.can_open(&path_keys) {
                return None;
            }
            path.push(self.vertices[u].0);

            if u == v {
                break;
            }
        }

        Some((path, path_keys))
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
struct KeySet(u64);

impl std::ops::Add<&Tile> for KeySet {
    type Output = KeySet;

    fn add(self, key: &Tile) -> KeySet {
        let bit = Self::to_bit(key).unwrap_or_else(|nk| panic!("add given non-key: {}", nk));

        if self.0 & bit == 0 {
            KeySet(self.0 | bit)
        } else {
            panic!("Key already in keyset: {:?}", key)
        }
    }
}

impl std::ops::AddAssign<&Tile> for KeySet {
    fn add_assign(&mut self, rhs: &Tile) {
        let bit = match Self::to_bit(rhs) {
            Ok(bit) => bit,
            Err(_) => return,
        };

        if self.0 & bit == 0 {
            self.0 |= bit;
        } else {
            panic!("Key already in keyset: {:?}", rhs);
        }
    }
}

impl std::ops::Sub<&KeySet> for KeySet {
    type Output = KeySet;

    fn sub(self, rhs: &KeySet) -> KeySet {
        KeySet(self.0 & !rhs.0)
    }
}

impl KeySet {
    fn to_bit(key: &Tile) -> Result<u64, &Tile> {
        match key {
            &Tile::Key(ch) => Ok(1 << (ch as u8 - b'a')),
            &Tile::Door(ch) => Ok(1 << ch as u8 - b'A' + b'z' - b'a' + 1),
            x => Err(x),
        }
    }

    fn can_open(&self, doors: &KeySet) -> bool {
        let only_keys = self.only_keys().0;
        let only_doors = doors.only_doors().0;

        let shifted = only_doors >> 26;
        // println!("init({:?}, {:?}): only({:08x} and {:08x})", self, doors, only_keys, only_doors >> 26);
        only_keys & shifted == shifted
    }

    fn contains(&self, key: &Tile) -> bool {
        match Self::to_bit(key) {
            Ok(bit) => self.0 & bit == 0,
            Err(e) => panic!("contains given non-key: {}", e),
        }
    }

    fn subset_of(&self, rhs: &KeySet) -> bool {
        (self.0 & rhs.0) == self.0
    }

    fn key_count(&self) -> usize {
        let keys = 0xff_ffff;
        (self.0 & keys).count_ones() as usize
    }

    fn only_keys(&self) -> Self {
        let keys = 0xff_ffff;
        KeySet(self.0 & keys)
    }

    fn only_doors(&self) -> Self {
        let doors = !0x0000_0000_00ff_ffff;
        KeySet(self.0 & doors)
    }
}

#[test]
fn keyset_opening() {
    let mut ks = KeySet::default();

    ks += &Tile::Key('a');

    let ks = ks;

    let doors = KeySet::default() + &Tile::Door('A');

    assert!(!KeySet::default().can_open(&doors));
    assert!(ks.can_open(&doors));

    assert!(ks.subset_of(&(ks + &Tile::Door('A'))));
    assert!(ks.subset_of(&(ks + &Tile::Key('b'))));
    assert!(!ks.subset_of(&(KeySet::default() + &Tile::Key('b'))));
}

impl fmt::Debug for KeySet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 0 {
            return write!(fmt, "0");
        }

        write!(fmt, "\"")?;

        for ch in b'a'..=b'z' {
            let bit = 1 << (ch - b'a');

            if self.0 & bit != 0 {
                write!(fmt, "{}", ch as char)?;
            }
        }
        for ch in b'A'..=b'Z' {
            let bit = 1 << (ch - b'A' + b'z' - b'a' + 1);
            if self.0 & bit != 0 {
                write!(fmt, "{}", ch as char)?;
            }
        }

        write!(fmt, "\"")
    }
}

/// We can explain our travers with this?
struct Waypoint((Word, Word), Tile);

struct Map {
    gd: GameDisplay<Tile>,
    poi: HashMap<Tile, (Word, Word)>,
    initial_position: (Word, Word),
}

impl Map {
    fn frontier<'a>(&'a self, pos: (Word, Word)) -> impl Iterator<Item = ((Word, Word), Tile)> + 'a {
        match self.gd.get(&pos) {
            Some(&Tile::Wall) | None => Either::Left(std::iter::empty()),
            Some(t) => Either::Right(self.frontier_where(pos, t))
        }
    }

    fn frontier_where<'a>(&'a self, pos: (Word, Word), t: &'a Tile) -> impl Iterator<Item = ((Word, Word), Tile)> + 'a {
        let deltas = &[
            ( 0, -1), // up
            ( 1,  0), // right
            ( 0,  1), // down
            (-1,  0) // left
        ];

        match t {
            &Tile::Wall => return Either::Left(std::iter::empty()),
            _ => {}
        }

        Either::Right(deltas.iter()
            .map(move |d| (pos.0 + d.0, pos.1 + d.1))
            .filter_map(move |p| {
                self.gd.get(&p).and_then(|t| match t {
                    &Tile::Wall => None,
                    _ => Some((p, t.clone()))
                })
            }))
    }

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

impl Tile {
    fn is_key(&self) -> bool {
        match *self {
            Tile::Key(_) => true,
            _ => false,
        }
    }
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
        let (mut gd, mut poi) = s
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

        let initial_position = poi.remove(&Tile::Portal).expect("No portal '@' found on the map");
        gd.insert(&initial_position, Tile::Empty);

        Ok(Map {
            gd,
            poi,
            initial_position,
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

    assert_eq!(s.trim().replace("@", "."), format!("{}", m).trim());
    assert_eq!((5, 1), m.initial_position);
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

    let mut m = Map::from_str(s).unwrap();

    assert_eq!(steps_to_collect_all_keys(&mut m), 8);
}

#[test]
fn first_multiple_choice() {
    let s = "\
########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################";

    let mut m = Map::from_str(s).unwrap();

    assert_eq!(steps_to_collect_all_keys(&mut m), 86);
}

#[test]
fn second_multiple_choice() {
    let s = "\
########################
#...............b.C.D.f#
#.######################
#.....@.a.B.c.d.A.e.F.g#
########################";

    let mut m = Map::from_str(s).unwrap();

    assert_eq!(steps_to_collect_all_keys(&mut m), 132);
}
