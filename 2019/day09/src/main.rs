use intcode::{parse_stdin_program, Environment, Program, Word};

fn main() {
    let data = parse_stdin_program();

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

#[test]
fn stage1_full() {
    intcode::with_parsed_program(|input| assert_eq!(boost(input, 1), 3638931938));
}

#[test]
fn stage2_full() {
    intcode::with_parsed_program(|input| assert_eq!(boost(input, 2), 86025));
}
