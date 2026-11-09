//! Overlapping each program's native check with the next program's preparation.

use std::path::PathBuf;
use std::thread::ScopedJoinHandle;

/// Prepare `items` in order and run each prepared check on its own thread while
/// the next item prepares. `program_concurrency` is the upper bound on checks
/// this run may overlap, so every program shares one machine budget.
/// Checks with the same key take turns, in order. Results keep item order; a
/// preparation error stops preparing.
pub(super) fn prepare_and_check_overlapped<T, P, R, E>(
    items: Vec<T>,
    program_concurrency: usize,
    mut prepare: impl FnMut(T, usize) -> Result<Option<P>, E>,
    key: impl Fn(&P) -> PathBuf,
    check: impl Fn(P) -> R + Sync,
) -> (Vec<R>, Option<E>)
where
    P: Send,
    R: Send,
{
    let check = &check;
    std::thread::scope(|scope| {
        let mut results = Vec::new();
        let mut running: Vec<(usize, PathBuf, ScopedJoinHandle<'_, R>)> = Vec::new();
        let mut error = None;
        let program_concurrency = program_concurrency.max(1);
        for item in items {
            let prepared = match prepare(item, program_concurrency) {
                Ok(Some(prepared)) => prepared,
                Ok(None) => continue,
                Err(failure) => {
                    error = Some(failure);
                    break;
                }
            };
            let root = key(&prepared);
            let (shared, other): (Vec<_>, Vec<_>) = std::mem::take(&mut running)
                .into_iter()
                .partition(|(_, running_root, _)| *running_root == root);
            running = other;
            for (index, _, handle) in shared {
                finish(&mut results, index, handle);
            }
            while running.len() >= program_concurrency {
                let finished = running
                    .iter()
                    .position(|(_, _, handle)| handle.is_finished())
                    .unwrap_or(0);
                let (index, _, handle) = running.remove(finished);
                finish(&mut results, index, handle);
            }
            running.push((results.len(), root, scope.spawn(move || check(prepared))));
            results.push(None);
        }
        for (index, _, handle) in running {
            finish(&mut results, index, handle);
        }
        (results.into_iter().flatten().collect(), error)
    })
}

fn finish<R>(results: &mut [Option<R>], index: usize, handle: ScopedJoinHandle<'_, R>) {
    let result = handle
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    if let Some(slot) = results.get_mut(index) {
        *slot = Some(result);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    use super::prepare_and_check_overlapped;

    #[test]
    fn a_check_runs_while_the_next_program_prepares() {
        let (prepared_second, second_prepared) = mpsc::channel();
        let second_prepared = Mutex::new(second_prepared);
        let (results, error) = prepare_and_check_overlapped(
            vec![0, 1],
            2,
            |item: usize, concurrency| {
                assert_eq!(concurrency, 2);
                if item == 1 {
                    prepared_second.send(()).unwrap();
                }
                Ok::<_, ()>(Some(item))
            },
            |item| PathBuf::from(if *item == 0 { "first" } else { "second" }),
            |item| {
                item != 0
                    || second_prepared
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(10))
                        .is_ok()
            },
        );
        assert_eq!((results, error), (vec![true, true], None));
    }

    #[test]
    fn checks_sharing_a_root_take_turns_and_results_keep_order() {
        let first_done = AtomicBool::new(false);
        let prepared = Mutex::new(Vec::new());
        let (results, error) = prepare_and_check_overlapped(
            vec![0, 1, 2, 3],
            4,
            |item: usize, concurrency| {
                assert_eq!(concurrency, 4);
                prepared.lock().unwrap().push(item);
                match item {
                    2 => Ok(None),
                    3 => Err("failed"),
                    _ => Ok(Some(item)),
                }
            },
            |_| PathBuf::from("shared"),
            |item| {
                if item == 0 {
                    std::thread::sleep(Duration::from_millis(50));
                    first_done.store(true, Ordering::SeqCst);
                }
                (item, first_done.load(Ordering::SeqCst))
            },
        );
        assert_eq!(results, vec![(0, true), (1, true)]);
        assert_eq!(error, Some("failed"));
        assert_eq!(*prepared.lock().unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn checks_stay_within_the_program_limit() {
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let (results, error) = prepare_and_check_overlapped(
            vec![0, 1, 2, 3],
            2,
            |item, concurrency| {
                assert_eq!(concurrency, 2);
                Ok::<_, ()>(Some(item))
            },
            |item| PathBuf::from(item.to_string()),
            |item| {
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(current, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(20));
                active.fetch_sub(1, Ordering::SeqCst);
                item
            },
        );
        assert_eq!(results, [0, 1, 2, 3]);
        assert_eq!(error, None);
        assert_eq!(peak.load(Ordering::SeqCst), 2);
    }
}
