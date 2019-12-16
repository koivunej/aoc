fn main() {
    println!("Hello, world!");
}

fn fft<T: AsRef<[u8]>>(input: T, phases: usize) -> Vec<u8> {
    Vec::new()
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

    let i = input.chars().map(|ch| ch as u8 - b'0').collect::<Vec<_>>();
    let e = expected.chars().map(|ch| ch as u8 - b'0').collect::<Vec<_>>();

    let output = fft(i.as_slice(), 100);

    assert_eq!(output.as_slice(), e.as_slice());
}
