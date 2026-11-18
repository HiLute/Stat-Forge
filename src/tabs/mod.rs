//! Связь правил из `rules` с интерфейсом: у каждой вкладки свой модуль
//! с функцией `install`, которая подключает её глобальное состояние в Slint.

mod board;
pub mod experiments;
pub mod point_buy;
pub mod random;
pub mod standard;

use slint::{ModelRc, SharedString, VecModel};

use crate::rules::modifier;

pub fn model<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items))
}

/// Модификатор значения характеристики в виде «+2» / «−1».
pub fn modifier_text(score: i32) -> SharedString {
    let value = modifier(score);
    if value < 0 {
        format!("−{}", -value).into()
    } else {
        format!("+{value}").into()
    }
}

/// Число с разделением разрядов: 1234567 → «1 234 567».
pub fn group_digits(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() * 2);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('\u{202F}'); // узкий неразрывный пробел
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_digits_by_three() {
        assert_eq!(group_digits(0), "0");
        assert_eq!(group_digits(999), "999");
        assert_eq!(group_digits(1_000), "1\u{202F}000");
        assert_eq!(group_digits(1_234_567), "1\u{202F}234\u{202F}567");
    }

    #[test]
    fn modifier_text_uses_sign() {
        assert_eq!(modifier_text(8), "−1");
        assert_eq!(modifier_text(10), "+0");
        assert_eq!(modifier_text(15), "+2");
    }
}
