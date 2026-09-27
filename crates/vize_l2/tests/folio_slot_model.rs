//! Regression coverage for slot-owned `ui.model` folio bindings.
//!
//! The folio parser accepts the same attached-binding group for elements,
//! components, and slot outlets. `ui.model` used the same admission path as
//! leaf bindings but did not close back into `DumpSlot::bindings`, so malformed
//! fuzz input could panic when indentation closed a slot-owned model frame.

use vize_davinci::dump::{Dump, Mode as DumpMode};
use vize_l2::dump::Page as L2Page;

const SLOT_MODEL: &str = "\
[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.slot name=\"head\" @21:40
  ui.model read=js(\"value\" @22:27) write=js(\"value = $event\" @30:44) @21:40
    attr element-kind=\"slot\" @22:33

";

#[test]
fn slot_owned_model_bindings_round_trip_without_panicking() {
    let parsed = L2Page::parse(SLOT_MODEL).expect("slot-owned model binding parses");
    assert_eq!(parsed.print_to_string(DumpMode::Full), SLOT_MODEL);
}
