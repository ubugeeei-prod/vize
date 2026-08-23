//! P2-10 acceptance: a committed `v-bind()`-bearing SFC whose S2 folio
//! pins the style ops beside the template tree.

use vize_carton::Allocator;
use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::folio::DisegnoFolio;
use vize_disegno::verify::{Rigor, Violation, verify};
use vize_ricalco::lower;
use vize_sinopia::parse;

const SOURCE: &str = include_str!("fixtures/css_bind.vue");

const CANONICAL: &str = "\
[disegno]
ops=4

[disegno.ops]
ui.element p @1:31
  attr class=\"foo\" @4:15
  ui.interpolation js(\"color\" @19:24) @16:27
ui.element style @0:32
  vue.css-bind value=js(\"color\" @22:27) @15:28

";

fn block_between<'a>(source: &'a str, open: &str, close: &str) -> (&'a str, u32) {
    let tag = source.find(open).expect("opening tag");
    let inner = tag + open.len();
    let rel = source[inner..].find(close).expect("closing tag");
    (&source[inner..inner + rel], inner as u32)
}

#[test]
fn the_sfc_fixture_folio_pins_template_and_css_bind() {
    let allocator = Allocator::default();
    let (template, _) = block_between(SOURCE, "<template>", "</template>");
    let (css, css_start) = block_between(SOURCE, "<style>", "</style>");
    let (tree, errors) = parse(&allocator, template);
    let mut lowered = lower(&allocator, &tree, &errors);
    lowered.push_style_block(&allocator, css, css_start);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    assert_eq!(u64::from(lowered.op_count), folio.op_count());
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(verify(&folio, Rigor::Raw), Vec::<Violation>::new());
}
