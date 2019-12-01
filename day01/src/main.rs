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

    println!("Fuel for modules: {}", sum);
    let fuel_for_fuel = FuelModule::from(Fuel::new(sum)).fuel_required();
    println!("Fuel for fuel:    {}", fuel_for_fuel.0);
    println!("Total:            {}", sum + fuel_for_fuel.0);
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

#[derive(Clone)]
struct FuelModule { fuel: Fuel }

impl From<Fuel> for FuelModule {
    fn from(fuel: Fuel) -> Self {
        Self { fuel }
    }
}

impl FuelModule {
    fn fuel_required(&self) -> Fuel {
        let mut sum = 0.0f64;

        let mut next = FuelModule::clone(self);

        loop {
            let amount = next.as_module().fuel_required().unwrap_or_default();
            if amount.0 == 0.0f64 {
                return Fuel::new(sum);
            }
            sum += amount.0;
            next = FuelModule::from(amount);
        }
    }

    fn as_module(&self) -> Module {
        Module::from_mass(self.fuel.0)
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Fuel(f64);

impl std::default::Default for Fuel {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Fuel {
    fn new(fuel: f64) -> Self {
        assert!(fuel >= 0.0);
        Self(fuel)
    }
}

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
    use std::convert::TryFrom;
    use super::{Module, Fuel, FuelModule};

    #[test]
    fn module_fuel_examples() {
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

    #[test]
    fn fuel_fuel_examples() {
        let fuel_masses = &[2, 654, 33583];
        let answers = &[0, 966 - 654, 50346 - 33583];

        let solutions = fuel_masses
            .iter()
            .map(|m| *m as f64)
            .map(|m| Fuel::try_from(m).unwrap())
            .map(FuelModule::from)
            .map(|fm| fm.fuel_required())
            .zip(answers.iter().map(|a| *a as f64));

        for (actual, expected) in solutions {
            assert_eq!(actual.0, expected);
        }
    }
}
