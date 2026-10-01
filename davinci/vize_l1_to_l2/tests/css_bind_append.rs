//! P2-10/P2-11 boundary: append only style facts the DOM skipper can see.

#![expect(
    clippy::expect_used,
    clippy::string_slice,
    reason = "tests assert by panicking"
)]

use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, SourceRoot};
use vize_l1::parse;
use vize_l1_to_l2::lower_source_block;
use vize_l2::dump::Page as L2Page;

fn block_between<'a>(source: &'a str, open: &str, close: &str) -> (&'a str, u32) {
    let tag = source.find(open).expect("opening tag");
    let inner = tag + open.len();
    let rel = source[inner..].find(close).expect("closing tag");
    (&source[inner..inner + rel], inner as u32)
}

#[test]
fn style_blocks_without_css_binds_do_not_append_a_carrier() {
    let source = "<template><p>hi</p></template><style>.foo{color:red}</style>";
    let allocator = Allocator::new();
    let root = SourceRoot::new(source).expect("source is small");
    let (template, template_start) = block_between(source, "<template>", "</template>");
    let (css, css_start) = block_between(source, "<style>", "</style>");
    let (tree, errors) = parse(&allocator, template);
    let mut lowered = lower_source_block(
        &allocator,
        &tree,
        &errors,
        root.block(template, template_start)
            .expect("template block is a source slice"),
    );
    lowered.push_style_block_in(
        &allocator,
        root.block(css, css_start)
            .expect("style block is a source slice"),
    );

    let folio = L2Page::of(&lowered.root.ops).expect("compatibility expression aliases");
    assert_eq!(lowered.op_count, 2);
    assert_eq!(
        folio.print_to_string(DumpMode::Full).as_str(),
        "\
[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.element p @10:19
  ui.text \"hi\" @13:15

"
    );
}
