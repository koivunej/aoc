use intcode::{Program, Config, Environment, parse_program, ParsingError};


fn main() {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    let data = match parse_program(locked) {
        Ok(data) => data,
        Err(ParsingError::Io(e, line)) => {
            eprintln!("Failed to read stdin near line {}: {}", line, e);
            std::process::exit(1);
        },
        Err(ParsingError::Int(e, line, raw)) => {
            eprintln!("Bad input at line {}: \"{}\" ({})", line, raw, e);
            std::process::exit(1);
        }
    };

    {
        let mut data = data.to_vec();
        let mut env = Environment::collector(Some(1));

        Program::wrap_and_eval_with_env(data.as_mut_slice(), &mut env, &Config::day05())
            .unwrap();

        let output = env.unwrap_collected();
        println!("stage1: {:?}", output);
    }
}
