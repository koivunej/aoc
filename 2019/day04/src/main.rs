use std::fmt::Write;

fn main() {
    let range = 108457..=562041;
    let mut buf = String::with_capacity(6);

    // n_(-1) <= n

    let stage1 = range.map(move |guess| {
            buf.clear();
            write!(buf, "{}", guess).unwrap();
            analyze(&buf)
        })
        .filter(|k| k.have_it_all())
        .count();

    println!("stage1: {}", stage1);
}

#[derive(Default)]
struct Kind {
    monotonous: bool,
    have_repeat: bool,
}

impl Kind {
    fn have_it_all(&self) -> bool {
        self.monotonous && self.have_repeat
    }
}

fn analyze(buf: &str) -> Kind {

    let mut ret = Kind::default();

    for window in buf.as_bytes().windows(2) {
        let left = window[0] as u8 - b'0';
        let right = window[1] as u8 - b'0';

        if left > right {
            return ret;
        }

        ret.have_repeat |= left == right;
    }

    // not sure if this name is correct
    ret.monotonous = true;
    ret
}
