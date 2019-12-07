use std::io::BufRead;
use std::convert::TryFrom;

fn main() {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    let mut program = Vec::new();

    for line in locked.lines() {
        let line = line.unwrap();
        program.extend(line.split(',').map(|p| p.parse::<isize>().unwrap()));
    }

    println!("stage1: {}", find_max_output(0, &program[..]));
}

fn find_max_output(seed: isize, program: &[isize]) -> isize {
    let combined = CombinedMachine::new(&program[..]);

    let mut data = vec![0, 1, 2, 3, 4];
    permutohedron::Heap::new(&mut data).into_iter()
        .map(|settings| PhaseSettings::try_from(settings.to_vec()).unwrap())
        .map(move |settings| combined.in_sequence(seed, &(settings.0)[..]))
        .max()
        .unwrap()
}

struct CombinedMachine<'a> {
    program: &'a [isize],
}

impl<'a> CombinedMachine<'a> {
    fn new(program: &'a [isize]) -> Self {
        CombinedMachine {
            program,
        }
    }

    fn in_sequence(&self, seed: isize, settings: &[isize]) -> isize {
        use std::iter::repeat;
        use std::collections::VecDeque;
        use intcode::{Program, Environment};

        let mut tmp = Vec::new();

        let ret = settings
            .iter()
            .zip(repeat(self.program)) //
            .enumerate()
            .scan(seed, move |input_signal, (index, (phase_setting, data))| {
                tmp.clear();
                tmp.resize(data.len(), 0);
                tmp.copy_from_slice(data);

                let inputs = {
                    let mut inputs = VecDeque::new();
                    inputs.push_back(*phase_setting);
                    inputs.push_back(*input_signal);
                    inputs
                };

                let mut env = Environment::collected_with_many_inputs(inputs);

                let res = Program::wrap_and_eval_with_env(
                    &mut tmp[..],
                    &mut env);

                match res {
                    Ok(_) => {},
                    Err(e) => {
                        panic!("Failed {}th run with inputs ({}, {}): {:?}", index, phase_setting, input_signal, e);
                    }
                }

                let outputs = env.unwrap_collected();
                assert_eq!(outputs.len(), 1);

                println!("{}: ({}, {}, _) => {}", index, phase_setting, input_signal, outputs[0]);
                *input_signal = outputs[0];
                Some(outputs[0])
            })
            .last()
            .unwrap();

        println!("phases: {:?} output = {}", settings, ret);
        ret
    }
}

struct PhaseSettings(Vec<isize>);

#[derive(Debug)]
enum InvalidPhaseSettings {
    WrongNumber,
    OutOfRange,
    Duplicates
}

impl TryFrom<Vec<isize>> for PhaseSettings {
    type Error = InvalidPhaseSettings;

    fn try_from(v: Vec<isize>) -> Result<Self, Self::Error> {
        if v.len() != 5 {
            return Err(InvalidPhaseSettings::WrongNumber);
        }

        v.iter()
            .try_fold([false; 5], |mut acc, next| {

            let next = *next;

            if next < 0 || next > 4 {
                return Err(InvalidPhaseSettings::OutOfRange);
            }

            let next = next as usize;

            if acc[next] {
                return Err(InvalidPhaseSettings::Duplicates);
            }

            acc[next] = true;
            Ok(acc)
        })?;

        Ok(PhaseSettings(v))
    }
}

#[test]
fn stage1_example1() {
    let program = &[3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0];

    assert_eq!(find_max_output(0, &program[..]), 43210);
}
