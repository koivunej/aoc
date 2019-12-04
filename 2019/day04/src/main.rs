use std::fmt::Write;

fn main() {
    let range = 108_457..=562_041;
    let mut buf = String::with_capacity(6);

    let (stage1, stage2) = range
        .map(move |guess| analyze(guess, &mut buf))
        .filter(|k| k.have_any_of_it())
        .fold((0, 0), |mut counts, next| {
            if next.have_it_all(Stage::Two) {
                counts.1 += 1;
            }
            counts.0 += 1;
            counts
        });

    println!("stage1: {}", stage1);
    println!("stage2: {}", stage2);
}

enum Stage {
    One,
    Two,
}

#[derive(Default, PartialEq, Debug)]
struct Kind {
    monotonous: bool,
    have_repeat: bool,
    have_repeat_of_two: bool,
}

impl Kind {
    fn have_any_of_it(&self) -> bool {
        self.have_it_all(Stage::One)
    }

    fn have_it_all(&self, stage: Stage) -> bool {
        match stage {
            Stage::One => self.monotonous && self.have_repeat,
            Stage::Two => self.have_it_all(Stage::One) && self.have_repeat_of_two,
        }
    }
}

fn analyze(guess: u32, buf: &mut String) -> Kind {
    buf.clear();
    write!(buf, "{}", guess).unwrap();
    analyze_str(&buf)
}

fn analyze_str(buf: &str) -> Kind {
    let mut ret = Kind::default();
    let mut repeat = None;

    for window in buf.as_bytes().windows(2) {
        let left = window[0] as u8 - b'0';
        let right = window[1] as u8 - b'0';

        if left > right {
            return ret;
        }

        let (updated_repeat, have_repeat_of_two) = match (repeat.take(), left == right) {
            (Some(count), true) => (Some(count + 1), false),
            (Some(count), false) => (None, count == 1),
            (None, true) => (Some(1), false),
            (None, false) => (None, false),
        };

        ret.have_repeat |= left == right;
        ret.have_repeat_of_two |= have_repeat_of_two;
        repeat = updated_repeat;
    }

    if let Some(count) = repeat.take() {
        ret.have_repeat_of_two |= count == 1;
    }

    ret.monotonous = true;
    ret
}

#[test]
fn stage1_examples() {
    let mut buf = String::new();
    assert!(analyze(111_111, &mut buf).have_it_all(Stage::One));
    assert!(!analyze(223_450, &mut buf).have_it_all(Stage::One));
    assert!(!analyze(123_789, &mut buf).have_it_all(Stage::One));
}

#[test]
fn stage2_examples() {
    let mut buf = String::new();
    assert!(analyze(112233, &mut buf).have_it_all(Stage::Two));
    assert!(!analyze(123444, &mut buf).have_it_all(Stage::Two));
    assert!(analyze(111122, &mut buf).have_it_all(Stage::Two));
}
