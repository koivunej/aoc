#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
struct Point(i64, i64);

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Self) -> Self::Output {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Dispute {
    Undisputed,
    Disputed,
}

fn main() {
    use std::collections::{hash_map::Entry, HashMap};
    use std::io::BufRead;

    let mut inches = HashMap::new();
    // keep track of undisputed claims, which will initially contain all but as there arrive
    // competing claims in their region, the id will be removed
    let mut disputes = HashMap::new();

    let stdin = std::io::stdin();
    let mut locked = stdin.lock();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        let bytes = locked
            .read_line(&mut buffer)
            .expect("Failed to read line from stdin");

        if bytes == 0 {
            break;
        }

        // #int @ Left,Top: WidthxHeight
        let (id, corner, size) = parse_claim(buffer.trim());

        for y_off in 0..size.1 {
            for x_off in 0..size.0 {
                let p = corner + Point(x_off as i64, y_off as i64);
                match inches.entry(p) {
                    Entry::Vacant(ve) => {
                        ve.insert((id, 1));
                        disputes.entry(id).or_insert(Dispute::Undisputed);
                    }
                    Entry::Occupied(oe) if oe.get().0 == id => unreachable!("duplicate inch claim"),
                    Entry::Occupied(mut oe) => {
                        let (original_id, count) = oe.get_mut();
                        *count += 1;

                        // both the original and the new claim must now be marked as disputed
                        disputes.insert(*original_id, Dispute::Disputed);
                        disputes.insert(id, Dispute::Disputed);
                    }
                }
            }
        }
    }

    let part1 = inches.values().filter(|&&(_, c)| c > 1).count();

    println!("part1: {}", part1);

    let undisputed = disputes
        .into_iter()
        .filter(|(_, v)| *v == Dispute::Undisputed)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    let part2 = undisputed.iter().single();

    println!("part2: {:?}", part2);

    assert_eq!(part1, 111_485);
    assert_eq!(part2, Ok(&113));
}

trait IteratorExt {
    fn single<T>(self) -> Result<T, Option<(T, T)>>
    where
        Self: Iterator<Item = T>;
}

impl<Iter: Iterator> IteratorExt for Iter {
    fn single<T>(mut self) -> Result<T, Option<(T, T)>>
    where
        Self: Iterator<Item = T>,
    {
        let only = self.next();
        match only {
            Some(only) => {
                let next = self.next();

                if next.is_none() {
                    return Ok(only);
                }

                Err(Some((only, next.unwrap())))
            }
            None => Err(None),
        }
    }
}

fn parse_claim(s: &str) -> (usize, Point, (u64, u64)) {
    use nom::{
        bytes::complete::{tag, take_while},
        combinator::map_res,
        IResult,
    };
    use std::str::FromStr;

    fn next_pair<'a, T: FromStr>(s: &'a str, sep: &str) -> IResult<&'a str, (T, T)> {
        let (s, x) = next_num::<T>(s)?;
        let (s, _) = tag(sep)(s)?;
        let (s, y) = next_num::<T>(s)?;
        Ok((s, (x, y)))
    }

    fn next_num<T: FromStr>(s: &str) -> IResult<&str, T> {
        map_res(take_while(|c: char| c.is_digit(10)), T::from_str)(s)
    }

    fn inner(s: &str) -> IResult<&str, (usize, Point, (u64, u64))> {
        let (s, _) = tag("#")(s)?;
        let (s, id) = next_num::<usize>(s)?;
        let (s, _) = tag(" @ ")(s)?;
        let (s, (left, top)) = next_pair::<i64>(s, ",")?;
        let (s, _) = tag(": ")(s)?;
        let (s, size) = next_pair::<u64>(s, "x")?;

        Ok((s, (id, Point(left, top), size)))
    }

    match inner(s) {
        IResult::Ok(("", p)) => p,
        x => panic!("Unexpected for {:?}: {:?}", s, x),
    }
}
