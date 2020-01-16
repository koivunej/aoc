use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::mpsc::{channel, Sender};

use intcode::{Word, util::parse_stdin_program, Program, Registers, ExecutionState};

fn main() {
    let (zero_tx, zero_rx) = channel();
    let inner = Arc::new(Mutex::new(SlowBusInner { buffers: HashMap::new(), last_nat: None, idle: 0, zero_tx }));
    let (tx, rx) = channel();
    let prog = parse_stdin_program();

    let _join_handles = (0..50).into_iter()
        .map(|addr| {
            let bus = SlowBus { bus: Arc::clone(&inner), addr, sender: Sender::clone(&tx) };
            (bus, Nic::new(&prog, addr))
        })
        .map(|(bus, mut nic)| std::thread::spawn(move || nic.run_to_halt(bus)))
        .collect::<Vec<_>>();

    let part1 = rx.recv().unwrap();

    println!("part1: {:?}", part1);

    let mut first_duplicate_y_sent_to_zero = None;

    let mut seen_ys = HashSet::new();
    for y in zero_rx {
        if !seen_ys.insert(y) {
            first_duplicate_y_sent_to_zero = Some(y);
            break;
        }
    }

    println!("part2: {:?}", first_duplicate_y_sent_to_zero);

    assert_eq!(part1, (5471, 17714));
    assert_eq!(first_duplicate_y_sent_to_zero, Some(10982));
}

trait Bus {
    fn send(&self, dest: Word, x: Word, y: Word);
    fn poll_recv(&self) -> Option<(Word, Word)>;
}

struct SlowBusInner {
    buffers: HashMap<Word, VecDeque<(Word, Word)>>,
    last_nat: Option<(Word, Word)>,
    idle: u64,
    zero_tx: Sender<Word>,
}

impl SlowBusInner {
    fn send(&mut self, src: Word, dest: Word, x: Word, y: Word) {
        if dest == 255 {
            self.last_nat = Some((x, y));
        } else {
            let msgs = self.buffers.entry(dest).or_insert_with(VecDeque::new);
            msgs.push_back((x, y));
        }
    }

    fn poll_recv(&mut self, addr: &Word) -> Option<(Word, Word)> {
        assert!(*addr >= 0);
        let ret = self.buffers.get_mut(addr).and_then(|buffer| buffer.pop_front());

        if ret.is_none() {
            self.idle |= (1 << *addr as u64);
        } else {
            self.idle &= !(1 << *addr as u64);
        }

        let all_idle = self.idle == 0x3ffffffffffff;

        if all_idle {
            if let Some((x, y)) = self.last_nat.take() {
                self.buffers.entry(0).or_insert_with(VecDeque::new).push_back((x, y));
                self.zero_tx.send(y).unwrap();
            }
        }

        ret
    }
}

struct SlowBus {
    bus: Arc<Mutex<SlowBusInner>>,
    addr: Word,
    sender: Sender<(Word, Word)>,
}

impl Bus for SlowBus {
    fn send(&self, dest: Word, x: Word, y: Word) {
        if dest == 255 {
            self.sender.send((x, y)).unwrap();
        }
        let mut g = self.bus.lock().unwrap_or_else(|e| e.into_inner());
        g.send(self.addr, dest, x, y);
    }

    fn poll_recv(&self) -> Option<(Word, Word)> {
        let mut g = self.bus.lock().unwrap_or_else(|e| e.into_inner());
        g.poll_recv(&self.addr)
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
