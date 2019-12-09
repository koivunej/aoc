use std::io::Read;

fn main() {
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    let mut input = Vec::new();
    let _bytes = locked.read_to_end(&mut input).unwrap();

    // there is an LF in the last pos
    let (ones, twos) = &input[..input.len() - 2].chunks(25 * 6)
        .enumerate()
        .map(|(i, chunk)| (i, chunk, chunk.iter().filter(|b| (**b - '0' as u8) == 0).count()))
        .min_by_key(|(_, _, zeroes)| *zeroes)
        .into_iter()
        .fold((0, 0), |(mut ones, mut twos), (_, chunk, _)| {
            for b in chunk {
                match b {
                    b'1' => ones += 1,
                    b'2' => twos += 1,
                    _ => {},
                }
            }
            (ones, twos)
        });

    println!("stage1: {}", ones * twos);
}
