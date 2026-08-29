//! Exact-pinned lowering for `v-cloak` as the P2-11 `vue.cloak` binding.

mod support;

use support::artifact;
#[test]
fn bare_v_cloak_lowers_to_the_cloak_dialect_op() {
    let art = artifact("<p v-cloak>x</p>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=3\n\
         \n\
         [disegno.ops]\n\
         ui.element p @0:16\n\
         \x20 vue.cloak @3:10\n\
         \x20 ui.text \"x\" @11:12\n\
         \n"
    );
    assert_eq!(art.diagnostics, vec![]);
}

#[test]
fn v_cloak_with_value_argument_or_modifier_lowers_to_the_same_marker() {
    for (src, attr_end, text_start) in [
        (r#"<p v-cloak="x">x</p>"#, 14, 15),
        (r#"<p v-cloak:foo>x</p>"#, 14, 15),
        (r#"<p v-cloak.mod>x</p>"#, 14, 15),
        (r#"<p v-cloak:[foo]="x">x</p>"#, 20, 21),
    ] {
        let art = artifact(src);
        assert_eq!(
            art.folio,
            format!(
                "[disegno]\n\
                 ops=3\n\
                 \n\
                 [disegno.ops]\n\
                 ui.element p @0:{}\n\
                 \x20 vue.cloak @3:{attr_end}\n\
                 \x20 ui.text \"x\" @{text_start}:{}\n\
                 \n",
                text_start + 5,
                text_start + 1
            )
        );
        assert_eq!(art.diagnostics, vec![], "{src}");
    }
}

#[test]
fn v_cloak_on_a_slot_outlet_lowers_as_an_inert_slot_binding() {
    let art = artifact("<slot v-cloak></slot>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=2\n\
         \n\
         [disegno.ops]\n\
         ui.slot name=\"default\" @0:21\n\
         \x20 vue.cloak @6:13\n\
         \n"
    );
    assert_eq!(art.diagnostics, vec![]);
}
