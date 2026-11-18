//! Стандартный набор значений и рекомендованные распределения по классам.

use super::{ABILITY_COUNT, assignment::Assignment};

pub const STANDARD_ARRAY: [u8; ABILITY_COUNT] = [15, 14, 13, 12, 10, 8];

/// Рекомендованное распределение стандартного набора для класса.
/// Значения идут в порядке `Ability::ALL`: СИЛ, ЛОВ, ТЕЛ, ИНТ, МДР, ХАР.
pub struct ClassTemplate {
    pub name: &'static str,
    pub scores: [u8; ABILITY_COUNT],
}

/// Шаблоны из «Книги игрока» D&D 2024.
pub const CLASS_TEMPLATES: [ClassTemplate; 12] = [
    ClassTemplate {
        name: "Бард",
        scores: [8, 14, 12, 13, 10, 15],
    },
    ClassTemplate {
        name: "Варвар",
        scores: [15, 13, 14, 10, 12, 8],
    },
    ClassTemplate {
        name: "Воин",
        scores: [15, 14, 13, 8, 10, 12],
    },
    ClassTemplate {
        name: "Волшебник",
        scores: [8, 12, 13, 15, 14, 10],
    },
    ClassTemplate {
        name: "Друид",
        scores: [8, 12, 14, 13, 15, 10],
    },
    ClassTemplate {
        name: "Жрец",
        scores: [14, 8, 13, 10, 15, 12],
    },
    ClassTemplate {
        name: "Колдун",
        scores: [8, 14, 13, 12, 10, 15],
    },
    ClassTemplate {
        name: "Монах",
        scores: [12, 15, 13, 10, 14, 8],
    },
    ClassTemplate {
        name: "Паладин",
        scores: [15, 10, 13, 8, 12, 14],
    },
    ClassTemplate {
        name: "Плут",
        scores: [12, 15, 13, 14, 10, 8],
    },
    ClassTemplate {
        name: "Следопыт",
        scores: [12, 15, 13, 8, 14, 10],
    },
    ClassTemplate {
        name: "Чародей",
        scores: [10, 13, 14, 8, 12, 15],
    },
];

/// Распределение значений стандартного набора по характеристикам
/// (слот — индекс характеристики в `Ability::ALL`).
#[derive(Clone, Debug)]
pub struct StandardArray {
    values: Assignment<u8>,
}

impl Default for StandardArray {
    fn default() -> Self {
        Self {
            values: Assignment::new(STANDARD_ARRAY, ABILITY_COUNT),
        }
    }
}

impl StandardArray {
    pub fn score(&self, ability: usize) -> Option<u8> {
        self.values.get(ability)
    }

    pub fn options(&self, ability: usize) -> Vec<Option<u8>> {
        self.values.options(ability)
    }

    pub fn assign(&mut self, ability: usize, value: Option<u8>) {
        self.values.assign(ability, value);
    }

    pub fn apply(&mut self, template: &ClassTemplate) {
        for (ability, &value) in template.scores.iter().enumerate() {
            self.values.assign(ability, Some(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_permutations_of_standard_array() {
        let mut expected = STANDARD_ARRAY;
        expected.sort_unstable();
        for template in &CLASS_TEMPLATES {
            let mut scores = template.scores;
            scores.sort_unstable();
            assert_eq!(scores, expected, "шаблон «{}»", template.name);
        }
    }

    #[test]
    fn apply_template_fills_every_ability() {
        let mut array = StandardArray::default();
        array.assign(0, Some(8));
        let wizard = &CLASS_TEMPLATES[3];
        array.apply(wizard);
        for (i, &value) in wizard.scores.iter().enumerate() {
            assert_eq!(array.score(i), Some(value));
        }
    }

    #[test]
    fn values_are_unique() {
        let mut array = StandardArray::default();
        array.assign(0, Some(15));
        array.assign(1, Some(15));
        assert_eq!(array.score(0), None);
        assert_eq!(array.options(2).len(), STANDARD_ARRAY.len());
    }
}
