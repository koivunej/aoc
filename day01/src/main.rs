use std::convert::TryFrom;
use std::str::FromStr;
use std::io::BufRead;

fn main() {
    eprintln!("day01 first round, reading from stdin");

    let stdin = std::io::stdin();
    let locked = stdin.lock();


    let mut sum = 0.0f64;

    for (line_num, line) in locked.lines().enumerate() {
        if let Err(e) = line {
            eprintln!("Failed to read stdin: {}", e);
            std::process::exit(1); // diverges
        }

        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }

        let mass = match f64::from_str(&line) {
            Ok(mass) if mass >= 0.0 => mass,
            Ok(mass) => {
                eprintln!("Negative mass at line {}: {}", line_num, mass);
                std::process::exit(1);
            },
            Err(e) => {
                eprintln!("Bad mass at line {}: \"{}\" ({})", line_num, line, e);
                std::process::exit(1);
            },
        };

        let module = Module::from_mass(mass);
        match module.fuel_required() {
            Ok(Fuel(f)) => sum += f,
            Err(e) => {
                eprintln!("Invalid fuel requirement for {:?}: {:?}", module, e);
                std::process::exit(1);
            }
        }
    }

    println!("{}", sum);
}

#[derive(Debug)]
struct Module { mass: f64, }

impl Module {
    fn from_mass(mass: f64) -> Self {
        assert!(mass >= 0.0);
        Self { mass }
    }

    fn fuel_required(&self) -> Result<Fuel, <Fuel as TryFrom<f64>>::Error> {
        Fuel::try_from((self.mass / 3.0).floor() - 2.0)
    }
}

#[derive(PartialEq, Debug)]
struct Fuel(f64);

impl PartialEq<f64> for Fuel {
    fn eq(&self, other: &f64) -> bool { self.0 == *other }
}

#[derive(PartialEq, Debug)]
struct NegativeFuel(f64);

impl TryFrom<f64> for Fuel {
    type Error = NegativeFuel;

    fn try_from(f: f64) -> Result<Self, Self::Error> {
        if f < 0.0 {
            Err(NegativeFuel(f))
        } else {
            Ok(Fuel(f))
        }
    }
}

#[cfg(test)]
mod test {
    use super::Module;

    #[test]
    fn examples() {
        let masses = &[12, 14, 1969, 100756];
        let answers = &[2, 2, 654, 33583];

        let solutions = masses
            .iter()
            .cloned()
            .map(|m| m as f64)
            .map(Module::from_mass)
            .map(|m| m.fuel_required())
            .zip(answers.iter().cloned().map(|a| a as f64));

        for (actual, expected) in solutions {
            assert_eq!(actual.unwrap(), expected);
        }
    }
}
