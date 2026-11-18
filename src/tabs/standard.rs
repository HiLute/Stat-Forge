//! Вкладка «Стандартный набор»: распределение 15, 14, 13, 12, 10, 8
//! вручную или по шаблону класса.

use std::{cell::RefCell, rc::Rc};

use slint::{ComponentHandle, SharedString};

use super::{model, modifier_text};
use crate::rules::{
    Ability,
    standard::{CLASS_TEMPLATES, STANDARD_ARRAY, StandardArray},
};
use crate::{AppWindow, StandardCard, StandardState};

const UNSELECTED: &str = "Не выбрано";

pub fn install(ui: &AppWindow) {
    let state = Rc::new(RefCell::new(StandardArray::default()));
    let globals = ui.global::<StandardState>();

    let values: Vec<String> = STANDARD_ARRAY.iter().map(u8::to_string).collect();
    globals.set_values_text(format!("{}.", values.join(", ")).into());
    globals.set_classes(model(
        CLASS_TEMPLATES.iter().map(|t| SharedString::from(t.name)).collect(),
    ));
    render(ui, &state.borrow());

    globals.on_assign({
        let ui = ui.as_weak();
        let state = state.clone();
        move |ability, option| {
            let Some(ui) = ui.upgrade() else { return };
            let mut state = state.borrow_mut();
            let ability = ability as usize;
            if let Some(&choice) = state.options(ability).get(option as usize) {
                state.assign(ability, choice);
            }
            render(&ui, &state);
        }
    });

    globals.on_apply_template({
        let ui = ui.as_weak();
        move |index| {
            let Some(ui) = ui.upgrade() else { return };
            let Some(template) = CLASS_TEMPLATES.get(index as usize) else { return };
            let mut state = state.borrow_mut();
            state.apply(template);
            render(&ui, &state);
        }
    });
}

fn render(ui: &AppWindow, state: &StandardArray) {
    let cards: Vec<StandardCard> = Ability::ALL
        .iter()
        .enumerate()
        .map(|(i, ability)| {
            let options = state.options(i);
            let score = state.score(i);
            StandardCard {
                name: ability.name().into(),
                modifier: score.map(|s| modifier_text(i32::from(s))).unwrap_or_default(),
                selected: options.iter().position(|&o| o == score).unwrap_or(0) as i32,
                options: model(
                    options
                        .iter()
                        .map(|o| o.map_or(UNSELECTED.into(), |v| v.to_string().into()))
                        .collect(),
                ),
            }
        })
        .collect();
    ui.global::<StandardState>().set_cards(model(cards));
}
