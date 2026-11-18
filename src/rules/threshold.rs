//! Поиск набора характеристик с минимальной суммой: наборы «4d6, отбросить
//! наименьший» бросаются, пока сумма шести значений не станет не меньше
//! заданной.
//!
//! Поиск идёт на всех ядрах. Попытки пронумерованы глобально и раздаются
//! потокам блоками; ответом считается успешная попытка с наименьшим номером.
//! Поэтому число брошенных наборов такое же, как при последовательном поиске,
//! а генератор случайных чисел каждого блока выводится из общего зерна —
//! результат не зависит от числа потоков.
//!
//! Внутренний цикл не выделяет память: бросок 4d6 — одно случайное число
//! 0..1296 и поиск в таблице на 1296 байт, набор живёт на стеке.

use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
};

use super::{
    ABILITY_COUNT,
    random::{Roll, Sheet},
};

/// Наименьшая и наибольшая возможная сумма шести бросков.
pub const MIN_TOTAL: u32 = 3 * ABILITY_COUNT as u32;
pub const MAX_TOTAL: u32 = 18 * ABILITY_COUNT as u32;

/// Сколько попыток поток берёт за раз. Между блоками проверяется флаг
/// остановки и обновляется счётчик — это доли миллисекунды работы.
const BLOCK: u64 = 4096;

/// Число исходов 4d6. Исход кодирует четыре кубика в шестеричной записи.
const OUTCOMES: u32 = 6 * 6 * 6 * 6;

/// Сумма трёх наибольших кубиков для каждого исхода 4d6.
static TOTALS: [u8; OUTCOMES as usize] = build_totals();

/// Четыре кубика (1..=6) исхода `outcome`.
const fn decode(outcome: u32) -> [u8; 4] {
    [
        (outcome % 6) as u8 + 1,
        (outcome / 6 % 6) as u8 + 1,
        (outcome / 36 % 6) as u8 + 1,
        (outcome / 216 % 6) as u8 + 1,
    ]
}

const fn build_totals() -> [u8; OUTCOMES as usize] {
    let mut totals = [0; OUTCOMES as usize];
    let mut outcome = 0;
    while outcome < OUTCOMES {
        let d = decode(outcome);
        let mut lowest = d[0];
        let mut i = 1;
        while i < 4 {
            if d[i] < lowest {
                lowest = d[i];
            }
            i += 1;
        }
        totals[outcome as usize] = d[0] + d[1] + d[2] + d[3] - lowest;
        outcome += 1;
    }
    totals
}

/// Управление идущим поиском из другого потока: остановка и прогресс.
#[derive(Debug, Default)]
pub struct Control {
    stop: AtomicBool,
    attempts: AtomicU64,
}

impl Control {
    pub fn new() -> Self {
        Self::default()
    }

    /// Просит поиск остановиться; он завершится в течение блока попыток.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    /// Сколько наборов брошено на данный момент (обновляется блоками).
    pub fn attempts(&self) -> u64 {
        self.attempts.load(Ordering::Relaxed)
    }
}

/// Итог поиска.
#[derive(Debug)]
pub enum Search {
    /// `attempts` — сколько наборов понадобилось, включая найденный.
    Found { sheet: Sheet, attempts: u64 },
    /// Поиск остановлен до того, как подходящий набор выпал.
    Stopped { attempts: u64 },
}

/// Бросает наборы на всех доступных ядрах, пока сумма не станет не меньше
/// `min_total` или пока не вызван `control.stop()`. Блокирует вызывающий
/// поток до конца поиска.
pub fn search(min_total: u32, control: &Control) -> Search {
    let threads = thread::available_parallelism().map_or(1, NonZeroUsize::get);
    search_with(min_total, threads, fastrand::u64(..), control)
}

/// То же, что [`search`], с явным числом потоков и зерном. При одинаковом
/// зерне результат одинаков для любого числа потоков.
pub fn search_with(min_total: u32, threads: usize, seed: u64, control: &Control) -> Search {
    if min_total > MAX_TOTAL {
        return Search::Stopped { attempts: 0 };
    }
    let next_block = AtomicU64::new(0);
    // Наименьший номер успешной попытки среди найденных.
    let best = AtomicU64::new(u64::MAX);

    let winner = thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.max(1))
            .map(|_| scope.spawn(|| worker(min_total, seed, &next_block, &best, control)))
            .collect();
        workers
            .into_iter()
            .filter_map(|w| w.join().expect("поток поиска завершился с паникой"))
            .min_by_key(|&(index, _)| index)
    });

    match winner {
        Some((index, outcomes)) => Search::Found {
            sheet: Sheet::from_rolls(outcomes.map(|o| Roll::from_dice(decode(o.into()))).into()),
            attempts: index + 1,
        },
        None => Search::Stopped { attempts: control.attempts() },
    }
}

