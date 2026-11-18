//! Вкладка «Случайный набор»: бросок с анимацией и распределение результатов.

use slint::ComponentHandle;

use super::board::{self, Board, SharedBoard};
use crate::rules::random::Sheet;
use crate::{AppWindow, RandomState};

pub fn install(ui: &AppWindow) {
    let board = Board::new(|ui, rolls, total| {
        let globals = ui.global::<RandomState>();
        globals.set_rolls(rolls);
        globals.set_grand_total(total);
    });
    roll(&board, ui);

    let globals = ui.global::<RandomState>();

    globals.on_roll({
        let ui = ui.as_weak();
        let board = board.clone();
        move || {
            if let Some(ui) = ui.upgrade() {
                roll(&board, &ui);
            }
        }
    });

    globals.on_assign({
        let ui = ui.as_weak();
        move |card, option| {
            if let Some(ui) = ui.upgrade() {
                board::assign(&board, &ui, card, option);
            }
        }
    });
}

fn roll(board: &SharedBoard, ui: &AppWindow) {
    let sheet = Sheet::roll(board.borrow_mut().rng());
    board::show(board, ui, sheet);
}
