#[macro_use]
extern crate itertools;

fn main() {
    let input = "#.##.##.
.##..#..
....#..#
.##....#
#..##...
.###..#.
..#.#..#
.....#..";

    let part_one = {
        let mut world = three_dimensional::World::default();
        let (w, h) = three_dimensional::parse_slice(input, 0, (0, 0), &mut world);
        assert_eq!((w, h), (8, 8));
        three_dimensional::n_gol_rounds(&mut world, 6);

        world.len()
    };
    println!("{}", part_one);

    let part_two = {
        let mut world = four_dimensional::World::default();
        let (w, h) = four_dimensional::parse_slice(input, (0, 0, 0, 0), &mut world);
        assert_eq!((w, h), (8, 8));
        four_dimensional::n_gol_rounds(&mut world, 6);

        world.len()
    };

    println!("{}", part_two);

    assert_eq!(part_one, 273);
    assert_eq!(part_two, 1504);
}

// now ... how to macro this?
mod three_dimensional {
    use std::collections::{HashMap, HashSet};
    use std::convert::TryInto;

    pub fn n_gol_rounds(world: &mut World, n: usize) {
        let mut first = world.to_owned();
        let mut second = world;
        let mut scratch = HashMap::new();

        // round one
        for _ in 0..n {
            gol_round(&first, &mut second, &mut scratch);

            std::mem::swap(&mut first, &mut second);
        }

        if n % 2 == 0 {
            std::mem::swap(&mut first, &mut second);
        }
    }

    fn gol_round(old: &World, new: &mut World, discovered_neighbours: &mut HashMap<Point, usize>) {
        new.clear();
        discovered_neighbours.clear();

        let mut activated_new = 0;
        let mut kept_active = 0;
        let mut deactivated = 0;

        for point in &old.inner {
            let mut current_neighbours = 0;
            for neighbour in neighbours(point) {
                let their_neighbours = discovered_neighbours.entry(neighbour).or_insert(0);
                // one for being the neighbour of the current
                *their_neighbours += 1;
                if old.contains(&neighbour) {
                    current_neighbours += 1;
                }
            }

            if current_neighbours == 2 || current_neighbours == 3 {
                new.insert(*point);
                kept_active += 1;
            } else {
                deactivated += 1;
            }
        }

        for (point, count) in discovered_neighbours.drain() {
            // how to process only those we haven't already processed from the old world?
            if old.contains(&point) {
                continue;
            }

            if count == 3 {
                new.insert(point);
                activated_new += 1;
            }
        }

        drop(activated_new);
        drop(kept_active);
        drop(deactivated);
    }

    fn neighbours(p: &Point) -> impl Iterator<Item = Point> {
        let xs = (p.0 - 1)..=(p.0 + 1);
        let ys = (p.1 - 1)..=(p.1 + 1);
        let zs = (p.2 - 1)..=(p.2 + 1);

        let p = *p;

        itertools::iproduct!(xs, ys, zs).filter(move |&point| point != p)
    }

    type Coord = i16;

    type Point = (Coord, Coord, Coord);

    type WorldState = HashSet<Point>;

    #[derive(Clone)]
    pub struct World {
        inner: WorldState,
        minmaxes: [(i16, i16); 3],
    }

    impl Default for World {
        fn default() -> Self {
            World {
                inner: Default::default(),
                minmaxes: [(i16::MAX, i16::MIN); 3],
            }
        }
    }

    impl World {
        pub fn len(&self) -> usize {
            self.inner.len()
        }

        fn insert(&mut self, p: Point) -> bool {
            if self.inner.insert(p) {
                self.minmaxes.iter_mut().zip(&[p.0, p.1, p.2]).for_each(
                    |((min, max), coord): (&mut (_, _), &i16)| {
                        *min = *(&*min).min(coord);
                        *max = *(&*max).max(coord);
                    },
                );
                true
            } else {
                false
            }
        }

        fn contains(&self, p: &Point) -> bool {
            self.inner.contains(p)
        }

