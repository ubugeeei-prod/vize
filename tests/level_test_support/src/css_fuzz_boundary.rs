//! Exercise recovery inside the public API without libFuzzer aborting first.

use std::{
    cell::Cell,
    panic::{UnwindSafe, catch_unwind},
};

thread_local! {
    static IN_PUBLIC_CALL: Cell<bool> = const { Cell::new(false) };
}

pub fn install_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if !IN_PUBLIC_CALL.get() {
            previous(info);
        }
    }));
}

pub fn call<T>(body: impl FnOnce() -> T + UnwindSafe) -> std::thread::Result<T> {
    let previous = IN_PUBLIC_CALL.replace(true);
    let result = catch_unwind(body);
    IN_PUBLIC_CALL.set(previous);
    result
}
