use either::Either;
use num_bigint::BigInt;
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let notes = Notes::read(std::io::stdin().lock())?;

    let (earliest_busline, ttl): (usize, usize) = notes
        .buslines
        .iter()
        .filter_map(|e| e.as_ref().left())
        .map(|x| {
            (
                *x,
                notes.first_timestamp_to_depart + x - (notes.first_timestamp_to_depart % x),
            )
        })
        .min_by_key(|&x| x.1)
        .expect("multiple buslines => min must exist");

    let part_one = (ttl - notes.first_timestamp_to_depart) * earliest_busline;

    let mut remainders = Vec::new();
    let mut modulii = Vec::new();

    notes
        .buslines
        .iter()
        .enumerate()
        .filter_map(|(i, e)| e.as_ref().left().map(|e| (i, e)))
        .map(|(t_plus, busline)| {
            // t_plus is the minute offset for this busline from a timestamp after which all busses
            // leave one each minute.
            //
            // so at ts: (ts + t_plus) % busline == 0
            // it is not product of buslines
            //
            // buslines are primes. better version of the equation:
            //
            // ts % busline == t_plus
            //
            // chinese remainder theory is most likely, online calculator doesn't yield correct
            // results (for example1: 2093560), I may have typoed though. yeah, we get correct
            // answers when:
            //   ts = 0 mod first
            //   ts = (second-t_plus) mod second
            //   ts = (third-t_plus) mod third

            let m = (*busline as i64) - (t_plus as i64);
            (
                BigInt::from(m.rem_euclid(*busline as i64)),
                BigInt::from(*busline),
            )
        })
        .for_each(|(remainder, moduli)| {
            // TODO: there must be an std fn for this
            remainders.push(remainder);
            modulii.push(moduli);
        });

    // sadly ring_algorithm::chinese_remainder_theorem will only yield negative results and I
    // cannot figure out how to make it positive. https://www.dcode.fr/chinese-remainder did get
    // the expected positive accepted result right away. took quite a bit of time to wrangle the
    // gauss method for chinese remainder.

    let part_two = chinese_remainder_gauss(&remainders, &modulii).unwrap();

    println!("{}", part_one);
    println!("{}", part_two);
    assert_ne!(part_one, 369);
    assert_eq!(part_one, 115);

    assert_eq!(part_two, BigInt::from(756_261_495_958_122u64));

    Ok(())
}

fn invmod(x: &BigInt, y: &BigInt) -> Option<BigInt> {
    use num_bigint::Sign;
    use num_integer::Integer;
    let (gcd, _) = x.extended_gcd_lcm(y);

    if gcd.gcd != BigInt::from(1) {
        None
    } else {
        // failed incredibly many times around here
        let a = gcd.x;
        let n = y;
        let a = a % n;

        if a.sign() == Sign::Minus {
            Some(a + n)
        } else {
            Some(a)
        }
    }
}

#[test]
fn yt_invmod() {
    assert_eq!(
        invmod(&BigInt::from(40), &BigInt::from(7)),
        Some(BigInt::from(3))
    );
}

fn chinese_remainder_gauss(remainders: &[BigInt], modulii: &[BigInt]) -> Option<BigInt> {
    use num_bigint::Sign;

    // following https://shainer.github.io/crypto/math/2017/10/22/chinese-remainder-theorem.html
    let big_n: BigInt = modulii.iter().product();

    remainders
        .iter()
        .zip(modulii.iter())
        .try_fold(BigInt::from(0), |acc, (ri, mi)| {
            // lets hope this truncates
            let bi = &big_n / mi;
            assert_ne!(ri.sign(), Sign::Minus);
            assert_ne!(mi.sign(), Sign::Minus);

            let inv = invmod(&bi, mi)?;

            Some(acc + ri * bi * inv)
        })
        .map(|result| result % big_n)
}

#[test]
fn yt_example_for_chinese_remainder_gauss() {
    let remainders = [3, 1, 6]
        .iter()
        .copied()
        .map(BigInt::from)
        .collect::<Vec<_>>();

    let modulii = [5, 7, 8]
        .iter()
        .copied()
        .map(BigInt::from)
        .collect::<Vec<_>>();

    assert_eq!(
        chinese_remainder_gauss(&remainders, &modulii),
        Some(BigInt::from(78))
    );
}

struct Notes {
    first_timestamp_to_depart: usize,
    buslines: Vec<Either<usize, X>>,
}

impl Notes {
    fn read<R: BufRead>(mut input: R) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let mut line = String::new();

        let read = input.read_line(&mut line)?;
        if read == 0 {
            todo!("timestamp");
        }

        let first_timestamp_to_depart = line.trim().parse::<usize>()?;

        line.clear();

        let read = input.read_line(&mut line)?;
        if read == 0 {
            todo!("timetables");
        }

        let buslines = line
            .trim()
            .split(',')
            .map::<Result<Either<usize, X>, <usize as std::str::FromStr>::Err>, _>(|part| {
                Ok(if part == "x" {
                    Either::Right(X)
                } else {
                    Either::Left(part.parse::<usize>()?)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Notes {
            first_timestamp_to_depart,
            buslines,
        })
    }
}

// the x which appears in the notes
struct X;
