#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
struct Point(i64, i64);

impl std::ops::Add<&Point> for &Point {
    type Output = Point;

    fn add(self, other: &Point) -> Self::Output {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Self) -> Self::Output {
        &self + &other
    }
}

fn main() {
    use std::io::BufRead;
    use std::collections::HashMap;

    let mut inches = HashMap::new();

    let stdin = std::io::stdin();
    let mut locked = stdin.lock();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        let bytes = locked.read_line(&mut buffer)
            .expect("Failed to read line from stdin");

        if bytes == 0 {
            break;
        }

        // #int @ Left,Top: WidthxHeight
        let (_, corner, size) = parse_claim(buffer.trim());

        for y_off in 0..size.1 {
            for x_off in 0..size.0 {
                let p = corner + Point(x_off as i64, y_off as i64);
                *inches.entry(p).or_insert(0) += 1;
            }
        }
    }

    let part1 = inches.values()
        .filter(|&&c| c > 1)
        .count();

    println!("part1: {}", part1);

    assert_eq!(part1, 111485);
}

fn parse_claim(s: &str) -> (usize, Point, (u64, u64)) {
    use std::str::FromStr;
    use nom::{IResult, bytes::complete::{tag, take_while}, combinator::map_res};

    fn next_pair<'a, T: FromStr>(s: &'a str, sep: &str) -> IResult<&'a str, (T, T)> {
        let (s, x) = next_num::<T>(s)?;
        let (s, _) = tag(sep)(s)?;
        let (s, y) = next_num::<T>(s)?;
        Ok((s, (x, y)))
    }

    fn next_num<T: FromStr>(s: &str) -> IResult<&str, T> {
        map_res(
            take_while(|c: char| c.is_digit(10)),
            T::from_str
        )(s)
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
