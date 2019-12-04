use std::cmp::Ordering;
use std::fmt::Write;
use std::iter::FromIterator;

fn main() {
    let range = 108_457..=562_041;

    let (stage1, stage2) = run_stages(range);

    println!("stage1: {}", stage1);
    println!("stage2: {}", stage2);
}

fn run_stages<I: Iterator<Item = u32>>(iter: I) -> (usize, usize) {
    let mut buf = String::with_capacity(6);

    iter.map(move |guess| analyze(guess, &mut buf))
        .filter(|k| k.have_any_of_it())
        .fold((0, 0), |mut counts, next| {
            if next.have_it_all(Stage::Two) {
                counts.1 += 1;
            }
            counts.0 += 1;
            counts
        })
}

enum Stage {
    One,
    Two,
}

#[derive(Default, PartialEq, Debug)]
struct Analyzed {
    monotonous: bool,
    have_repeat: bool,
    have_repeat_of_two: bool,
}

impl Analyzed {
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

impl FromIterator<Ordering> for Analyzed {
    fn from_iter<I: IntoIterator<Item = Ordering>>(iter: I) -> Self {
        use std::iter::repeat;

        let mut repeats = 0;
        let mut have_repeat_of_two = false;
        let mut max_repeat = 0;

        // chain the most neutral element (Less) to "flush" accumulated state
        // to avoid repeating it after the loop
        let chained = iter.into_iter().chain(repeat(Ordering::Less).take(1));

        for pair_ordering in chained {
            match pair_ordering {
                Ordering::Greater => return Analyzed::default(),
                Ordering::Less => {
                    max_repeat = max_repeat.max(repeats);
                    have_repeat_of_two |= repeats == 1;
                    repeats = 0;
                }
                Ordering::Equal => {
                    repeats += 1;
                }
            }
        }

        Analyzed {
            monotonous: true,
            have_repeat: max_repeat > 0,
            have_repeat_of_two,
        }
    }
}

fn analyze(guess: u32, buf: &mut String) -> Analyzed {
    buf.clear();
    write!(buf, "{}", guess).unwrap();
    buf.as_bytes()
        .windows(2)
        .map(|bytes| bytes[0].cmp(&bytes[1]))
        .collect()
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

#[test]
fn answers() {
    assert_eq!(run_stages(108_457..=562_041), (2779, 1972));
}
