use std::convert::TryFrom;
use std::borrow::Cow;
use intcode::{Word, parse_stdin_program};

fn main() {
    let program = parse_stdin_program();

    println!("stage1: {}", find_max_output(0, &program[..]));
    println!("stage2: {}", find_max_feedback_output(0, &program[..]));
}

fn find_max_output(seed: Word, program: &[Word]) -> Word {
    let combined = CombinedMachine::new(&program[..]);

    let mut data = vec![0, 1, 2, 3, 4];
    permutohedron::Heap::new(&mut data).into_iter()
        .map(|settings| PhaseSettings::try_from(Cow::from(settings)).unwrap())
        .map(move |settings| combined.in_sequence(seed, settings.as_ref()))
        .max()
        .unwrap()
}

fn find_max_feedback_output(seed: Word, program: &[Word]) -> Word {
    let combined = CombinedMachine::new(&program[..]);

    let mut data = vec![5, 6, 7, 8, 9];
    permutohedron::Heap::new(&mut data).into_iter()
        .map(|settings| PhaseSettings::try_from(Cow::from(settings)).unwrap())
        .map(move |settings| combined.in_feedback_seq(seed, settings.as_ref()))
        .max()
        .unwrap()
}

struct CombinedMachine<'a> {
    program: &'a [Word],
}

impl<'a> CombinedMachine<'a> {
    fn new(program: &'a [Word]) -> Self {
        CombinedMachine {
            program,
        }
    }

    fn in_sequence(&self, seed: Word, settings: &[Word]) -> Word {
        use std::iter::repeat;
        use std::collections::VecDeque;
        use intcode::{Program, Environment};

        let mut tmp = Vec::new();

        let ret = settings
            .iter()
            .zip(repeat(self.program))
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

                *input_signal = outputs[0];
                Some(*input_signal)
            })
            .last()
            .unwrap();

        ret
    }

    fn in_feedback_seq(&self, seed: Word, settings: &[Word]) -> Word {
        //Strategy::ThreadedNaive.in_feedback_seq(self.program, seed, settings)
        Strategy::SingleThread.in_feedback_seq(self.program, seed, settings)
    }
}

enum Strategy {
    ThreadedNaive,
    SingleThread,
}

impl Strategy {
    fn in_feedback_seq(&self, program: &[Word], seed: Word, settings: &[Word]) -> Word {
        match *self {
            Self::ThreadedNaive => threaded_naive(program, seed, settings),
            Self::SingleThread => single_thread(program, seed, settings),
        }
    }
}

fn threaded_naive(program: &[Word], seed: Word, settings: &[Word]) -> Word {
    use std::iter::repeat;
    use intcode::{Program, ExecutionState, Registers};
    use std::sync::mpsc::{channel, TryRecvError, SendError};

    let count = settings.len();
    let range = 0..count;

    let mut channels = range.clone()
        .map(|_| channel::<Word>())
        .map(|(tx, rx)| (Some(tx), Some(rx)))
        .collect::<Vec<_>>();

    // seed -+-> 1 -> 2 -> 3 -> 4 -> 5 --+---\
    //       \___________________________/   |
    //                                       \--> output
    //

    // send out the phase settings first
    settings.iter()
        .zip(channels.iter().map(|(tx, _)| tx.as_ref().unwrap()))
        .for_each(|(phase, tx)| tx.send(*phase).unwrap());

    // keep this for now, lets start everything up before seeding
    let seeder = channels[0].0.as_ref().cloned().unwrap();

    let join_handles = range.clone()
        .map(|index| (index + 1) % count)
        .zip(range)
        // output is always sent to next (index + 1), input is always read from index
        .map(|(output_index, input_index)| (channels[output_index].0.take().unwrap(), channels[input_index].1.take().unwrap()))
        // each have their own owned copy of the program
        .zip(repeat(program).map(|p| p.to_vec()))
        // each run in separate threads
        .enumerate()
        .map(|(tid, ((tx, rx), mut prog))| std::thread::spawn(move || {
            let mut p = Program::wrap(&mut prog);
            let mut regs = Registers::default();
            let mut last_output = None;
            let mut remote_disconnected = false;
            loop {
                regs = match p.eval_from_instruction(regs).unwrap() {
                    ExecutionState::Paused(_) => unreachable!(),
                    ExecutionState::HaltedAt(_) => {
                        return last_output.expect("Nothing was output?");
                    },
                    ExecutionState::InputIO(io) => {
                        let read = match rx.try_recv() {
                            Ok(read) => {
                                read
                            },
                            Err(TryRecvError::Empty) => {
                                let read = rx.recv().unwrap();
                                read
                            },
                            Err(TryRecvError::Disconnected) => {
                                panic!("{} was disconnected", tid);
                            },
                        };
                        p.handle_input_completion(io, read).unwrap()
                    }
                    ExecutionState::OutputIO(io, val) => {
                        last_output = Some(val);
                        match tx.send(val) {
                            Ok(_) => {},
                            Err(SendError(_)) => {
                                // allow this to happen once; it does not always happen as the
                                // first one may still be alive when the message is sent but it
                                // will never consume it
                                assert!(!remote_disconnected);
                                remote_disconnected = true;
                            }
                        }
                        p.handle_output_completion(io)
                    }
                }
            }
        }))
        .collect::<Vec<_>>();

    // everyone is up and running, hopefully blocking soon, seed the first
    seeder.send(seed).unwrap();
    // no need to keep the channel up for us
    drop(seeder);

    join_handles.into_iter()
        .map(|jh| jh.join())
        .enumerate()
        .map(|(tid, res)| match res {
            Ok(x) => x,
            Err(e) => {
                // this is always "Any" so not really helpful
                panic!("{}: returned error of type {:?}", tid, e);
            }
        })
        .last()
        .unwrap()
}

