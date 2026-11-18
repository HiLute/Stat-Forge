//! Случайный набор: броски «4d6, отбросить наименьший»
//! и распределение результатов по характеристикам.

use std::cmp::Reverse;

use super::{ABILITY_COUNT, Ability, assignment::Assignment};

/// Один бросок 4d6: все четыре кубика и индекс отброшенного (наименьшего).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roll {
    pub dice: [u8; 4],
    pub dropped: usize,
}

impl Roll {
    pub fn from_dice(dice: [u8; 4]) -> Self {
        // При равенстве отбрасывается первый из наименьших.
        let dropped = (0..dice.len()).min_by_key(|&i| dice[i]).unwrap_or(0);
        Self { dice, dropped }
    }

    pub fn random(rng: &mut fastrand::Rng) -> Self {
        Self::from_dice(std::array::from_fn(|_| rng.u8(1..=6)))
    }

    pub fn total(&self) -> u32 {
        let sum: u32 = self.dice.iter().map(|&d| u32::from(d)).sum();
        sum - u32::from(self.dice[self.dropped])
    }
}

/// Шесть бросков и характеристики, назначенные каждому из них.
#[derive(Clone, Debug)]
pub struct Sheet {
    rolls: Vec<Roll>,
    abilities: Assignment<Ability>,
}

impl Sheet {
    /// Бросает шесть характеристик; результаты отсортированы по убыванию.
    pub fn roll(rng: &mut fastrand::Rng) -> Self {
        Self::from_rolls((0..ABILITY_COUNT).map(|_| Roll::random(rng)).collect())
    }

    pub fn from_rolls(mut rolls: Vec<Roll>) -> Self {
        rolls.sort_by_key(|r| Reverse(r.total()));
        let abilities = Assignment::new(Ability::ALL, rolls.len());
        Self { rolls, abilities }
    }

    pub fn rolls(&self) -> &[Roll] {
        &self.rolls
    }

    pub fn assigned(&self, idx: usize) -> Option<Ability> {
        self.abilities.get(idx)
    }

    pub fn options(&self, idx: usize) -> Vec<Option<Ability>> {
        self.abilities.options(idx)
    }

    pub fn assign(&mut self, idx: usize, ability: Option<Ability>) {
        self.abilities.assign(idx, ability);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_lowest_die() {
        let roll = Roll::from_dice([5, 1, 4, 5]);
        assert_eq!(roll.dropped, 1);
        assert_eq!(roll.total(), 14);
    }

    #[test]
    fn drops_first_of_equal_lowest() {
        let roll = Roll::from_dice([6, 3, 3, 3]);
        assert_eq!(roll.dropped, 1);
        assert_eq!(roll.total(), 12);
    }

    #[test]
    fn random_rolls_stay_in_range() {
        let mut rng = fastrand::Rng::with_seed(42);
        for _ in 0..10_000 {
            let roll = Roll::random(&mut rng);
            assert!(roll.dice.iter().all(|d| (1..=6).contains(d)));
            assert!((3..=18).contains(&roll.total()));
        }
    }

    #[test]
    fn sheet_sorted_descending_and_assignable() {
        let mut sheet = Sheet::from_rolls(vec![
            Roll::from_dice([1, 1, 1, 1]),
            Roll::from_dice([6, 6, 6, 6]),
            Roll::from_dice([3, 3, 3, 3]),
        ]);
        let totals: Vec<u32> = sheet.rolls().iter().map(Roll::total).collect();
        assert_eq!(totals, [18, 9, 3]);

        sheet.assign(0, Some(Ability::Wisdom));
        assert_eq!(sheet.assigned(0), Some(Ability::Wisdom));
        assert!(!sheet.options(1).contains(&Some(Ability::Wisdom)));
    }
}
