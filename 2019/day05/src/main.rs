use intcode::{parse_program, Config, Environment, ParsingError, Program};

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

fn stage1(data: &[isize]) -> isize {
    let mut data = data.to_vec();
    let mut env = Environment::collector(Some(1));

    Program::wrap_and_eval_with_env(data.as_mut_slice(), &mut env, &Config::day05()).unwrap();

    let output = env.unwrap_collected();
    *output.last().expect("No output?")
}

fn stage2(data: &[isize]) -> isize {
    let mut data = data.to_vec();
    let mut env = Environment::once(Some(5));

    Program::wrap_and_eval_with_env(data.as_mut_slice(), &mut env, &Config::day05()).unwrap();

    let (_, output) = env.unwrap_once();
    output.expect("No output?")
}

#[test]
fn full_stage1() {
    with_input(|data| assert_eq!(stage1(data), 9938601));
}

// FIXME: copied from day02, should refactor this into some test-support
#[cfg(test)]
fn with_input<V, F: FnOnce(&[isize]) -> V>(f: F) -> V {
    use std::io::BufReader;

    let file = std::fs::File::open("input").expect("Could not open day02 input?");

    let data = parse_program(BufReader::new(file)).unwrap();

    f(&data)
}
