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
}

fn stage1(data: &[Word]) -> Word {
    let mut data = data.to_vec();
    let mut env = Environment::once(Some(1));

    Program::wrap(&mut data)
        .with_memory_expansion()
        .eval_with_env(&mut env)
        .unwrap();

    env.unwrap_input_consumed_once().unwrap()
}
