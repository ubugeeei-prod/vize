use vize_davinci::folio::{Folio, FolioMode};
use vize_l2::folio::{DisegnoFolio, L2Folio};

const EMPTY: &str = "[disegno]\nops=0\n\n";

#[test]
fn l2_folio_is_the_physical_stage_name() {
    let folio = L2Folio::parse(EMPTY).expect("empty L2 folio parses");

    assert_eq!(folio, L2Folio::default());
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), EMPTY);
}

#[test]
fn disegno_folio_remains_a_compatibility_alias() {
    let folio: DisegnoFolio = L2Folio::default();
    let stage_named: L2Folio = DisegnoFolio::parse(EMPTY).expect("compat alias parses");

    assert_eq!(folio, stage_named);
}
