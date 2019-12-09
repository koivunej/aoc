use std::io::Read;
use std::iter::repeat;

fn main() {
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    let mut input = Vec::new();
    let _bytes = locked.read_to_end(&mut input).unwrap();

    // there is an LF in the last pos
    let (ones, twos) = &input[..input.len() - 1].chunks(25 * 6)
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

    let w = 25;
    let h = 6;

    let image = to_image(&input[..input.len() - 1], w, h);

    let header_footer = repeat(b'0').take(w);

    for (i, color) in image.into_iter().enumerate() {
        let ch = match color {
            b'2' => '!',
            b'0' => 'X',
            b'1' => ' ',
            x => panic!("invalid byte in image: {:?}", x),
        };

        if i > 0 && i % w == 0 {
            println!();
        }

        print!("{}", ch);
    }

    println!();
}

fn to_image(raw: &[u8], width: usize, height: usize) -> Vec<u8> {
    let black = '0' as u8;
    let white = '1' as u8;
    let transparent = '2' as u8;

    assert_eq!(raw.len() % (width * height), 0);

    raw.chunks(width * height)
        .fold(vec![transparent; width * height], |mut image, layer| {
            for (i, b) in layer.iter().enumerate() {
                match b {
                    b'2' => {},
                    color if *color  == black || *color == white => {
                        if image[i] != transparent {
                            continue;
                        }
                        image[i] = *color;
                    },
                    other => panic!("unexpected color {:?}", other),
                }
            }
            image
        })
}

#[test]
fn stage2_example() {
    let input = b"0222112222120000";
    let expected = b"0110";

    let image = to_image(input, 2, 2);
    assert_eq!(&image[..], expected);
}
