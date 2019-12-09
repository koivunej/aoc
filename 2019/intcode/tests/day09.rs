use intcode::{Program, Environment};

#[test]
fn stage1_quine() {
    let input = &[109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99];
    let mut prog = input.to_vec();

    let mut env = Environment::collector(None);

    Program::wrap(&mut prog)
        .with_memory_expansion()
        .eval_with_env(&mut env)
        .unwrap();

    let output = env.unwrap_collected();

    assert_eq!(&input[..], &output[..]);
}

#[test]
fn stage1_16_bit_number() {
    let input = &[1102,34915192,34915192,7,4,7,99,0];
    let mut prog = input.to_vec();

    let mut env = Environment::once(None);

    Program::wrap_and_eval_with_env(&mut prog, &mut env)
        .unwrap();

    let output = env.unwrap_input_consumed_once().unwrap();

    assert!(output.abs() > 1 << 14);
}

#[test]
fn stage1_bignum() {
    let input = &[104,1125899906842624,99];
    let mut prog = input.to_vec();

    let mut env = Environment::once(None);

    Program::wrap_and_eval_with_env(&mut prog, &mut env)
        .unwrap();

    let output = env.unwrap_input_consumed_once().unwrap();

    assert_eq!(output, 1125899906842624);
}
