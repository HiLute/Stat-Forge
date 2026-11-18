/// Количество характеристик персонажа.
pub const ABILITY_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Ability {
    pub const ALL: [Ability; ABILITY_COUNT] = [
        Ability::Strength,
        Ability::Dexterity,
        Ability::Constitution,
        Ability::Intelligence,
        Ability::Wisdom,
        Ability::Charisma,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Ability::Strength => "Сила",
            Ability::Dexterity => "Ловкость",
            Ability::Constitution => "Телосложение",
            Ability::Intelligence => "Интеллект",
            Ability::Wisdom => "Мудрость",
            Ability::Charisma => "Харизма",
        }
    }
}

/// Модификатор характеристики по правилам D&D 5e: ⌊(значение − 10) / 2⌋.
pub fn modifier(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers() {
        assert_eq!(modifier(3), -4);
        assert_eq!(modifier(8), -1);
        assert_eq!(modifier(9), -1);
        assert_eq!(modifier(10), 0);
        assert_eq!(modifier(11), 0);
        assert_eq!(modifier(14), 2);
        assert_eq!(modifier(18), 4);
    }
}
