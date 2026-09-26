use vize_davinci::dump::{Dump, Mode as DumpMode};
use vize_l2::dump::Page as L2Page;

const EMPTY: &str = "[disegno]\nops=0\n\n";

#[test]
fn l2_folio_is_the_physical_stage_name() {
    let folio = L2Page::parse(EMPTY).expect("empty L2 folio parses");

    assert_eq!(folio, L2Page::default());
    assert_eq!(folio.print_to_string(DumpMode::Full).as_str(), EMPTY);
}

#[test]
fn canonical_dump_namespace_keeps_the_full_wire_contract() {
    let page: vize_l2::dump::Page =
        vize_l2::dump::Page::parse(EMPTY).expect("canonical namespace parses");
    let full = page.print_to_string(DumpMode::Full);
    let replay = vize_l2::dump::Page::parse(&full).expect("full dump parses");

    assert_eq!(full.as_str(), EMPTY);
    assert_eq!(replay, page);
}
