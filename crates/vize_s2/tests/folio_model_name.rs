//! `ui.model name=` folio grammar pins.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s2::folio::DisegnoFolio;

#[test]
fn model_names_round_trip_on_the_model_line() {
    let canonical = "\
[disegno]
ops=2

[disegno.ops]
ui.component Field @0:32
  ui.model name=js(\"field\" @16:21) read=js(\"msg\" @24:27) write=js(\"msg\" @24:27) @5:28

";
    let value = DisegnoFolio::parse(canonical).expect("named model parses");
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), canonical);
}
