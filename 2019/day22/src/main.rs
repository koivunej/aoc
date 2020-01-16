use std::convert::TryFrom;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    let mut ops = Vec::new();
    let mut buffer = String::new();
    loop {
        buffer.clear();
        match locked.read_line(&mut buffer).unwrap() {
            0 => break,
            _ => {},
        }

        ops.push(Op::try_from(buffer.trim()).unwrap());
    }

    let mut deck = (0i16..10_007).into_iter().collect::<Vec<_>>();
    let mut target = deck.clone();

    for op in ops {
        op.shuffle(&mut deck, &mut target);
        std::mem::swap(&mut deck, &mut target);
    }

    println!("part1: {}", deck.iter().position(|&card| card == 2019).unwrap());
}

enum Op {
    DealWithIncrement(i16),
    Cut(i16),
    DealIntoNewStack,
}

impl<'a> TryFrom<&'a str> for Op {
    type Error = &'a str;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if s == "deal into new stack" {
            Ok(Op::DealIntoNewStack)
        } else {
            let s = s.trim();
            let pos = s.as_bytes().iter().rposition(|&ch| ch == b' ').unwrap();
            let discriminant = &s[..pos as usize];
            let arg = &s[pos as usize + 1..];

            Ok(match discriminant {
                "deal with increment" => Op::DealWithIncrement(arg.parse::<i16>().unwrap()),
                "cut" => Op::Cut(arg.parse::<i16>().unwrap()),
                x => return Err(x),
            })
        }
    }
}

impl Op {
    fn shuffle(&self, deck: &mut Vec<i16>, target: &mut Vec<i16>) {
        target.clear();
        match *self {
            Op::DealIntoNewStack => {
                deck.reverse();
                std::mem::swap(deck, target);
            },
            Op::DealWithIncrement(incr) => {
                if incr <= 0 {
                    unreachable!();
                } else {
                    target.clear();
                    target.resize(deck.len(), 0);

                    let mut puts = 0;
                    let mut ptr = 0;

                    while puts < deck.len() {
                        target[ptr] = deck[puts];
                        puts += 1;
                        ptr = (ptr + incr as usize) % deck.len();
                    }
                }
            },
            Op::Cut(amt) => {
                target.clear();
                let amt = if amt < 0 {
                    deck.len() - (-amt) as usize
                } else if amt > 0 {
                    amt as usize
                } else {
                    unreachable!()
                };
                target.extend(&deck[amt..]);
                target.extend(&deck[..amt]);
            }
        }
    }
}

#[test]
fn reverse() {
    let mut v = Vec::new();
    v.push(0);
    v.push(1);
    v.push(2);

    let mut target = Vec::new();
    Op::DealIntoNewStack.shuffle(&mut v, &mut target);

    assert_eq!(target.into_iter().collect::<Vec<_>>(), vec![2, 1, 0]);
}

#[test]
fn cut() {
    let mut v = (0i16..10).into_iter().collect::<Vec<i16>>();
    let mut target = Vec::new();
    Op::Cut(3).shuffle(&mut v, &mut target);

    assert_eq!(target.into_iter().collect::<Vec<_>>(), vec![3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);
}

#[test]
fn cut_negative() {
    let mut v = (0i16..10).into_iter().collect::<Vec<i16>>();
    let mut target = Vec::new();
    Op::Cut(-4).shuffle(&mut v, &mut target);

    assert_eq!(target.into_iter().collect::<Vec<_>>(), vec![6, 7, 8, 9, 0, 1, 2, 3, 4, 5]);
}

#[test]
fn deal_with_incr_pos() {
    let mut v = (0i16..10).into_iter().collect::<Vec<i16>>();
    let mut target = Vec::new();
    Op::DealWithIncrement(3).shuffle(&mut v, &mut target);

    assert_eq!(target.into_iter().collect::<Vec<_>>(), vec![0, 7, 4, 1, 8, 5, 2, 9, 6, 3]);
}
