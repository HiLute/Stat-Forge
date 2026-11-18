//! Набор из шести бросков на экране: анимация броска и распределение
//! результатов по характеристикам. Общая часть вкладок «Случайный набор»
//! и «Эксперименты»; куда выводить карточки, решает функция `publish`.

use std::{cell::RefCell, rc::Rc, time::Duration};

use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode};

use super::{model, modifier_text};
use crate::rules::{
    Ability,
    random::{Roll, Sheet},
};
use crate::{AppWindow, DieData, RollData};

const UNASSIGNED: &str = "Не назначено";

/// Интервал смены граней во время броска.
const TICK: Duration = Duration::from_millis(60);
/// На каком тике останавливается первая карточка и шаг до следующей:
/// карточки «приземляются» по очереди.
const SETTLE_FIRST: usize = 6;
const SETTLE_STEP: usize = 2;
/// Максимальный наклон кубика во время броска, в градусах.
const MAX_TILT: f32 = 35.0;

/// Передаёт карточки и общую сумму в глобальное состояние нужной вкладки.
pub type Publish = fn(&AppWindow, ModelRc<RollData>, i32);

pub struct Board {
    sheet: Option<Sheet>,
    rng: fastrand::Rng,
    /// Тиков с начала текущей анимации броска.
    tick: usize,
    timer: Timer,
    publish: Publish,
}

pub type SharedBoard = Rc<RefCell<Board>>;

impl Board {
    pub fn new(publish: Publish) -> SharedBoard {
        Rc::new(RefCell::new(Self {
            sheet: None,
            rng: fastrand::Rng::new(),
            tick: 0,
            timer: Timer::default(),
            publish,
        }))
    }

    pub fn rng(&mut self) -> &mut fastrand::Rng {
        &mut self.rng
    }

    fn settled(&self, idx: usize) -> bool {
        self.tick >= SETTLE_FIRST + idx * SETTLE_STEP
    }

    fn all_settled(&self) -> bool {
        let count = self.sheet.as_ref().map_or(0, |s| s.rolls().len());
        (0..count).all(|i| self.settled(i))
    }

    fn card(&mut self, idx: usize) -> Option<RollData> {
        let rolling = !self.settled(idx);
        let sheet = self.sheet.as_ref()?;
        // Пока карточка «катится», показываем случайные грани, а не итог.
        let shown = if rolling {
            Roll::random(&mut self.rng)
        } else {
            sheet.rolls()[idx].clone()
        };
        let options = sheet.options(idx);
        let assigned = sheet.assigned(idx);

        let dice: Vec<DieData> = shown
            .dice
            .iter()
            .map(|&d| DieData {
                value: i32::from(d),
                angle: if rolling {
                    (self.rng.f32() * 2.0 - 1.0) * MAX_TILT
                } else {
                    0.0
                },
            })
            .collect();
        let labels: Vec<SharedString> = options
            .iter()
            .map(|o| o.map_or(UNASSIGNED, Ability::name).into())
            .collect();

        Some(RollData {
            total: shown.total() as i32,
            modifier: modifier_text(shown.total() as i32),
            dice: model(dice),
            dropped: if rolling { -1 } else { shown.dropped as i32 },
            rolling,
            selected: options.iter().position(|&o| o == assigned).unwrap_or(0) as i32,
            options: model(labels),
        })
    }

    fn render(&mut self, ui: &AppWindow) {
        let count = self.sheet.as_ref().map_or(0, |s| s.rolls().len());
        let cards: Vec<RollData> = (0..count).filter_map(|i| self.card(i)).collect();
        let total = cards.iter().map(|c| c.total).sum();
        (self.publish)(ui, model(cards), total);
    }
}

/// Показывает новый набор и запускает анимацию: таймер меняет грани,
/// пока все карточки не остановятся.
pub fn show(board: &SharedBoard, ui: &AppWindow, sheet: Sheet) {
    let mut this = board.borrow_mut();
    this.sheet = Some(sheet);
    this.tick = 0;
    this.render(ui);

    let board = Rc::downgrade(board);
    let ui = ui.as_weak();
    this.timer.start(TimerMode::Repeated, TICK, move || {
        let (Some(board), Some(ui)) = (board.upgrade(), ui.upgrade()) else { return };
        let mut this = board.borrow_mut();
        this.tick += 1;
        this.render(&ui);
        if this.all_settled() {
            this.timer.stop();
        }
    });
}

/// Убирает набор с экрана.
pub fn clear(board: &SharedBoard, ui: &AppWindow) {
    let mut this = board.borrow_mut();
    this.timer.stop();
    this.sheet = None;
    this.render(ui);
}

/// Назначает броску `card` характеристику из его списка вариантов.
pub fn assign(board: &SharedBoard, ui: &AppWindow, card: i32, option: i32) {
    let mut this = board.borrow_mut();
    let card = card as usize;
    let Some(sheet) = this.sheet.as_mut() else { return };
    if let Some(&choice) = sheet.options(card).get(option as usize) {
        sheet.assign(card, choice);
    }
    this.render(ui);
}
