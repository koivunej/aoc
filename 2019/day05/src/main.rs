use intcode::{parse_stdin_program, Environment, Program, Word};

fn main() {
    let data = parse_stdin_program();

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
    intcode::with_parsed_program(|data| assert_eq!(stage1(data), 9938601));
}

#[test]
fn full_stage2() {
    intcode::with_parsed_program(|data| assert_eq!(stage2(data), 4283952));
}
