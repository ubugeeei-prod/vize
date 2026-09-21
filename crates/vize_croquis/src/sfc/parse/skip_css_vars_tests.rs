//! `parse_sfc_without_css_vars` leaves `css_vars` empty and changes nothing
//! else in the descriptor.

use super::{parse_sfc, parse_sfc_without_css_vars};
use crate::sfc::SfcParseOptions;

const SOURCE: &str = "<template><p>{{ a }}</p></template>\n\
<style scoped>\n.a { color: v-bind(color); }\n</style>\n\
<style>\n.b { width: v-bind('size.w'); }\n</style>\n\
<art title=\"A\"></art>\n";

#[test]
fn skipping_css_vars_changes_only_css_vars() {
    let full = parse_sfc(SOURCE, SfcParseOptions::default()).expect("parses");
    let skipped = parse_sfc_without_css_vars(SOURCE, SfcParseOptions::default()).expect("parses");
    let vars: Vec<&str> = full.css_vars.iter().map(|var| var.as_ref()).collect();
    assert_eq!(vars, ["color", "size.w"]);
    assert_eq!(skipped.css_vars.len(), 0);

    let mut expected = full;
    expected.css_vars.clear();
    assert_eq!(
        vize_carton::cstr!("{skipped:?}"),
        vize_carton::cstr!("{expected:?}")
    );
}
