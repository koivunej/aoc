use std::collections::HashMap;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    let ctx = Context::parse(BufReadWrapper(locked));
    let ore_for_one_fuel = stage1(&ctx);

    println!("stage1: {}", ore_for_one_fuel);
    println!("stage2: {} ORE", fuel_for_ore(&ctx, 1_000_000_000_000));
}

/// Wanted to do randomized testing so ended up creating this.
trait HorribleLineAbstraction {
    fn for_lines<F: FnMut(Option<&str>)>(self, f: F);
}

struct BufReadWrapper<B>(B);

impl<B: BufRead> HorribleLineAbstraction for BufReadWrapper<B> {
    fn for_lines<F: FnMut(Option<&str>)>(mut self, mut f: F) {
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if self.0.read_line(&mut buffer).unwrap() == 0 {
                f(None);
                break;
            }

            f(Some(buffer.as_str()));
        }
    }
}

impl<'a, T: AsRef<str> + 'a> HorribleLineAbstraction for &'a [T] {
    fn for_lines<F: FnMut(Option<&str>)>(self, mut f: F) {
        for line in self {
            f(Some(line.as_ref()));
        }
        f(None);
    }
}

struct Context {
    interned: HashMap<String, usize>,
    produced: HashMap<usize, Production>,
}

impl Context {
    fn parse<R: HorribleLineAbstraction>(br: R) -> Self {
        let mut interned = HashMap::new();
        let mut produced = HashMap::new();

        br.for_lines(|maybe_line| {
            let line = match maybe_line {
                Some(line) => line,
                None => return,
            };

            let mut top = line.trim()
                .split(" => ");

            let lhs = top.next().unwrap();
            let rhs = top.next().unwrap();

            let required = lhs.split(", ")
                .map(|part| parse_ingredient(&mut interned, part).1)
                .collect::<Vec<_>>();

            let (_, product) = parse_ingredient(&mut interned, rhs);

            let product = Production {
                id: product.id,
                amount: product.amount,
                required
            };

            produced.insert(product.id, product);
        });

        assert_eq!(interned.len(), produced.len() + 1);

        Context {
            interned,
            produced,
        }
    }

    fn len(&self) -> usize {
        self.interned.len()
    }
}

fn stage1(ctx: &Context) -> usize {
    ore_for_fuel(ctx, 1)
}

fn ore_for_fuel(ctx: &Context, fuel: usize) -> usize {

    let mut used: Vec<Option<usize>> = vec![None; ctx.len() + 1];
    let mut leftovers: Vec<usize> = vec![0; ctx.len() + 1];
    let mut totals: Vec<usize> = vec![0; ctx.len() + 1];

    let fuel_at = ctx.interned["FUEL"];
    {
        used[fuel_at] = Some(fuel);
        ctx.produced[&fuel_at].explode(&mut used, &mut leftovers, &mut totals);
    }

    let mut round_productions = Vec::new();

    let ore_at = ctx.interned["ORE"];

    loop {
        let productions = used.iter()
            .enumerate()
            .filter(|(_, c)| match c {
                // ones we do not yet know ignore for now
                Some(0) => false,
                None => false,
                _ => true,
            })
            .filter(|(i, _)| *i != fuel_at)
            // fetch recipe
            .filter_map(|(i, _)| ctx.produced.get(&i));

        round_productions.clear();
        round_productions.extend(productions);

        if round_productions.is_empty() {
            break;
        }

        for p in round_productions.drain(..) {
            // explode will set it's own coefficient to zero which will make us filter it out in
            // the next run
            p.explode(&mut used, &mut leftovers, &mut totals);
            // cannot add to processed ... which is interesting, perhaps few need to be processed
            // multiple times?
        }
    }

    assert_eq!(
        used.iter().filter(|c| c.unwrap_or(0) > 0).count(),
        1,
        "there should only be the ore coefficient non-zero");

    used[ore_at].unwrap() as usize
}

fn fuel_for_ore(ctx: &Context, ore: usize) -> usize {

    let mut min = 1;
    let mut max = ore;

    let mut prev = 1;
    loop {
        let mid = (min + max) / 2;
        let amount = ore_for_fuel(&ctx, mid);

        if amount < ore {
            min = mid;
        } else if amount > ore {
            max = mid;
        }

        if prev == amount {
            return mid;
        }

        if min == max {
            return mid;
        }

        prev = amount;
    }
}

