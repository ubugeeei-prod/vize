use vize_davinci::folio::{Folio, FolioMode};
use vize_s2::folio::{DisegnoFolio, S2Folio};

const EMPTY: &str = "[disegno]\nops=0\n\n";

#[test]
fn s2_folio_is_the_physical_stage_name() {
    let folio = S2Folio::parse(EMPTY).expect("empty S2 folio parses");

    assert_eq!(folio, S2Folio::default());
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), EMPTY);
}

#[test]
fn disegno_folio_remains_a_compatibility_alias() {
    let folio: DisegnoFolio = S2Folio::default();
    let stage_named: S2Folio = DisegnoFolio::parse(EMPTY).expect("compat alias parses");

    assert_eq!(folio, stage_named);
}
