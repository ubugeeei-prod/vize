//! Bounded waits for the existing response-backed readiness requests.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
};
use vize_l0::{String, cstr};

const MAX_IN_FLIGHT: usize = 16;

pub(super) fn run<T: Sync>(
    items: &[T],
    request: impl Fn(&T) -> Result<(), String> + Sync,
) -> Result<(), String> {
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let first_error = Mutex::new(None);
    thread::scope(|scope| {
        let mut workers = Vec::with_capacity(items.len().min(MAX_IN_FLIGHT));
        for _ in 0..items.len().min(MAX_IN_FLIGHT) {
            let worker = thread::Builder::new()
                .name("vize-lsp-readiness".into())
                .spawn_scoped(scope, || {
                    while !failed.load(Ordering::Acquire) {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(index) else {
                            break;
                        };
                        if failed.load(Ordering::Acquire) {
                            break;
                        }
                        if let Err(error) = request(item) {
                            remember_error(&first_error, error);
                            failed.store(true, Ordering::Release);
                            break;
                        }
                    }
                });
            match worker {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    remember_error(
                        &first_error,
                        cstr!("Failed to start editor readiness worker: {error}"),
                    );
                    failed.store(true, Ordering::Release);
                    break;
                }
            }
        }
        // The owning bridge operation cannot return or retire its session
        // until every submitted request has completed or hit its own deadline.
        for worker in workers {
            if worker.join().is_err() {
                remember_error(&first_error, "Editor readiness worker panicked".into());
                failed.store(true, Ordering::Release);
            }
        }
    });
    let error = match first_error.into_inner() {
        Ok(error) => error,
        Err(poisoned) => poisoned.into_inner(),
    };
    match error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn remember_error(errors: &Mutex<Option<String>>, error: String) {
    let mut first = match errors.lock() {
        Ok(first) => first,
        Err(poisoned) => poisoned.into_inner(),
    };
    if first.is_none() {
        *first = Some(error);
    }
}

#[cfg(all(test, unix))]
mod tests;