        fn clear(&mut self) {
            self.inner.clear();
            // intentionally not clearing out the minmaxes
        }

        /*
        fn slice_2d<'a>(
            &'a self,
            z: Coord,
            (w, h): (usize, usize),
        ) -> impl Iterator<Item = Point> + 'a {
            let xs = self.min_max_range(0);
            let ys = self.min_max_range(1);

            let w: Coord = w.try_into().unwrap();
            let h: Coord = h.try_into().unwrap();

            assert!(
                xs.end() - xs.start() + 1 <= w,
                "range wont fit: {:?} vs. width {}",
                xs,
                w
            );
            assert!(
                ys.end() - ys.start() + 1 <= h,
                "range wont fit: {:?} vs. width {}",
                ys,
                h
            );

            assert_eq!(w % 2, 1);
            assert_eq!(h % 2, 1);

            let xs = (-w / 2)..=(w / 2);
            let ys = (-h / 2)..=(h / 2);

            itertools::iproduct!(xs, ys)
                .map(move |(x, y)| (x, y, z))
                .filter(move |p| self.inner.contains(p))
        }

        fn min_max_range(&self, axis: usize) -> RangeInclusive<Coord> {
            self.minmaxes[axis].0..=self.minmaxes[axis].1
        }*/
    }

    /// Returns the width and height of the slice parsed
    pub fn parse_slice(s: &str, z: i16, start: (Coord, Coord), target: &mut World) -> (i16, i16) {
        let mut w = None;

        let mut y = start.1;

        for line in s.lines() {
            if let Some(w) = w {
                assert_eq!(line.len(), w);
            } else {
                w = Some(line.len());
                assert_eq!(line.len(), line.trim().len());
            }

            line.as_bytes()
                .iter()
                .inspect(|&&ch| assert!(ch == b'#' || ch == b'.'))
                .zip(start.0..)
                .filter_map(|(&ch, x)| if ch == b'#' { Some((x, y, z)) } else { None })
                .for_each(|p| assert!(target.insert(p)));

            y += 1;
        }

        (w.unwrap().try_into().unwrap(), y - start.1)
    }

    #[test]
    fn first_example() {
        let mut world = World::default();

        let (w, h) = parse_slice(".#.\n..#\n###", 0, (-1, -1), &mut world);
        assert_eq!((w, h), (3, 3));

        let mut first = world.clone();
        let mut second = world;
        let mut scratch = HashMap::new();

        // round one
        for _ in 1..=6 {
            gol_round(&first, &mut second, &mut scratch);

            std::mem::swap(&mut first, &mut second);
        }

        // initially got 155; there seems to be zero kept active at all six
        assert_eq!(first.inner.len(), 112);
    }
}

mod four_dimensional {
    use std::collections::{HashMap, HashSet};
    use std::convert::TryInto;

    pub fn n_gol_rounds(world: &mut World, n: usize) {
        let mut first = world.to_owned();
        let mut second = world;
        let mut scratch = HashMap::new();

        // round one
        for _ in 0..n {
            gol_round(&first, &mut second, &mut scratch);

            std::mem::swap(&mut first, &mut second);
        }

        if n % 2 == 0 {
            std::mem::swap(&mut first, &mut second);
        }
    }

    fn gol_round(old: &World, new: &mut World, discovered_neighbours: &mut HashMap<Point, usize>) {
        new.clear();
        discovered_neighbours.clear();

        let mut activated_new = 0;
        let mut kept_active = 0;
        let mut deactivated = 0;

        for point in &old.inner {
            let mut current_neighbours = 0;
            for neighbour in neighbours(point) {
                let their_neighbours = discovered_neighbours.entry(neighbour).or_insert(0);
                // one for being the neighbour of the current
                *their_neighbours += 1;
                if old.contains(&neighbour) {
                    current_neighbours += 1;
                }
            }

            if current_neighbours == 2 || current_neighbours == 3 {
                new.insert(*point);
                kept_active += 1;
            } else {
                deactivated += 1;
            }
        }

        for (point, count) in discovered_neighbours.drain() {
            // how to process only those we haven't already processed from the old world?
            if old.contains(&point) {
                continue;
            }

            if count == 3 {
                new.insert(point);
                activated_new += 1;
            }
        }

        drop(activated_new);
        drop(kept_active);
        drop(deactivated);
    }

