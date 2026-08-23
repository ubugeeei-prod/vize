//! Regression coverage for slot-owned `ui.model` folio bindings.
//!
//! The folio parser accepts the same attached-binding group for elements,
//! components, and slot outlets. `ui.model` used the same admission path as
//! leaf bindings but did not close back into `FolioSlot::bindings`, so malformed
//! fuzz input could panic when indentation closed a slot-owned model frame.

use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::folio::DisegnoFolio;

const SLOT_MODEL: &str = "\
[disegno]
ops=2

[disegno.ops]
ui.slot name=\"head\" @21:40
  ui.model read=js(\"value\" @22:27) write=js(\"value = $event\" @30:44) @21:40
    attr element-kind=\"slot\" @22:33

";

#[test]
fn slot_owned_model_bindings_round_trip_without_panicking() {
    let parsed = DisegnoFolio::parse(SLOT_MODEL).expect("slot-owned model binding parses");
    assert_eq!(parsed.print_to_string(FolioMode::Full), SLOT_MODEL);
}