fn single_thread(program: &[Word], seed: Word, settings: &[Word]) -> Word {
    use std::iter::repeat;
    use std::collections::VecDeque;
    use intcode::{Program, ExecutionState, Registers};

    let mut inputs = settings.iter().cloned().map(|i| { let mut v = VecDeque::new(); v.push_back(i); v }).collect::<Vec<_>>();
    let mut programs = repeat(program.to_vec()).take(settings.len()).collect::<Vec<_>>();
    let mut registers = (0..settings.len()).into_iter().map(|_| Some(ExecutionState::Paused(Registers::default()))).collect::<Vec<_>>();

    inputs[0].push_back(seed);

    let mut i = 0;

    while !registers.iter().all(|r| if let Some(&ExecutionState::HaltedAt(_)) = r.as_ref() { true } else { false }) {

        if let Some(&ExecutionState::HaltedAt(_)) = registers[i].as_ref() {
            i = (i + 1) % settings.len();
            continue;
        }

        let next = match registers[i].take() {
            Some(ExecutionState::Paused(regs)) => {
                Program::wrap(&mut programs[i])
                    .eval_from_instruction(regs)
                    .unwrap()
            },
            Some(ExecutionState::HaltedAt(_)) => unreachable!(),
            Some(ExecutionState::InputIO(io)) if !inputs[i].is_empty() => {
                let mut p = Program::wrap(&mut programs[i]);
                let regs = p.handle_input_completion(io, inputs[i].pop_front().unwrap()).unwrap();
                p.eval_from_instruction(regs)
                    .unwrap()
            },
            Some(x @ ExecutionState::InputIO(_)) => x,
            Some(ExecutionState::OutputIO(io, val)) => {
                let index = (i + 1) % settings.len();
                inputs[index].push_back(val);

                let mut p = Program::wrap(&mut programs[i]);
                let regs = p.handle_output_completion(io);
                p.eval_from_instruction(regs)
                    .unwrap()
            }
            None => unreachable!(),
        };

        registers[i] = Some(next);
        i = (i + 1) % settings.len();
    }

    assert_eq!(inputs[i].len(), 1);
    inputs[i].pop_front().unwrap()
}

struct PhaseSettings<'a>(Cow<'a, [Word]>);

impl<'a> AsRef<[Word]> for PhaseSettings<'a> {
    fn as_ref(&self) -> &[Word] {
        self.0.as_ref()
    }
}

#[derive(Debug)]
enum InvalidPhaseSettings {
    WrongNumber,
    OutOfRange,
    Duplicates
}

impl<'a> TryFrom<Cow<'a, [Word]>> for PhaseSettings<'a> {
    type Error = InvalidPhaseSettings;

    fn try_from(v: Cow<'a, [Word]>) -> Result<Self, Self::Error> {
        if v.len() != 5 {
            return Err(InvalidPhaseSettings::WrongNumber);
        }

        let mut min = None;
        let mut max = None;

        for x in v.iter() {
            let x = *x;
            min = min.map(|m: Word| m.min(x)).or_else(|| Some(x));
            max = max.map(|m: Word| m.max(x)).or_else(|| Some(x));
        }

        let min = min.expect("Length already checked, there must be minimum");
        let max = max.expect("Length already checked, there must be maximum");

        if max - min != 4 {
            return Err(InvalidPhaseSettings::OutOfRange);
        }

        v.iter()
            .try_fold([false; 5], |mut acc, next| {

            let next = *next;

            if next < 0 {
                return Err(InvalidPhaseSettings::OutOfRange);
            }

            let index = (next - min) as usize;

            if acc[index] {
                return Err(InvalidPhaseSettings::Duplicates);
            }

            acc[index] = true;
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

#[test]
fn stage2_example1() {
    let program = &[3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5];

    assert_eq!(find_max_feedback_output(0, &program[..]), 139629729);
}

#[test]
fn stage2_example2() {
    let program = &[3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10];

    assert_eq!(find_max_feedback_output(0, &program[..]), 18216);
}

#[test]
fn stage1_full() {
    intcode::with_parsed_program(|input| assert_eq!(find_max_output(0, input), 212460));
}

#[test]
fn stage2_full() {
    intcode::with_parsed_program(|input| assert_eq!(find_max_feedback_output(0, input), 21844737));
}
