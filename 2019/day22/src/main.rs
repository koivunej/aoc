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

    let part1 = if true {
        let m = 10_007;
        ops.iter().fold(Composed(1, 0), |mut acc, next| {
            acc.compose(&next.as_composed(), m);
            acc
        }).get(2019, m)
    } else if true {
        let mut tracker = CardInDeckTracker::new(10_007, 2019);

        for op in &ops {
            op.exec(&mut tracker);
        }

        tracker.pos
    } else {

        let mut deck = (0i16..10_007).into_iter().collect::<Vec<_>>();
        let mut target = deck.clone();

        for op in &ops {
            op.shuffle(&mut deck, &mut target);
            std::mem::swap(&mut deck, &mut target);
        }

        deck.iter().position(|&card| card == 2019).unwrap() as i64
    };

    println!("part1: {}", part1);

    let part2 = {
        let m = 119315717514047;
        let card = 2020;
        let times = 101741582076661;

        let mut composed = Composed(1, 0);
        for op in &ops {
            composed.compose(&op.as_composed(), m);
        }

        // well this was horrible: I couldn't had done this without reddit answers, I was all out
        // of math for this one.
        composed.powmod_get_inv(times, m, card)
    };

    println!("part2: {}", part2);

    assert_eq!(part1, 6831);
    assert_ne!(part2, 6413998145076);
    assert_ne!(part2, 73610987851873);
    assert_ne!(part2, 119315717514046);
    assert_ne!(part2, 119315717514045);
    assert_eq!(part2, 81781678911487);
}

trait CardDeck {
    fn deal_with_increment(&mut self, increment: i64);
    fn cut(&mut self, cut: i64);
    fn deal_into_new_stack(&mut self);
}

#[cfg(test)]
struct FullCardDeckTracker {
    deck: Vec<CardInDeckTracker>,
}

#[cfg(test)]
impl FullCardDeckTracker {
    fn new(cards: u64) -> Self {
        FullCardDeckTracker {
            deck: (0..cards).into_iter().map(|i| CardInDeckTracker::new(cards, i)).collect(),
        }
    }

    fn cards(&self) -> Vec<u64> {
        let mut ret = vec![0; self.deck.len()];

        for tracker in &self.deck {
            ret[tracker.pos as usize] = tracker.tracked;
        }

        ret
    }
}

#[cfg(test)]
impl CardDeck for FullCardDeckTracker {
    fn deal_with_increment(&mut self, increment: i64) {
        self.deck.iter_mut().for_each(|v| v.deal_with_increment(increment));
    }
    fn cut(&mut self, cut: i64) {
        self.deck.iter_mut().for_each(|v| v.cut(cut));
    }
    fn deal_into_new_stack(&mut self) {
        self.deck.iter_mut().for_each(|v| v.deal_into_new_stack());
    }
}

struct CardInDeckTracker {
    len: i64,
    #[cfg(test)]
    tracked: u64,
    pos: i64,
}

impl CardInDeckTracker {
    // created in factory order of 0..cards
    fn new(cards: u64, tracked: u64) -> Self {
        assert!(cards > tracked);
        Self {
            len: cards as i64,
            #[cfg(test)]
            tracked,
            pos: tracked as i64,
        }
    }
}

impl CardDeck for CardInDeckTracker {
    fn deal_with_increment(&mut self, increment: i64) {
        use num::integer::mod_floor;
        assert!(increment > 0);
        self.pos = mod_floor(self.pos * increment, self.len);
    }

    fn cut(&mut self, cut: i64) {
        use num::integer::mod_floor;
        self.pos = mod_floor(self.pos - cut, self.len);
    }

    fn deal_into_new_stack(&mut self) {
        use num::integer::mod_floor;
        self.pos = mod_floor(-1 * self.pos - 1, self.len);
    }
}

enum Op {
    DealWithIncrement(i16),
    Cut(i16),
    DealIntoNewStack,
}

struct Composed(i64, i64);

