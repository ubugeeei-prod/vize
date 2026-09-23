//! TS-34 spec for `ProvideInject` and `RaceConditions`.
//!
//! The naive side is the tracker row in registration order. Production is
//! the fact table. A dropped, reordered, or rewritten row is a divergence.

use vize_carton::{CompactString, cstr};

use super::agreement::Agreement;
use crate::Croquis;
use crate::facts::{composable_calls, inject_entries, provide_entries, race_risks};

/// Compare one artifact's provide/inject rows with the tracker.
pub fn compare_provide(name: &str, croquis: &Croquis, agreement: &mut Agreement) {
    let provides = provide_entries(croquis);
    let injects = inject_entries(croquis);
    let composables = composable_calls(croquis);
    if provides.is_empty() && injects.is_empty() && composables.is_empty() {
        agreement.skip("no-provide-inject");
        return;
    }
    let tracker = &croquis.provide_inject;
    let bad = if provides.as_slice() != tracker.provides() {
        Some(cstr!("{name}: provides diverged"))
    } else if injects.as_slice() != tracker.injects() {
        Some(cstr!("{name}: injects diverged"))
    } else if composables.as_slice() != tracker.composables() {
        Some(cstr!("{name}: composables diverged"))
    } else {
        None
    };
    agreement.compare(provides.len() + injects.len() + composables.len(), bad);
}

/// Compare one artifact's race rows with the tracker.
pub fn compare_race(name: &str, croquis: &Croquis, agreement: &mut Agreement) {
    let risks = race_risks(croquis);
    if risks.is_empty() {
        agreement.skip("no-race");
        return;
    }
    let bad = (risks.as_slice() != croquis.race_conditions.risks())
        .then(|| cstr!("{name}: race risks diverged"));
    agreement.compare(risks.len(), bad);
}

/// The pairing key is the provide key text, not the local binding name.
#[must_use]
pub fn pairing_key(key: &crate::provide::ProvideKey) -> CompactString {
    match key {
        crate::provide::ProvideKey::String(text) | crate::provide::ProvideKey::Symbol(text) => {
            text.clone()
        }
    }
}
