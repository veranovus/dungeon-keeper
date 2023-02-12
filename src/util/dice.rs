#![allow(dead_code)]

use core::fmt;
use rand::Rng;
use log::error;

// NOTE: Possible dice sides.
#[derive(Debug, Clone, Copy)]
pub enum Die {
    D4(u32),
    D6(u32),
    D8(u32),
    D10(u32),
    D12(u32),
    D20(u32),
    D100(u32),
}

impl Die {
    pub fn from_sides(side: u32, value: u32) -> Self {
        return match side {
            4 => Die::D4(value),
            6 => Die::D6(value),
            8 => Die::D8(value),
            10 => Die::D10(value),
            12 => Die::D12(value),
            20 => Die::D20(value),
            100 => Die::D100(value),
            _ => {
                error!("Failed to match the value given for argument `side: u32`.");
                panic!();
            }
        };
    }

    pub fn sides(&self) -> u32 {
        return match self {
            Die::D4(_) => 4,
            Die::D6(_) => 6,
            Die::D8(_) => 8,
            Die::D10(_) => 10,
            Die::D12(_) => 12,
            Die::D20(_) => 20,
            Die::D100(_) => 100,
        };
    }

    pub fn value(&self) -> u32 {
        return match *self {
            Die::D4(v) => v,
            Die::D6(v) => v,
            Die::D8(v) => v,
            Die::D10(v) => v,
            Die::D12(v) => v,
            Die::D20(v) => v,
            Die::D100(v) => v,
        };
    }
}

impl fmt::Display for Die {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Die::D4(_) => write!(f, "d4"),
            Die::D6(_) => write!(f, "d6"),
            Die::D8(_) => write!(f, "d8"),
            Die::D10(_) => write!(f, "d10"),
            Die::D12(_) => write!(f, "d12"),
            Die::D20(_) => write!(f, "d20"),
            Die::D100(_) => write!(f, "d100"),
        }
    }
}

// NOTE: Advantage status for a dice
#[derive(Debug, Clone, Copy)]
pub enum Advantage {
    Dissadvantage,
    Normal,
    Advantage,
}

// NOTE: Result returned from the dice roll, contains
//       the every dice and their total value.
#[derive(Debug)]
pub struct DiceRollResult(Vec<Die>, i32);

// NOTE: Helper function that calculates the result of a single
//       dice roll, without considering the advantage.
fn roll_dice(count: u32, die: Die, bonus: i32) -> DiceRollResult {
    let mut rng = rand::thread_rng();

    let mut rolls = vec![];
    let mut total: i32 = 0;

    for _ in 0..count {
        let sides: u32 = die.sides();
        let result = rng.gen_range(0..sides);

        total += result as i32;
        rolls.push(Die::from_sides(sides, result));
    }

    return DiceRollResult(rolls, total + bonus);
}

// NOTE: A function that simulates dice roll.
pub fn roll(count: u32, die: Die, bonus: i32, advantage: Advantage) -> DiceRollResult {
    match advantage {
        Advantage::Dissadvantage => {
            let results = (
                roll_dice(count, die, bonus),
                roll_dice(count, die, bonus),
            );

            if results.0.1 < results.1.1 {
                results.0
            } else {
                results.1
            }
        },
        Advantage::Normal => {
            return roll_dice(count, die, bonus);
        },
        Advantage::Advantage => {
            let results = (
                roll_dice(count, die, bonus),
                roll_dice(count, die, bonus),
            );

            if results.0.1 > results.1.1 {
                results.0
            } else {
                results.1
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn roll_dice() {
        let count = 1;
        let die = Die::D20(0);
        let bonus = 5;
        let advantage = Advantage::Normal;

        let result = roll(count, die, bonus, advantage);

        assert!((result.1 <= count as i32 * die.sides() as i32 + 5));
    }
}