impl Composed {
    fn compose(&mut self, other: &Composed, m: i64) {
        use num::integer::mod_floor;
        self.0 = mod_floor(other.0 * self.0, m);
        self.1 = mod_floor(other.0 * self.1 + other.1, m);
    }

    fn powmod_get_inv(&self, n: i64, m: i64, tracked_pos: i64) -> i64 {
        use num::integer::Integer;
        use num::bigint::BigInt;
        use num::ToPrimitive;

        let n = BigInt::from(n);
        let m = BigInt::from(m);

        let a = BigInt::from(self.0);
        let a_k = a.modpow(&n, &m);

        let times: BigInt = 1 - &a_k;
        let div: BigInt = 1 - &a;

        let b = BigInt::from(self.1) * times;

        let b = (b * div.modpow(&(&m - 2), &m)).mod_floor(&m);

        let inv = (BigInt::from(tracked_pos) - b) * a_k.modpow(&(&m - 2), &m);

        inv.mod_floor(&m).to_i64().unwrap()
    }

    fn get(&self, tracked: i64, m: i64) -> i64 {
        use num::integer::mod_floor;
        mod_floor(self.0 * tracked + self.1, m)
    }
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

    fn as_composed(&self) -> Composed {
        match *self {
            Op::DealIntoNewStack => Composed(-1, -1),
            Op::DealWithIncrement(incr) => Composed(incr as i64, 0),
            Op::Cut(cut) => Composed(1, -cut as i64),
        }
    }

    fn exec<D: CardDeck>(&self, deck: &mut D) {
        match *self {
            Op::DealIntoNewStack => deck.deal_into_new_stack(),
            Op::DealWithIncrement(incr) => deck.deal_with_increment(incr as i64),
            Op::Cut(cut) => deck.cut(cut as i64),
        }
    }

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

    assert_eq!(
        target.iter().copied().collect::<Vec<_>>(),
        vec![2, 1, 0]);

    let mut multi = FullCardDeckTracker::new(3);
    multi.deal_into_new_stack();

    assert_eq!(
        multi.cards(),
        &[2, 1, 0]);

    Op::DealIntoNewStack.shuffle(&mut target, &mut v);

    multi.deal_into_new_stack();

    assert_eq!(
        v.iter().copied().collect::<Vec<_>>(),
        &[0, 1, 2]);
}

#[test]
fn cut() {
    let mut v = (0i16..10).into_iter().collect::<Vec<i16>>();
    let mut target = Vec::new();
    Op::Cut(3).shuffle(&mut v, &mut target);

    assert_eq!(
        target.iter().copied().collect::<Vec<_>>(),
        &[3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);

    let mut multi = FullCardDeckTracker::new(10);
    multi.cut(3);

    assert_eq!(
        multi.cards(),
        &[3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);
}

#[test]
fn cut_negative() {
    let mut v = (0i16..10).into_iter().collect::<Vec<i16>>();
    let mut target = Vec::new();
    Op::Cut(-4).shuffle(&mut v, &mut target);

    assert_eq!(
        target.iter().copied().collect::<Vec<_>>(),
        &[6, 7, 8, 9, 0, 1, 2, 3, 4, 5]);

    let mut multi = FullCardDeckTracker::new(10);
    multi.cut(-4);

    assert_eq!(
        multi.cards(),
        &[6, 7, 8, 9, 0, 1, 2, 3, 4, 5]);
}

#[test]
fn deal_with_incr_pos() {
    let mut v = (0i16..10).into_iter().collect::<Vec<i16>>();
    let mut target = Vec::new();
    Op::DealWithIncrement(3).shuffle(&mut v, &mut target);

    assert_eq!(
        target.iter().copied().collect::<Vec<_>>(),
        &[0, 7, 4, 1, 8, 5, 2, 9, 6, 3]);

    let mut multi = FullCardDeckTracker::new(10);
    multi.deal_with_increment(3);

    assert_eq!(
        multi.cards(),
        &[0, 7, 4, 1, 8, 5, 2, 9, 6, 3]);
}