#[derive(Clone, Debug)]
struct Production {
    id: usize,
    amount: usize,
    required: Vec<Ingredient>,
}

impl Production {
    fn explode(&self, r: &mut Vec<Option<usize>>, l: &mut Vec<usize>, t: &mut Vec<usize>) {
        // make sure our index is valid
        let our_need = r[self.id].unwrap() - l[self.id];

        let times = if our_need > 1 {
            // ceiling div
            1 + ((our_need - 1) / self.amount)
        } else {
            1
        };

        for req in &self.required {
            // FIXME: this will probably still need to be done with those reserved and used?
            let would_reserve = times * req.amount;

            let our_need = would_reserve;

            // println!("{}: processing req({}) {} and {}", self.id, req.id, r.len(), l.len());

            match (r.get_mut(req.id).unwrap(), l.get_mut(req.id).unwrap()) {
                (Some(ref mut reserved), ref mut leftovers) => {
                    if our_need <= **leftovers {
                        //println!("req({}) using {}/{} leftovers", req.id, our_need, **leftovers);
                        **leftovers -= our_need;
                    } else {
                        let total_need = our_need + *reserved - **leftovers;
                        *reserved += times * req.amount;
                        **leftovers = *reserved - total_need;
                    }
                },
                (ref mut x, ref mut leftovers) => {
                    **x = Some(would_reserve);
                    **leftovers = would_reserve - our_need;
                }
            };
        }

        t[self.id] = r[self.id].unwrap();
        r[self.id] = Some(0);
        l[self.id] = times * self.amount - our_need;
    }
}

#[test]
fn explode_first() {
    // 7 A, 11 C, 6 D => 1 FUEL

    let p = Production {
        id: 0,
        amount: 1,
        required: vec![
            Ingredient { id: 1, amount: 7, },
            Ingredient { id: 4, amount: 11, },
            Ingredient { id: 5, amount: 6, },
        ]
    };

    let mut used = Vec::new();
    let mut reserved = Vec::new();

    used.resize(6, None);
    reserved.resize(used.len(), 0);

    used[0] = Some(1);

    p.explode(&mut used, &mut reserved, &mut vec![0; 6]);

    assert_eq!((used[0], reserved[0]), (Some(0),  0), "FUEL");
    assert_eq!((used[1], reserved[1]), (Some(7),  0), "A");
    assert_eq!((used[4], reserved[4]), (Some(11), 0), "C");
    assert_eq!((used[5], reserved[5]), (Some(6),  0), "D");
}

#[test]
fn explode_second() {
    // 3 A, 7 B => 5 D
    //   ^    ^      ^
    //   1    2      4

    let p = Production {
        id: 4,
        amount: 5,
        required: vec![
            Ingredient { id: 1, amount: 3, },
            Ingredient { id: 2, amount: 7, },
        ]
    };

    let mut used = vec![Some(0), Some(7), None, Some(11), Some(6)];
    let mut reserved = vec![0, 0, 0, 0, 0];

    assert_eq!(used.len(), reserved.len());

    p.explode(&mut used, &mut reserved, &mut vec![0; 5]);

    // to have 6D we need to build 2*5D which needs 2*3A + 2*7B
    // in the end we are left with 2*5D - 6D = 4D

    assert_eq!((used[0], reserved[0]), (Some(0),  0), "FUEL");
    assert_eq!((used[1], reserved[1]), (Some(7 + 6), 0), "A");
    assert_eq!((used[2], reserved[2]), (Some(2 * 7), 0), "B");
    assert_eq!((used[3], reserved[3]), (Some(11), 0), "C");
    assert_eq!((used[4], reserved[4]), (Some(0),  4), "D");
}

