//! Вкладка «Эксперименты»: наборы бросаются, пока сумма характеристик
//! не достигнет заданного минимума или пока пользователь не остановит поиск.

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use slint::{ComponentHandle, Timer, TimerMode};

use super::{
    board::{self, Board},
    group_digits,
};
use crate::rules::threshold::{self, Control, MAX_TOTAL, MIN_TOTAL, Search};
use crate::{AppWindow, ExperimentState};

/// Как часто обновлять счётчик брошенных наборов во время поиска.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(200);

/// Идущий фоновый поиск.
struct Running {
    result: mpsc::Receiver<Search>,
    control: Arc<Control>,
    /// Обновляет счётчик в интерфейсе; останавливается вместе с `Running`.
    _progress_timer: Timer,
}

pub fn install(ui: &AppWindow) {
    let board = Board::new(|ui, rolls, total| {
        let globals = ui.global::<ExperimentState>();
        globals.set_rolls(rolls);
        globals.set_grand_total(total);
    });
    let running: Rc<RefCell<Option<Running>>> = Rc::default();

    let globals = ui.global::<ExperimentState>();
    globals.set_min_limit(MIN_TOTAL as i32);
    globals.set_max_limit(MAX_TOTAL as i32);
    globals.set_status("Укажите минимальную сумму и нажмите «Бросок».".into());

    globals.on_search({
        let ui = ui.as_weak();
        let running = running.clone();
        move |min_total| {
            let Some(ui) = ui.upgrade() else { return };
            if running.borrow().is_some() {
                return;
            }
            let (tx, rx) = mpsc::channel();
            let control = Arc::new(Control::new());

            let globals = ui.global::<ExperimentState>();
            globals.set_searching(true);
            globals.set_status("Бросаем…".into());

            let progress_timer = Timer::default();
            progress_timer.start(TimerMode::Repeated, PROGRESS_INTERVAL, {
                let ui = ui.as_weak();
                let control = control.clone();
                move || {
                    if let Some(ui) = ui.upgrade() {
                        let attempts = control.attempts();
                        ui.global::<ExperimentState>().set_status(
                            format!("Бросаем… брошено наборов: {}", group_digits(attempts)).into(),
                        );
                    }
                }
            });

            // Поиск может занять миллионы наборов: он идёт на всех ядрах,
            // а этот поток только ждёт его, чтобы интерфейс не замирал.
            let min_total = min_total.clamp(MIN_TOTAL as i32, MAX_TOTAL as i32) as u32;
            let ui = ui.as_weak();
            let search_control = control.clone();
            thread::spawn(move || {
                let result = threshold::search(min_total, &search_control);
                let _ = tx.send(result);
                let _ = ui.upgrade_in_event_loop(|ui| {
                    ui.global::<ExperimentState>().invoke_search_finished();
                });
            });

            *running.borrow_mut() = Some(Running {
                result: rx,
                control,
                _progress_timer: progress_timer,
            });
        }
    });

    globals.on_stop({
        let running = running.clone();
        move || {
            if let Some(running) = running.borrow().as_ref() {
                running.control.stop();
            }
        }
    });

    globals.on_search_finished({
        let ui = ui.as_weak();
        let board = board.clone();
        move || {
            let Some(ui) = ui.upgrade() else { return };
            let Some(result) = running.borrow_mut().take().and_then(|r| r.result.recv().ok()) else {
                return;
            };
            let globals = ui.global::<ExperimentState>();
            globals.set_searching(false);
            match result {
                Search::Found { sheet, attempts } => {
                    globals.set_status(
                        format!("Брошено наборов: {}", group_digits(attempts)).into(),
                    );
                    board::show(&board, &ui, sheet);
                }
                Search::Stopped { attempts } => {
                    globals.set_status(
                        format!("Остановлено. Брошено наборов: {}", group_digits(attempts)).into(),
                    );
                    board::clear(&board, &ui);
                }
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
