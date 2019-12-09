use intcode::{parse_program, Environment, ParsingError, Program, Word};

fn main() {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    let data = match parse_program(locked) {
        Ok(data) => data,
        Err(ParsingError::Io(e, line)) => {
            eprintln!("Failed to read stdin near line {}: {}", line, e);
            std::process::exit(1);
        }
        Err(ParsingError::Int(e, line, raw)) => {
            eprintln!("Bad input at line {}: \"{}\" ({})", line, raw, e);
            std::process::exit(1);
        }
    };

    println!("stage1: {:?}", boost(&data, 1));
    println!("stage2: {:?}", boost(&data, 2));
}

fn boost(data: &[Word], input: Word) -> Word {
    let mut data = data.to_vec();
    let mut env = Environment::once(Some(input));

    Program::wrap(&mut data)
        .with_memory_expansion()
        .eval_with_env(&mut env)
        .unwrap();

    env.unwrap_input_consumed_once().unwrap()
}

// TODO: add test cases for stage1 3638931938 and stage2 86025

#[test]
fn stage1_full() {
    with_input(|input| assert_eq!(boost(input, 1), 3638931938));
}

#[test]
fn stage2_full() {
    with_input(|input| assert_eq!(boost(input, 2), 86025));
}

// FIXME: copied from day02, but too small to refactor
#[cfg(test)]
fn with_input<V, F: FnOnce(&[Word]) -> V>(f: F) -> V {
    use std::io::BufReader;

    let file = std::fs::File::open("input").expect("Could not open day02 input?");

    let data = parse_program(BufReader::new(file)).unwrap();

    f(&data)
}