fn parse_ingredient<'a>(ingredients: &mut HashMap<String, usize>, s: &'a str) -> (&'a str, Ingredient) {
    let mut split = s.split_whitespace();
    let amount = split.next().unwrap().parse::<usize>().unwrap();
    let name = split.next().unwrap();
    let id = ingredients.len();
    let id = *ingredients.entry(String::from(name)).or_insert(id);
    (name, Ingredient { id, amount })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Ingredient {
    id: usize,
    amount: usize,
}

#[test]
fn stage1_example0() {
    let input = b"\
10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL
";

    let ctx = Context::parse(wrap_into_bufreader(input));
    assert_eq!(31, stage1(&ctx));

//              x ORE
//              |
//          /--/ \--\
//         |         |
//       10 A       1 B
// (28 A)  |         |
//         |   7 A   | 1 B
//         +---------+
//         |         |
//         |        1 C
//         |   7 A   |
//         +---------+
//         |         |
//         |        1 D
//         |   7 A   |
//         +---------+
//         |         |
//         |        1 E
//         |   7 A   |
//         \---------+
//                   |
//                1 FUEL

    // 10 ORE => 10 A ---> 0
    // 11 ORE => 11 A ---> 1 FUEL + 10 A + 1 B + 0 ORE
    // 21 ORE => 21 A ---> 1 FUEL + 20 A + 1 B + 0 ORE
    // 31 ORE => 31 A ---> 1 FUEL +  2 A + 0 B + 0 ORE
    // 41 ORE => 41 A ---> 1 FUEL + 12 A + 1 B + 0 ORE
    // 51 ORE => 51 A ---> 1 FUEL + 22 A + 1 B + 0 ORE
    // 53 ORE => 53 A ---> 1 FUEL + 22 A + 1 B + 2 ORE
    // 55 ORE => 55 A ---> 1 FUEL + 22 A + 1 B + 4 ORE
    // 57 ORE => 57 A ---> 1 FUEL + 22 A + 1 B + 6 ORE
    // 59 ORE => 59 A ---> 1 FUEL + 22 A + 1 B + 8 ORE
    // 60 ORE => 60 A ---> 2 FUEL +  4 A + 0 B + 0 ORE
    // 61 ORE => 61 A ---> 2 FUEL +  4 A + 1 B + 0 ORE
    for ore in 31..62 {
        assert_eq!(1, fuel_for_ore(&ctx, ore), "{} ORE", ore);
    }

    for ore in 62..93 {
        assert_eq!(2, fuel_for_ore(&ctx, ore), "{} ORE", ore);
    }
}

#[test]
fn stage1_example1() {
    let input = b"\
9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL
";

    assert_eq!(165, stage1(&Context::parse(wrap_into_bufreader(input))));
}

#[test]
fn stage1_example2() {
    let input = b"\
157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
";
    let ctx = Context::parse(wrap_into_bufreader(input));
    assert_eq!(13312, stage1(&ctx));
    assert_eq!(82892753, fuel_for_ore(&ctx, 1_000_000_000_000));
}

#[test]
fn stage1_example3() {
    let input = b"\
2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF
";

    let ctx = Context::parse(wrap_into_bufreader(input));
    assert_eq!(180697, stage1(&ctx));
    assert_eq!(5586022, fuel_for_ore(&ctx, 1_000_000_000_000));
}

#[test]
fn stage1_example4() {
    use rand::seq::SliceRandom;
    let input = "\
171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX
";

    let mut lines = input.lines().map(String::from).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        lines.shuffle(&mut rng);
        assert_eq!(2210736, stage1(&Context::parse(lines.as_slice())));
    }

    assert_eq!(460664, fuel_for_ore(&Context::parse(lines.as_slice()), 1_000_000_000_000));
}

#[test]
fn stage1_does_not_show_the_issue() {
    let input = "\
1 ORE => 3 A
1 ORE => 2 B
2 A, 3 B => 5 C
3 A, 7 B => 5 D
7 A, 11 C, 6 D => 1 FUEL
";
    let lines = input.lines().map(String::from).collect::<Vec<_>>();
    // not sure of this 19, seems to be :)
    assert_eq!(19, stage1(&Context::parse(lines.as_slice())));
}

#[cfg(test)]
fn wrap_into_bufreader(s: &[u8]) -> BufReadWrapper<std::io::BufReader<std::io::Cursor<&[u8]>>> {
    BufReadWrapper(std::io::BufReader::new(std::io::Cursor::new(s)))
}

#[test]
fn full_stage1() {
    let ctx = Context::parse(BufReadWrapper(std::io::BufReader::new(std::fs::File::open("input").unwrap())));
    assert_eq!(stage1(&ctx), 1967319);
}

#[test]
fn full_stage2() {
    let ctx = Context::parse(BufReadWrapper(std::io::BufReader::new(std::fs::File::open("input").unwrap())));
    assert_eq!(fuel_for_ore(&ctx, 1_000_000_000_000), 1122036);
}
