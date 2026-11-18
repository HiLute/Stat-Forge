//! Вкладка «Покупка значений»: характеристики покупаются за очки бюджета.

use std::{cell::RefCell, rc::Rc};

use slint::ComponentHandle;

use super::{model, modifier_text};
use crate::rules::{
    Ability,
    point_buy::{MAX_BUDGET, MAX_SCORE, MIN_SCORE, PointBuy, cost},
};
use crate::{AppWindow, PointBuyCard, PointBuyState};

pub fn install(ui: &AppWindow) {
    let state = Rc::new(RefCell::new(PointBuy::default()));
    let globals = ui.global::<PointBuyState>();

    globals.set_min_score(MIN_SCORE.into());
    globals.set_max_score(MAX_SCORE.into());
    globals.set_max_budget(MAX_BUDGET as i32);
    render(ui, &state.borrow());

    // Все колбэки вкладки устроены одинаково: изменить состояние и перерисовать.
    let on_change = |action: fn(&mut PointBuy, i32, i32)| {
        let ui = ui.as_weak();
        let state = state.clone();
        move |a: i32, b: i32| {
            let Some(ui) = ui.upgrade() else { return };
            let mut state = state.borrow_mut();
            action(&mut state, a, b);
            render(&ui, &state);
        }
    };

    globals.on_change(on_change(|pb, ability, delta| {
        let ability = ability as usize;
        if ability >= Ability::ALL.len() {
            return;
        }
        if delta > 0 {
            pb.increase(ability);
        } else {
            pb.decrease(ability);
        }
    }));

    let set_budget = on_change(|pb, budget, _| pb.set_budget(budget.max(0) as u32));
    globals.on_set_budget(move |budget| set_budget(budget, 0));

    let reset = on_change(|pb, _, _| pb.reset());
    globals.on_reset(move || reset(0, 0));
}

fn render(ui: &AppWindow, state: &PointBuy) {
    let cards: Vec<PointBuyCard> = Ability::ALL
        .iter()
        .enumerate()
        .map(|(i, ability)| {
            let score = state.score(i);
            PointBuyCard {
                name: ability.name().into(),
                score: score.into(),
                cost: cost(score) as i32,
                modifier: modifier_text(score.into()),
                can_increase: state.can_increase(i),
                can_decrease: state.can_decrease(i),
            }
        })
        .collect();

    let globals = ui.global::<PointBuyState>();
    globals.set_budget(state.budget() as i32);
    globals.set_remaining(state.remaining() as i32);
    globals.set_cards(model(cards));
}
