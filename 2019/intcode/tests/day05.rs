use intcode::{Program, Environment, Config};

#[test]
fn stage1_example() {
    let mut prog = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];

    let expected = &[3500isize, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50];

    Program::wrap_and_eval(&mut prog, &Config::default()).unwrap();

    assert_eq!(&prog[..], expected);
}

#[test]
fn io_example() {
    let mut prog = vec![3, 0, 4, 0, 99];
    let expected = &[1, 0, 4, 0, 99];

    let mut env = Environment::Once(Some(1), None);

    Program::wrap_and_eval_with_env(&mut prog, &mut env, &Config::day05()).unwrap();

    let output = env.unwrap_input_consumed_once();

    assert_eq!(output, Some(1));
    assert_eq!(&prog[..], expected);
}

#[test]
fn stage2_eq_ne_example() {
    let data = &[3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
    assert_eq!(stage2_example_scenario(data, 8), 1);
    assert_eq!(stage2_example_scenario(data, 7), 0);
}

#[test]
fn stage2_lt_example() {
    let data = &[3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
    assert_eq!(stage2_example_scenario(data, 7), 1);
    assert_eq!(stage2_example_scenario(data, 8), 0);
}

#[test]
fn stage2_eq_ne_example_immediate() {
    let data = &[3, 3, 1108, -1, 8, 3, 4, 3, 99];
    assert_eq!(stage2_example_scenario(data, 8), 1);
    assert_eq!(stage2_example_scenario(data, 7), 0);
}

#[test]
fn stage2_lt_example_immediate() {
    let data = &[3, 3, 1107, -1, 8, 3, 4, 3, 99];
    assert_eq!(stage2_example_scenario(data, 7), 1);
    assert_eq!(stage2_example_scenario(data, 8), 0);
}

#[test]
fn stage2_input_eq_0() {
    let addressed = &[3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
    let immediate = &[3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
    for code in &[&addressed[..], &immediate[..]] {
        assert_eq!(stage2_example_scenario(code, 0), 0);
        assert_eq!(stage2_example_scenario(code, 2), 1);
    }
}

fn stage2_example_scenario(data: &[isize], input: isize) -> isize {
    let mut prog = data.to_vec();
    let mut env = Environment::once(Some(input));
    let conf = Config::day05();

    Program::wrap_and_eval_with_env(&mut prog, &mut env, &conf).unwrap();

    let (input, output) = env.unwrap_once();
    assert_eq!(input, None);
    output.unwrap()
}

#[test]
fn stage2_larger_example() {
    let code = &[
        3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
        0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
        20, 1105, 1, 46, 98, 99,
    ];

    // careful exploration of the whole state space :)
    let params = &[(6, 999), (7, 999), (8, 1000), (9, 1001), (10, 1001)];

    for (input, expected) in params {
        let mut prog = code.to_vec();
        let mut env = Environment::collector(Some(*input));
        let conf = Config::day05();

        Program::wrap_and_eval_with_env(&mut prog, &mut env, &conf).unwrap();

        let output = env.unwrap_collected();
        assert_eq!(&output[..], &[*expected]);
    }
}
