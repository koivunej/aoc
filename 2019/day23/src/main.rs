use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{channel, Sender};

use intcode::{Word, util::parse_stdin_program, Program, Registers, ExecutionState};

fn main() {
    let buffers = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = channel();
    let prog = parse_stdin_program();

    let join_handles = (0..50).into_iter()
        .map(|addr| {
            let bus = SlowBus { buffers: Arc::clone(&buffers), addr, sender: Sender::clone(&tx) };
            (bus, Nic::new(&prog, addr))
        })
        .map(|(bus, mut nic)| std::thread::spawn(move || nic.run_to_halt(bus)))
        .collect::<Vec<_>>();

    let part1 = rx.recv().unwrap();

    println!("part1: {:?}", part1);

    assert_eq!(part1, (5471, 17714));
}

trait Bus {
    fn send(&self, dest: Word, x: Word, y: Word);
    fn poll_recv(&self) -> Option<(Word, Word)>;
}

struct SlowBus {
    buffers: Arc<Mutex<HashMap<Word, VecDeque<(Word, Word)>>>>,
    addr: Word,
    sender: Sender<(Word, Word)>,
}

impl Bus for SlowBus {
    fn send(&self, dest: Word, x: Word, y: Word) {
        if dest == 255 {
            self.sender.send((x, y)).unwrap();
        } else {
            let mut g = self.buffers.lock().unwrap_or_else(|e| e.into_inner());
            let msgs = g.entry(dest).or_insert_with(VecDeque::new);
            msgs.push_back((x, y));
        }
    }

    fn poll_recv(&self) -> Option<(Word, Word)> {
        let mut g = self.buffers.lock().unwrap_or_else(|e| e.into_inner());
        g.get_mut(&self.addr).and_then(|buffer| buffer.pop_front())
    }
}

struct Nic {
    prog: Program<'static>,
    addr: Option<Word>,
}

enum SendMessage {
    Empty,
    Init(Word),
    Almost(Word, Word),
}

impl SendMessage {
    fn send<B: Bus>(&mut self, val: Word, bus: &mut B) {
        *self = match std::mem::replace(self, Self::Empty) {
            Self::Empty => SendMessage::Init(val),
            Self::Init(addr) => SendMessage::Almost(addr, val),
            Self::Almost(addr, x) => {
                bus.send(addr, x, val);
                SendMessage::Empty
            }
        }
    }

    fn is_empty(&self) -> bool {
        match *self {
            Self::Empty => true,
            _ => false,
        }
    }
}

impl Nic {
    fn new(mem: &[Word], addr: Word) -> Nic {
        let mem = intcode::Memory::from(mem.to_vec())
            .with_memory_expansion();

        let prog = Program::from(mem);
        Nic {
            prog,
            addr: Some(addr),
        }
    }

    fn run_to_halt<B: Bus>(&mut self, mut bus: B) {
        let mut regs = Registers::default();
        let mut buffered_msg = None;

        let mut sender = SendMessage::Empty;

        loop {
            regs = match self.prog.eval_from_instruction(regs).unwrap() {
                ExecutionState::HaltedAt(_regs) => {
                    assert!(sender.is_empty());
                    return
                },
                ExecutionState::Paused(regs) => unreachable!("Paused? {:?}", regs),
                ExecutionState::InputIO(io) => {

                    // FIXME: there must be a way to write this nicer
                    let (val, buffered) = match self.addr.take() {
                        Some(addr) => (addr, None),
                        None => match buffered_msg.as_ref() {
                            Some(y) => (*y, None),
                            None => match bus.poll_recv() {
                                Some((x, y)) => (x, Some(y)),
                                None => (-1, None),
                            },
                        }
                    };

                    buffered_msg = buffered;

                    self.prog.handle_input_completion(io, val).unwrap()
                },
                ExecutionState::OutputIO(io, value) => {
                    sender.send(value, &mut bus);
                    self.prog.handle_output_completion(io)
                },
            };
        }
    }
}
