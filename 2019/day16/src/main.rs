fn main() {
    println!("Hello, world!");
}

fn fft<T: AsRef<[u8]>>(input: T, phases: usize) -> Vec<u8> {
    let base = &[0i16, 1, 0, -1];

    let mut a = input.as_ref().to_vec();
    let mut b = a.clone();

    for _ in 0..phases {
        for i in 0..input.as_ref().len() {
            let s = BasePattern(base, 0)
                .flat_map(|v| std::iter::repeat(v).take(i + 1))
                .skip(1)
                .zip(a.iter().map(|v| *v as i16))
                .inspect(|(b, v)| print!("{:>2} * {:>2} = {:>3} ", v, b, *v * *b))
                .map(|(b, v)| v * b)
                .sum::<i16>();

            b[i] = (s.abs() % 10) as u8;
            println!(" ===> {}", b[i]);
        }
        println!("--");
        std::mem::swap(&mut a, &mut b);
    }

    a
}

struct BasePattern<'a>(&'a [i16], usize);

impl<'a> Iterator for BasePattern<'a> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0[self.1];
        self.1 = (self.1 + 1) % self.0.len();
        Some(ret)
    }
}

#[test]
fn simplest_example() {
    let input = &[1,2,3,4,5,6,7,8];
    let input = fft(input, 1);
    assert_eq!(input, &[4,8,2,2,6,1,5,8]);

    let input = fft(input, 1);
    assert_eq!(input, &[3,4,0,4,0,4,3,8]);

    let input = fft(input, 1);
    assert_eq!(input, &[0,3,4,1,5,5,1,8]);

    let input = fft(input, 1);
    assert_eq!(input, &[0,1,0,2,9,4,9,8]);
}

#[test]
#[ignore]
fn hundred_phase_examples() {
    let d = &[
        ("80871224585914546619083218645595", "24176176"),
        ("19617804207202209144916044189917", "73745418"),
        ("69317163492948606335995924319873", "52432133"),
    ];

    for (input, expected) in d {
        fft_example(input, expected, 100);
    }
}

#[cfg(test)]
fn fft_example(input: &str, expected: &str, phases: usize) {

    let i = input.chars().map(|ch| ch as u8 - b'0').map(|b| b as u8).collect::<Vec<_>>();
    let e = expected.chars().map(|ch| ch as u8 - b'0').map(|b| b as u8).collect::<Vec<_>>();

    let output = fft(i.as_slice(), phases);

    assert_eq!(&output[..8], e.as_slice());
}
