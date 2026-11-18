//! Правила генерации характеристик, независимые от интерфейса.

pub mod ability;
pub mod assignment;
pub mod point_buy;
pub mod random;
pub mod standard;
pub mod threshold;

pub use ability::{ABILITY_COUNT, Ability, modifier};
