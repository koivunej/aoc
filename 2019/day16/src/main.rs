use std::io::BufRead;
use std::fmt;

fn main() {
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    let mut buffer = String::new();
    locked.read_line(&mut buffer).unwrap();

    let bytes = parse_str(buffer.trim());

    println!("stage1: {}", JoinedBytes(&naive_fft(bytes.iter().cloned(), 100)[..8]));
    println!("stage2: {}", JoinedBytes(&repeated_fft(&buffer, &bytes)));
}

fn naive_fft<I: Iterator<Item = u8>>(input: I, phases: usize) -> Vec<u8> {
    // nope, rayon does not make this go faster
    let base = &[0i8, 1, 0, -1];

    let mut a = input.collect::<Vec<_>>();
    let mut b = a.clone();

    for _ in 0..phases {
        for (i, x) in b.iter_mut().enumerate() {
            let sum = base.iter()
                .copied()
                .flat_map(|b| std::iter::repeat(b).take(i + 1))
                .cycle()
                .skip(1)
                .zip(a.iter().map(|v| *v as i32))
                .map(|(b, v)| (b as i32 * v))
                .sum::<i32>();

            *x = (sum.abs() % 10) as u8;
        }
        std::mem::swap(&mut a, &mut b);
    }

    a
}

fn repeated_fft(buffer: &str, bytes: &[u8]) -> Vec<u8> {
    let len = bytes.len();
    let offset = buffer[..7].parse::<usize>().unwrap();
    let mut repeated = bytes.iter()
        .cycle()
        .copied()
        .take(10_000 * len)
        .skip(offset)
        .collect::<Vec<_>>();

    // after halfpoint all of the coefficients become done. before half point everything is zero.
    assert!(offset >= 10_000 * len / 2);

    for _ in 0..100 {
        // "prefix sum" or suffix sum
        let mut tmp = 0u32;
        repeated.iter_mut().rev().for_each(|v| {
            // prefix/suffix sum which should had been seen from the examples in order to avoid the
            // O(n²). thank you 0e4ef622 for explaning this to me :)
            tmp += *v as u32;
            *v = (tmp % 10) as u8;
        });
    }

    repeated.truncate(8);
    repeated
}

#[test]
fn simplest_example() {
    let input = &[1,2,3,4,5,6,7,8];
    let input = naive_fft(input.iter().copied(), 1);
    assert_eq!(input, &[4,8,2,2,6,1,5,8]);

    let input = naive_fft(input.iter().copied(), 1);
    assert_eq!(input, &[3,4,0,4,0,4,3,8]);

    let input = naive_fft(input.iter().copied(), 1);
    assert_eq!(input, &[0,3,4,1,5,5,1,8]);

    let input = naive_fft(input.iter().copied(), 1);
    assert_eq!(input, &[0,1,0,2,9,4,9,8]);
}

#[test]
fn hundred_phase_examples() {
    let d = &[
        ("80871224585914546619083218645595", "24176176"),
        ("19617804207202209144916044189917", "73745418"),
        ("69317163492948606335995924319873", "52432133"),
    ];

    for (input, expected) in d {
        naive_fft_example(input, expected, 100);
    }
}

#[test]
fn stage2_examples() {
    let d = &[
        ("03036732577212944063491565474664", "84462026"),
        ("02935109699940807407585447034323", "78725270"),
        ("03081770884921959731165446850517", "53553731"),
    ];

    for (input, expected) in d {
        let i = parse_str(input);
        let expected = parse_str(expected);
        let actual = repeated_fft(input, &i);
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
fn naive_fft_example(input: &str, expected: &str, phases: usize) {

    let i = parse_str(input);
    let e = parse_str(expected);

    let output = naive_fft(i.iter().copied(), phases);

    assert_eq!(&output[..8], e.as_slice());
}

fn parse_str(input: &str) -> Vec<u8> {
    input.chars().map(|ch| ch as u8 - b'0').map(|b| b as u8).collect::<Vec<_>>()
}

struct JoinedBytes<'a>(&'a [u8]);

impl<'a> fmt::Display for JoinedBytes<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for b in self.0 {
            write!(fmt, "{}", b)?;
        }

        Ok(())
    }
}
