//! Покупка значений характеристик за очки (D&D 5e).

use super::ABILITY_COUNT;

pub const MIN_SCORE: u8 = 8;
pub const MAX_SCORE: u8 = 15;
pub const DEFAULT_BUDGET: u32 = 27;
/// Бюджет, при котором все характеристики можно поднять до максимума.
pub const MAX_BUDGET: u32 = ABILITY_COUNT as u32 * cost(MAX_SCORE);

/// Суммарная стоимость значения: 8 → 0, …, 13 → 5, 14 → 7, 15 → 9.
pub const fn cost(score: u8) -> u32 {
    match score {
        0..=MIN_SCORE => 0,
        9..=13 => (score - MIN_SCORE) as u32,
        14 => 7,
        _ => 9,
    }
}

#[derive(Clone, Debug)]
pub struct PointBuy {
    scores: [u8; ABILITY_COUNT],
    budget: u32,
}

impl Default for PointBuy {
    fn default() -> Self {
        Self {
            scores: [MIN_SCORE; ABILITY_COUNT],
            budget: DEFAULT_BUDGET,
        }
    }
}

impl PointBuy {
    pub fn score(&self, ability: usize) -> u8 {
        self.scores[ability]
    }

    pub fn budget(&self) -> u32 {
        self.budget
    }

    pub fn spent(&self) -> u32 {
        self.scores.iter().map(|&s| cost(s)).sum()
    }

    /// Остаток очков; отрицательный, если бюджет уменьшили ниже потраченного.
    pub fn remaining(&self) -> i64 {
        i64::from(self.budget) - i64::from(self.spent())
    }

    pub fn can_increase(&self, ability: usize) -> bool {
        let score = self.scores[ability];
        score < MAX_SCORE && i64::from(cost(score + 1) - cost(score)) <= self.remaining()
    }

    pub fn can_decrease(&self, ability: usize) -> bool {
        self.scores[ability] > MIN_SCORE
    }

    pub fn increase(&mut self, ability: usize) {
        if self.can_increase(ability) {
            self.scores[ability] += 1;
        }
    }

    pub fn decrease(&mut self, ability: usize) {
        if self.can_decrease(ability) {
            self.scores[ability] -= 1;
        }
    }

    pub fn set_budget(&mut self, budget: u32) {
        self.budget = budget.min(MAX_BUDGET);
    }

    pub fn reset(&mut self) {
        self.scores = [MIN_SCORE; ABILITY_COUNT];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_table() {
        let costs: Vec<u32> = (MIN_SCORE..=MAX_SCORE).map(cost).collect();
        assert_eq!(costs, [0, 1, 2, 3, 4, 5, 7, 9]);
        assert_eq!(MAX_BUDGET, 54);
    }

    #[test]
    fn spends_exact_budget() {
        let mut pb = PointBuy::default();
        // 15, 15, 15 = 27 очков.
        for ability in 0..3 {
            for _ in MIN_SCORE..MAX_SCORE {
                pb.increase(ability);
            }
        }
        assert_eq!([pb.score(0), pb.score(1), pb.score(2)], [15, 15, 15]);
        assert_eq!(pb.remaining(), 0);
        assert!(!pb.can_increase(3));
        pb.increase(3);
        assert_eq!(pb.score(3), MIN_SCORE);
    }

    #[test]
    fn respects_score_bounds() {
        let mut pb = PointBuy::default();
        pb.decrease(0);
        assert_eq!(pb.score(0), MIN_SCORE);
        pb.set_budget(MAX_BUDGET);
        for _ in 0..20 {
            pb.increase(0);
        }
        assert_eq!(pb.score(0), MAX_SCORE);
    }

    #[test]
    fn step_cost_must_fit_remaining() {
        let mut pb = PointBuy::default();
        pb.set_budget(6);
        for _ in 0..5 {
            pb.increase(0); // 13 за 5 очков
        }
        assert_eq!(pb.remaining(), 1);
        assert!(!pb.can_increase(0)); // 13 → 14 стоит 2
        assert!(pb.can_increase(1));
    }

    #[test]
    fn lowering_budget_can_go_negative_and_reset_restores() {
        let mut pb = PointBuy::default();
        pb.increase(0);
        pb.increase(0);
        pb.set_budget(0);
        assert_eq!(pb.remaining(), -2);
        pb.reset();
        assert_eq!(pb.remaining(), 0);
        assert_eq!(pb.spent(), 0);
    }
}