    fn neighbours(p: &Point) -> impl Iterator<Item = Point> {
        let xs = (p.0 - 1)..=(p.0 + 1);
        let ys = (p.1 - 1)..=(p.1 + 1);
        let zs = (p.2 - 1)..=(p.2 + 1);
        let ws = (p.3 - 1)..=(p.3 + 1);

        let p = *p;

        itertools::iproduct!(xs, ys, zs, ws).filter(move |&point| point != p)
    }

    type Coord = i16;

    type Point = (Coord, Coord, Coord, Coord);

    type WorldState = HashSet<Point>;

    #[derive(Clone)]
    pub struct World {
        inner: WorldState,
        minmaxes: [(i16, i16); 4],
    }

    impl Default for World {
        fn default() -> Self {
            World {
                inner: Default::default(),
                minmaxes: [(Coord::MAX, Coord::MIN); 4],
            }
        }
    }

    impl World {
        pub fn len(&self) -> usize {
            self.inner.len()
        }

        fn insert(&mut self, p: Point) -> bool {
            if self.inner.insert(p) {
                self.minmaxes.iter_mut().zip(&[p.0, p.1, p.2]).for_each(
                    |((min, max), coord): (&mut (_, _), &i16)| {
                        *min = *(&*min).min(coord);
                        *max = *(&*max).max(coord);
                    },
                );
                true
            } else {
                false
            }
        }

        fn contains(&self, p: &Point) -> bool {
            self.inner.contains(p)
        }

        fn clear(&mut self) {
            self.inner.clear();
            // intentionally not clearing out the minmaxes
        }

        /*
        fn slice_2d<'a>(
            &'a self,
            z: Coord,
            (w, h): (usize, usize),
        ) -> impl Iterator<Item = Point> + 'a {
            let xs = self.min_max_range(0);
            let ys = self.min_max_range(1);

            let w: Coord = w.try_into().unwrap();
            let h: Coord = h.try_into().unwrap();

            assert!(
                xs.end() - xs.start() + 1 <= w,
                "range wont fit: {:?} vs. width {}",
                xs,
                w
            );
            assert!(
                ys.end() - ys.start() + 1 <= h,
                "range wont fit: {:?} vs. width {}",
                ys,
                h
            );

            assert_eq!(w % 2, 1);
            assert_eq!(h % 2, 1);

            let xs = (-w / 2)..=(w / 2);
            let ys = (-h / 2)..=(h / 2);

            itertools::iproduct!(xs, ys)
                .map(move |(x, y)| (x, y, z))
                .filter(move |p| self.inner.contains(p))
        }

        fn min_max_range(&self, axis: usize) -> RangeInclusive<Coord> {
            self.minmaxes[axis].0..=self.minmaxes[axis].1
        }*/
    }

    /// Returns the width and height of the slice parsed
    pub fn parse_slice(
        s: &str,
        start: (Coord, Coord, Coord, Coord),
        target: &mut World,
    ) -> (i16, i16) {
        let mut width = None;

        let mut y = start.1;
        let z = start.2;
        let w = start.3;

        for line in s.lines() {
            if let Some(width) = width {
                assert_eq!(line.len(), width);
            } else {
                width = Some(line.len());
                assert_eq!(line.len(), line.trim().len());
            }

            line.as_bytes()
                .iter()
                .inspect(|&&ch| assert!(ch == b'#' || ch == b'.'))
                .zip(start.0..)
                .filter_map(|(&ch, x)| if ch == b'#' { Some((x, y, z, w)) } else { None })
                .for_each(|p| assert!(target.insert(p)));

            y += 1;
        }

        (width.unwrap().try_into().unwrap(), y - start.1)
    }
}
