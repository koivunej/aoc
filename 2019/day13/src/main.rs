use std::collections::HashSet;
use intcode::{parse_stdin_program, Program, Environment, Word};

fn main() {
    let data = parse_stdin_program();

    println!("stage1: {}", stage1(&data[..]));
}

fn stage1(data: &[Word]) -> usize {
    let mut data = data.to_vec();
    let mut env = Environment::collector(None);

    Program::wrap(&mut data)
        .with_memory_expansion()
        .eval_with_env(&mut env)
        .unwrap();

    let collected = env.unwrap_collected();

    let mut uniq = HashSet::new();
    for p in collected.chunks(3).filter(|chunk| chunk[2] == 2).map(|chunk| (chunk[0], chunk[1])) {
        uniq.insert(p);
    }

    uniq.len()
}
