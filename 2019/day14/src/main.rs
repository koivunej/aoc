use std::collections::HashMap;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    println!("stage1: {}", stage1(BufReadWrapper(locked)));
}

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

fn stage1<R: HorribleLineAbstraction>(br: R) -> usize {

    let mut interned = HashMap::new();
    let mut produced = HashMap::new();
    let mut used: Vec<Option<usize>> = Vec::new();
    let mut leftovers: Vec<usize> = Vec::new();

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

        let (name, product) = parse_ingredient(&mut interned, rhs);

        let product = Production {
            id: product.id,
            amount: product.amount,
            required
        };

        if name == "FUEL" {
            used.resize(used.len().max(product.id + 1), None);
            used[product.id] = Some(1);
            product.explode(&mut used, &mut leftovers);
        } else {
            produced.insert(product.id, product);
        }
    });

    used.resize(interned.len() + 1, None);
    leftovers.resize(interned.len() + 1, 0);

    let mut round_productions = Vec::new();

    let ore_at = interned["ORE"];

    /*let _named = {
        let mut named = HashMap::new();
        for (k, v) in interned {
            named.insert(v, k);
        }
        named
    };*/

    loop {
        for (k, v) in &interned {
            println!("{:>6?} {:>4}", used[*v], k);
        }

        let productions = used.iter()
            .enumerate()
            .filter(|(_, c)| match c {
                // ones we do not yet know ignore for now
                Some(0) => false,
                None => false,
                _ => true,
            })
            // fetch recipe
            .filter_map(|(i, _)| produced.get(&i));

        round_productions.clear();
        round_productions.extend(productions);

        if round_productions.is_empty() {
            break;
        }

        // make sure to only run the next batch of recipes which already have their dependencies
        // this causes running in "waves" and prevents the ORE cascading too fast through a short
        // path.
        let all_new = round_productions.iter()
            .all(|p| p.all_are_new(&used));

        if !all_new {
            round_productions.retain(|p| !p.all_are_new(&used));
        }

        println!("productions : {:?}", round_productions.as_slice());
        println!("used        : {:?}", used.as_slice());
        println!("leftovers   : {:?}", leftovers.as_slice());

        for p in round_productions.drain(..) {
            // explode will set it's own coefficient to zero which will make us filter it out in
            // the next run
            p.explode(&mut used, &mut leftovers);
        }

        /*for c in coefficients.iter_mut() {
            let before = c.clone();
            // perhaps this will save up some garbage or waste better than the ceil div
            // *c = c.take().map(|q| q.ceil());
        }*/
    }

    /*
    for (k, v) in &interned {
        println!("{:>6?} {:>4}", coefficients[*v], k);
    }
    */

    // there should only be the ORE coefficient
    let zero = 0;
    assert_eq!(used.iter().filter(|c| c.unwrap_or(zero) > zero).count(), 1);

    //assert!(coefficients[ore_at].unwrap().is_integer());
    used[ore_at].unwrap() as usize
}

#[derive(Clone, Debug)]
struct Production {
    id: usize,
    amount: usize,
    required: Vec<Ingredient>,
}

impl Production {
    fn explode(&self, r: &mut Vec<Option<usize>>, l: &mut Vec<usize>) {
        assert_eq!(r.len(), l.len());
        // make sure our index is valid
        let our_need = r[self.id].unwrap();

        let times = if our_need > 1 {
            // ceiling div
            1 + ((our_need - 1) / self.amount)
        } else {
            1
        };

        println!("{}: our_need = used({}) * {} = {} and times = {}", self.id, r[self.id].unwrap(), self.amount, our_need, times);

        for req in &self.required {
            // FIXME: this will probably still need to be done with those reserved and used?
            let would_reserve = times * req.amount;

            let our_need = would_reserve;

            // FIXME: the leftovers are not handled!
            //println!("delta = {} * {} * {} == {}", our_c, times, req.amount, delta);

            println!("{}: processing req({}) {} and {}", self.id, req.id, r.len(), l.len());

            match (r.get_mut(req.id).unwrap(), l.get_mut(req.id).unwrap()) {
                (Some(ref mut reserved), ref mut leftovers) => {
                    if our_need < **leftovers {
                        println!("req({}) using {}/{} leftovers", req.id, our_need, **leftovers);
                        **leftovers -= our_need;
                    } else {
                        let total_need = our_need + *reserved - **leftovers;
                        println!("req({}) total need is {} + {} - {} = {}", req.id, our_need, *reserved, **leftovers, total_need);
                        println!("req({}) reserved = {} + {} * {} = {} + {}", req.id, *reserved, times, req.amount, *reserved, times * req.amount);
                        *reserved += times * req.amount;
                        println!("req({}) leftovers = {} - {}", req.id, reserved, total_need);
                        **leftovers = *reserved - total_need;
                    }
                },
                (ref mut x, ref mut leftovers) => {
                    **x = Some(would_reserve);
                    println!("req({}): {} - {}", req.id, would_reserve, our_need);
                    **leftovers = would_reserve - our_need;
                }
            };
        }

        r[self.id] = Some(0);
        l[self.id] = times * self.amount - our_need;
    }

    fn all_are_new<V>(&self, c: &Vec<Option<V>>) -> bool {
        for req in &self.required {
            match c.get(req.id) {
                Some(Some(_)) => return false,
                _ => {},
            }
        }
        return true;
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
    reserved.resize(6, 0);

    used[0] = Some(1);

    p.explode(&mut used, &mut reserved);

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

    p.explode(&mut used, &mut reserved);

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

    assert_eq!(31, stage1(wrap_into_bufreader(input)));
}

#[test]
#[ignore]
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

    assert_eq!(165, stage1(wrap_into_bufreader(input)));
}

#[test]
#[ignore]
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

    assert_eq!(13312, stage1(wrap_into_bufreader(input)));
}

#[test]
#[ignore]
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

    assert_eq!(180697, stage1(wrap_into_bufreader(input)));
}

#[test]
#[ignore]
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

        assert_eq!(2210736, stage1(lines.as_slice()));
    }
}

#[test]
#[ignore]
fn stage1_does_not_show_the_issue() {
    let input = "\
1 ORE => 3 A
1 ORE => 2 B
2 A, 3 B => 5 C
3 A, 7 B => 5 D
7 A, 11 C, 6 D => 1 FUEL
";
    // 1 FUEL:   7, 0, 11
    //         7+6, 9, 0
    //          13, 9
    //      -------------
    //           5  5 ==> 10

    let lines = input.lines().map(String::from).collect::<Vec<_>>();
    // not sure of this 19
    assert_eq!(19, stage1(lines.as_slice()));
}

#[cfg(test)]
fn wrap_into_bufreader(s: &[u8]) -> BufReadWrapper<std::io::BufReader<std::io::Cursor<&[u8]>>> {
    BufReadWrapper(std::io::BufReader::new(std::io::Cursor::new(s)))
}
