//! Взаимно-однозначное распределение: каждому слоту можно назначить один
//! элемент из набора, и каждый элемент занимает не больше одного слота.
//! Используется и для «бросок → характеристика», и для «характеристика → значение
//! стандартного набора».

use std::iter;

#[derive(Clone, Debug)]
pub struct Assignment<T> {
    items: Vec<T>,
    slots: Vec<Option<T>>,
}

impl<T: Copy + PartialEq> Assignment<T> {
    pub fn new(items: impl Into<Vec<T>>, slot_count: usize) -> Self {
        Self {
            items: items.into(),
            slots: vec![None; slot_count],
        }
    }

    pub fn get(&self, slot: usize) -> Option<T> {
        self.slots.get(slot).copied().flatten()
    }

    /// Варианты для слота: «ничего», затем элементы, не занятые другими слотами.
    pub fn options(&self, slot: usize) -> Vec<Option<T>> {
        let taken_elsewhere = |item: T| {
            self.slots
                .iter()
                .enumerate()
                .any(|(j, s)| j != slot && *s == Some(item))
        };
        iter::once(None)
            .chain(
                self.items
                    .iter()
                    .copied()
                    .filter(|&item| !taken_elsewhere(item))
                    .map(Some),
            )
            .collect()
    }

    /// Назначает элемент слоту. Если элемент был в другом слоте,
    /// тот освобождается, — элемент всегда занимает один слот.
    pub fn assign(&mut self, slot: usize, choice: Option<T>) {
        if slot >= self.slots.len() {
            return;
        }
        if let Some(item) = choice {
            if !self.items.contains(&item) {
                return;
            }
            for s in self.slots.iter_mut().filter(|s| **s == Some(item)) {
                *s = None;
            }
        }
        self.slots[slot] = choice;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_exclude_items_taken_elsewhere() {
        let mut a = Assignment::new(['a', 'b', 'c'], 3);
        a.assign(0, Some('a'));

        assert_eq!(a.options(0), [None, Some('a'), Some('b'), Some('c')]);
        assert_eq!(a.options(1), [None, Some('b'), Some('c')]);
    }

    #[test]
    fn assigning_taken_item_moves_it() {
        let mut a = Assignment::new(['a', 'b'], 2);
        a.assign(0, Some('b'));
        a.assign(1, Some('b'));
        assert_eq!(a.get(0), None);
        assert_eq!(a.get(1), Some('b'));

        a.assign(1, None);
        assert_eq!(a.get(1), None);
    }

    #[test]
    fn ignores_unknown_items_and_slots() {
        let mut a = Assignment::new(['a'], 1);
        a.assign(0, Some('z'));
        a.assign(5, Some('a'));
        assert_eq!(a.get(0), None);
        assert_eq!(a.get(5), None);
    }
}
