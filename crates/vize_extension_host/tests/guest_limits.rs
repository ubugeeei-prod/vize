//! P6-3: per-guest fuel and memory limits, enforced identically in both
//! hosting modes over the same guest binary.
//!
//! The echo guest has two probes: `<!-- spin -->` loops forever and
//! `<!-- hoard -->` allocates until memory runs out. Under tight limits each
//! is stopped with its exact error, and a real golden block still passes.

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use vize_extension_host::outproc::{OutOfProcessGuest, serve_command_with};
use vize_extension_host::wasm::WasmGuest;
use vize_extension_host::{
    ContractError, GuestError, GuestLimits, InputDialectGuest, Session, SourceBlock,
};
use vize_s0::{String, cstr};

use support::{CASES, build_echo_guest, golden_texts};

const TIGHT: GuestLimits = GuestLimits {
    fuel_per_call: 5_000_000,
    max_memory_bytes: 16 * 1024 * 1024,
};

fn echo_guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| build_echo_guest(&[]))
}

fn sessions() -> [(&'static str, Session<Box<dyn InputDialectGuest>>); 2] {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let out: Box<dyn InputDialectGuest> = Box::new(
        OutOfProcessGuest::spawn(serve_command_with(runner, echo_guest(), TIGHT)).expect("loads"),
    );
    let inside: Box<dyn InputDialectGuest> =
        Box::new(WasmGuest::load_with(echo_guest(), TIGHT).expect("loads"));
    [
        ("out-of-process", Session::open(out).expect("negotiates")),
        ("in-process", Session::open(inside).expect("negotiates")),
    ]
}

fn probe(source: &str) -> SourceBlock {
    SourceBlock {
        source: String::from(source),
        base: 0,
        lang: None,
    }
}

#[test]
fn the_default_limits_are_pinned() {
    assert_eq!(
        GuestLimits::default(),
        GuestLimits {
            fuel_per_call: 1_000_000_000,
            max_memory_bytes: 134_217_728,
        }
    );
}

#[test]
fn a_runaway_guest_is_stopped_at_its_fuel_budget_in_both_modes() {
    for (mode, mut session) in sessions() {
        let error = session
            .lower_block(&probe("<!-- spin -->"))
            .expect_err("the loop never ends on its own");
        assert_eq!(
            error,
            ContractError::Guest(GuestError::OutOfFuel { budget: 5_000_000 }),
            "{mode}"
        );
        assert_eq!(
            cstr!("{error}"),
            "guest stopped: it used up its fuel budget of 5000000 per call",
            "{mode}"
        );
    }
}

#[test]
fn a_hoarding_guest_is_stopped_at_its_memory_limit_in_both_modes() {
    for (mode, mut session) in sessions() {
        let error = session
            .lower_block(&probe("<!-- hoard -->"))
            .expect_err("the hoard outgrows 16 MiB");
        assert_eq!(
            error,
            ContractError::Guest(GuestError::MemoryLimit { limit: 16_777_216 }),
            "{mode}"
        );
        assert_eq!(
            cstr!("{error}"),
            "guest stopped: it tried to grow its memory past 16777216 bytes",
            "{mode}"
        );
    }
}

#[test]
fn honest_work_fits_the_tight_limits_in_both_modes() {
    for (mode, mut session) in sessions() {
        for case in CASES {
            let accepted = session
                .lower_block(&case.block())
                .unwrap_or_else(|error| panic!("{mode} {}: {error}", case.name));
            assert_eq!(golden_texts(&accepted.lowered), case.committed(), "{mode}");
        }
    }
}
