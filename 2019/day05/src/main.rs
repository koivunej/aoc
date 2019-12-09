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

    println!("stage1: {:?}", stage1(&data));
    println!("stage2: {:?}", stage2(&data));
}

fn stage1(data: &[Word]) -> Word {
    let mut data = data.to_vec();
    let mut env = Environment::collector(Some(1));

    Program::wrap_and_eval_with_env(data.as_mut_slice(), &mut env).unwrap();

    let output = env.unwrap_collected();
    *output.last().expect("No output?")
}

fn stage2(data: &[Word]) -> Word {
    let mut data = data.to_vec();
    let mut env = Environment::once(Some(5));

    Program::wrap_and_eval_with_env(data.as_mut_slice(), &mut env).unwrap();

    let output = env.unwrap_input_consumed_once();
    output.expect("No output?")
}

#[test]
fn full_stage1() {
    with_input(|data| assert_eq!(stage1(data), 9938601));
}

#[test]
fn full_stage2() {
    with_input(|data| assert_eq!(stage2(data), 4283952));
}

// FIXME: copied from day02, but too small to refactor
#[cfg(test)]
fn with_input<V, F: FnOnce(&[Word]) -> V>(f: F) -> V {
    use std::io::BufReader;

    let file = std::fs::File::open("input").expect("Could not open day02 input?");

    let data = parse_program(BufReader::new(file)).unwrap();

    f(&data)
}
