use intcode::{parse_stdin_program, Program, Word};

fn main() {
    let data = parse_stdin_program();

    {
        println!("Value at position 0: {}", stage1(&data[..]));
    }

    {
        let magic = 19690720;

        if let Some((i, j)) = find_coords(&data[..], magic) {
            println!(
                "Found it at {:?}: 100 * noun + verb == {}",
                (i, j),
                100 * i + j
            );
        } else {
            println!("Did not find...");
        }
    }
}

fn stage1(data: &[Word]) -> Word {
    let mut data = data.to_vec();

    // restore
    data[1] = 12;
    data[2] = 2;

    Program::wrap_and_eval(&mut data).expect("Invalid program");
    data[0]
}

fn find_coords(input: &[Word], magic: Word) -> Option<(Word, Word)> {
    let mut copy = input.to_vec();

    for i in 0..100 {
        for j in 0..100 {
            copy[1] = i;
            copy[2] = j;

            Program::wrap_and_eval(&mut copy)
                .expect("Failed to execute program");

            if copy[0] == magic {
                return Some((i, j));
            }

            copy.copy_from_slice(&input[..]);
        }
    }

    None
}

#[test]
fn full_stage1() {
    intcode::with_parsed_program(|data| {
        assert_eq!(stage1(data), 3224742);
    });
}

#[test]
fn full_stage2() {
    intcode::with_parsed_program(|data| {
        let magic = 19690720;
        let res = find_coords(data, magic).map(|(noun, verb)| 100 * noun + verb);
        assert_eq!(res, Some(7960));
    });
}
