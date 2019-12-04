use std::fmt::Write;

fn main() {
    let range = 108457..=562041;
    let mut buf = String::with_capacity(6);

    // n_(-1) <= n

    let stage1 = range.map(move |guess| analyze(guess, &mut buf))
        .filter(|k| k.have_it_all())
        .count();

    println!("stage1: {}", stage1);
}

#[derive(Default, PartialEq, Debug)]
struct Kind {
    monotonous: bool,
    have_repeat: bool,
}

impl Kind {
    fn have_it_all(&self) -> bool {
        self.monotonous && self.have_repeat
    }
}

fn analyze(guess: u32, buf: &mut String) -> Kind {
    buf.clear();
    write!(buf, "{}", guess).unwrap();
    analyze_str(&buf)
}

fn analyze_str(buf: &str) -> Kind {

    let mut ret = Kind::default();

    for window in buf.as_bytes().windows(2) {
        let left = window[0] as u8 - b'0';
        let right = window[1] as u8 - b'0';

        if left > right {
            return ret;
        }

        ret.have_repeat |= left == right;
    }

    ret.monotonous = true;
    ret
}

#[test]
fn stage1_examples() {
    let mut buf = String::new();
    assert!(analyze(111_111, &mut buf).have_it_all());
    assert!(!analyze(223_450, &mut buf).have_it_all());
    assert!(!analyze(123_789, &mut buf).have_it_all());
}

#[test]
fn stage2_examples() {
    unimplemented!()
}