/// Берёт блоки попыток, пока они идут раньше лучшего найденного успеха.
/// Возвращает свою успешную попытку: номер и исходы шести бросков.
fn worker(
    min_total: u32,
    seed: u64,
    next_block: &AtomicU64,
    best: &AtomicU64,
    control: &Control,
) -> Option<(u64, [u16; ABILITY_COUNT])> {
    let mut found = None;
    while !control.is_stopped() {
        let block = next_block.fetch_add(1, Ordering::Relaxed);
        let start = block * BLOCK;
        if start >= best.load(Ordering::Relaxed) {
            break;
        }
        let mut rng = fastrand::Rng::with_seed(seed ^ block.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut done = 0;
        for index in start..start + BLOCK {
            if index >= best.load(Ordering::Relaxed) {
                break;
            }
            done += 1;
            if let Some(outcomes) = roll_set(&mut rng, min_total) {
                best.fetch_min(index, Ordering::Relaxed);
                found = Some((index, outcomes));
                break;
            }
        }
        control.attempts.fetch_add(done, Ordering::Relaxed);
    }
    found
}

/// Бросает один набор. Возвращает исходы бросков, если сумма достигла
/// `min_total`. Бросок прекращается досрочно, когда даже восемнадцатки
/// в оставшихся характеристиках не дотянут до порога, — набор всё равно
/// неудачный, а на высоких порогах это экономит большую часть работы.
#[inline]
fn roll_set(rng: &mut fastrand::Rng, min_total: u32) -> Option<[u16; ABILITY_COUNT]> {
    let mut outcomes = [0; ABILITY_COUNT];
    let mut sum = 0;
    for (i, slot) in outcomes.iter_mut().enumerate() {
        let outcome = rng.u32(..OUTCOMES);
        *slot = outcome as u16;
        sum += u32::from(TOTALS[outcome as usize]);
        let remaining = (ABILITY_COUNT - 1 - i) as u32;
        if sum + 18 * remaining < min_total {
            return None;
        }
    }
    Some(outcomes)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn total(sheet: &Sheet) -> u32 {
        sheet.rolls().iter().map(Roll::total).sum()
    }

    #[test]
    fn totals_table_matches_4d6_drop_lowest() {
        for outcome in 0..OUTCOMES {
            let roll = Roll::from_dice(decode(outcome));
            assert_eq!(u32::from(TOTALS[outcome as usize]), roll.total());
        }
        let count = |t: u8| TOTALS.iter().filter(|&&x| x == t).count();
        assert_eq!(count(3), 1);
        assert_eq!(count(18), 21);
        // Матожидание 4d6 без наименьшего — 15869 / 1296 ≈ 12,24.
        assert_eq!(TOTALS.iter().map(|&t| u32::from(t)).sum::<u32>(), 15869);
    }

    #[test]
    fn minimum_total_succeeds_first_try() {
        match search_with(MIN_TOTAL, 4, 1, &Control::new()) {
            Search::Found { sheet, attempts } => {
                assert_eq!(attempts, 1);
                assert_eq!(sheet.rolls().len(), ABILITY_COUNT);
            }
            other => panic!("ожидался набор, получено {other:?}"),
        }
    }

    #[test]
    fn reaches_requested_total() {
        for min_total in [60, 75, 85, 90] {
            match search_with(min_total, 4, 7, &Control::new()) {
                Search::Found { sheet, attempts } => {
                    assert!(total(&sheet) >= min_total, "сумма {min_total}");
                    assert!(attempts >= 1);
                }
                other => panic!("сумма {min_total}: {other:?}"),
            }
        }
    }

    #[test]
    fn result_does_not_depend_on_thread_count() {
        let run = |threads| match search_with(100, threads, 2024, &Control::new()) {
            Search::Found { sheet, attempts } => (attempts, sheet.rolls().to_vec()),
            other => panic!("ожидался набор, получено {other:?}"),
        };
        let single = run(1);
        assert!(single.0 > BLOCK, "порог должен требовать нескольких блоков");
        assert_eq!(run(3), single);
        assert_eq!(run(8), single);
    }

    #[test]
    fn stopped_before_start() {
        let control = Control::new();
        control.stop();
        assert!(matches!(search_with(80, 4, 1, &control), Search::Stopped { attempts: 0 }));
    }

    #[test]
    fn stops_on_request_from_another_thread() {
        let control = Control::new();
        let result = thread::scope(|scope| {
            // 108 — шесть восемнадцатых, примерно раз на 10¹¹ наборов.
            let search = scope.spawn(|| search_with(MAX_TOTAL, 4, 3, &control));
            thread::sleep(Duration::from_millis(50));
            control.stop();
            search.join().unwrap()
        });
        match result {
            Search::Stopped { attempts } => assert!(attempts > 0),
            other => panic!("ожидалась остановка, получено {other:?}"),
        }
    }

    #[test]
    fn rejects_impossible_total() {
        assert!(matches!(
            search_with(MAX_TOTAL + 1, 4, 1, &Control::new()),
            Search::Stopped { attempts: 0 }
        ));
    }
}
