use aoc2020::io::OnePerLine;
use std::collections::HashMap;
use std::convert::TryInto;

fn main() {
    let stdin = std::io::stdin();
    let (part_one, part_two) = process(stdin.lock());

    let part_one = part_one.iter().sum::<u64>();
    let part_two = part_two.values().sum::<u64>();

    println!("{}", part_one);
    println!("{}", part_two);

    // off by one shift here
    assert_ne!(part_one, 11_745_848_003_726);
    assert_eq!(part_one, 5_875_750_429_995);
    assert_ne!(part_two, 1_297_026_531_039, "low");
    assert_eq!(part_two, 5_272_149_590_143);
}

fn process<I: std::io::BufRead>(input: I) -> (Vec<u64>, HashMap<u64, u64>) {
    let mut one_per_line = OnePerLine::<_, Op>::new(input);

    let mut mask = None;
    let mut memory_1 = Vec::new();
    let mut memory_2 = HashMap::new();

    while let Some(op) = one_per_line.next() {
        let op = op.unwrap();

        match op {
            Op::Mask { or, and, floating } => mask = Some(Mask { or, and, floating }),
            Op::Mem(index, literal) => {
                let mask = mask.as_ref().unwrap();
                {
                    let index: usize = index.try_into().unwrap();
                    if memory_1.len() < index {
                        memory_1.resize(index + 1, 0);
                    }
                    memory_1[index] = (literal | mask.or) & mask.and;
                }

                for addr in mask.generate_write_addresses(index) {
                    memory_2.insert(addr, literal);
                }
            }
        }
    }

    (memory_1, memory_2)
}

#[derive(Debug, Clone, Copy)]
struct Mask {
    or: u64,
    and: u64,
    floating: u64,
}

impl Mask {
    fn generate_write_addresses(&self, addr: u64) -> impl Iterator<Item = u64> {
        let addr = addr | self.or;
        let init = addr & (!self.floating);

        let counter_len = 2u64.pow(self.floating.count_ones());

        let mut out = vec![];

        for ctr in 0..counter_len {
            let mut tmp = init;
            let mut index = 0;
            let mut test = 0;
            while index < 36 && test < 36 {
                if (self.floating >> test) & 1 == 1 {
                    // there has to be less wasteful version of this
                    tmp |= ((ctr as u64 >> index) & 1) << test;
                    index += 1;
                }
                test += 1;
            }
            out.push(tmp);
        }

        out.into_iter()
    }
}

#[derive(Debug)]
enum Op {
    Mask { or: u64, and: u64, floating: u64 },
    Mem(u64, u64),
}

impl std::str::FromStr for Op {
    type Err = Box<dyn std::error::Error + 'static>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(mask) = s.strip_prefix("mask = ") {
            let mask = mask.trim();
            assert_eq!(mask.len(), 36);

            let (or, and, floating) = mask
                .as_bytes()
                .iter()
                .enumerate()
                .map(|(idx, ch)| (36 - idx, ch))
                .fold((0u64, 0u64, 0u64), |acc, (shift, &zero_or_one)| {
                    let shifted = 1 << (shift - 1);
                    if zero_or_one == b'1' {
                        (acc.0 | shifted, acc.1, acc.2)
                    } else if zero_or_one == b'0' {
                        (acc.0, acc.1 | shifted, acc.2)
                    } else {
                        assert_eq!(zero_or_one, b'X');
                        (acc.0, acc.1, acc.2 | shifted)
                    }
                });

            Ok(Op::Mask {
                or,
                // FIXME: this is just !or?
                and: !and,
                floating,
            })
        } else if let Some(rest) = s.strip_prefix("mem[") {
            let closing = rest.find(']').expect("must be a closing bracket");

            let index = rest[..closing].parse::<u64>()?;

            let rest = rest[closing..].strip_prefix("] = ").unwrap().trim();

            let value = rest.parse::<u64>()?;

            Ok(Op::Mem(index, value))
        } else {
            unreachable!()
        }
    }
}

#[test]
fn first_example() {
    let input = "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
mem[8] = 11
mem[7] = 101
mem[8] = 0";

    let (memory_1, _) = process(std::io::BufReader::new(std::io::Cursor::new(input)));
    println!("{:#?}", memory_1);

    assert_eq!(memory_1.iter().sum::<u64>(), 165);
}

#[test]
fn second_example() {
    let input = "mask = 000000000000000000000000000000X1001X
mem[42] = 100
mask = 00000000000000000000000000000000X0XX
mem[26] = 1";

    let (_, memory_2) = process(std::io::BufReader::new(std::io::Cursor::new(input)));
    println!("{:#?}", memory_2);

    assert_eq!(memory_2.values().sum::<u64>(), 208);
}

#[test]
fn mask_writes_to() {
    let input = "mask = 000000000000000000000000000000X1001X
mem[42] = 100
mask = 00000000000000000000000000000000X0XX
mem[26] = 1";

    let mut input = OnePerLine::<_, Op>::new(std::io::BufReader::new(std::io::Cursor::new(input)));

    match input.next().unwrap().unwrap() {
        Op::Mask { or, and, floating } => {
            assert_eq!(or, 0b10010);
            // ignore it for now, remove later
            // assert_eq!(!and, 0b01100, "\n{:036b}\n{:036b}", !and, or);
            assert_eq!(floating, 0b100001);

            let m = Mask { or, and, floating };

            let v = m.generate_write_addresses(42).collect::<Vec<_>>();
            assert_eq!(&v, &[26, 27, 58, 59]);
        }
        x => unreachable!("{:?}", x),
    }

    input.next().unwrap().unwrap();

    match input.next().unwrap().unwrap() {
        Op::Mask { or, and, floating } => {
            let m = Mask { or, and, floating };

            let v = m.generate_write_addresses(26).collect::<Vec<_>>();
            assert_eq!(&v, &[16, 17, 18, 19, 24, 25, 26, 27]);
        }
        x => unreachable!("{:?}", x),
    }
}
