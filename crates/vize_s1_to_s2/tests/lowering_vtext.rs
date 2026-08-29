//! Exact-pinned lowering for `v-text` as the P2-11 `vue.text` binding.

mod support;

use support::artifact;
use vize_davinci::diagnostic::{Diagnostic, Severity, Stage};
use vize_s0::{Span, cstr};

#[test]
fn v_text_lowers_to_the_text_content_dialect_op() {
    let art = artifact(r#"<p v-text="raw">x</p>"#);
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=3\n\
         \n\
         [disegno.ops]\n\
         ui.element p @0:21\n\
         \x20 vue.text value=js(\"raw\" @11:14) @3:15\n\
         \x20 ui.text \"x\" @16:17\n\
         \n"
    );
    assert_eq!(art.diagnostics, vec![]);
}

#[test]
fn value_less_v_text_still_lowers_so_dom_emit_can_match_undefined() {
    for (src, element_end, attr_end, text_start) in [
        (r#"<p v-text>x</p>"#, 15, 9, 10),
        (r#"<p v-text="">x</p>"#, 18, 12, 13),
    ] {
        let art = artifact(src);
        assert_eq!(
            art.folio,
            cstr!(
                "[disegno]\n\
                 ops=3\n\
                 \n\
                 [disegno.ops]\n\
                 ui.element p @0:{element_end}\n\
                 \x20 vue.text @3:{attr_end}\n\
                 \x20 ui.text \"x\" @{text_start}:{}\n\
                 \n",
                text_start + 1
            )
        );
        assert_eq!(art.diagnostics, vec![]);
    }
}

#[test]
fn v_text_with_argument_or_modifier_still_defers() {
    for src in [
        r#"<p v-text:foo="raw">x</p>"#,
        r#"<p v-text.mod="raw">x</p>"#,
    ] {
        let art = artifact(src);
        assert_eq!(
            art.folio,
            "[disegno]\n\
             ops=2\n\
             \n\
             [disegno.ops]\n\
             ui.element p @0:25\n\
             \x20 ui.text \"x\" @20:21\n\
             \n"
        );
        assert_eq!(
            art.diagnostics,
            vec![Diagnostic::new(
                Severity::Info,
                Stage::Semantic,
                Span::new(3, 19),
                "`v-text` is representable as `vue.text` only with no argument or modifier",
            )]
        );
    }
}

#[test]
fn v_text_on_a_slot_outlet_lowers_as_a_slot_prop_binding() {
    let art = artifact(r#"<slot v-text="raw"></slot>"#);
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=2\n\
         \n\
         [disegno.ops]\n\
         ui.slot name=\"default\" @0:26\n\
         \x20 vue.text value=js(\"raw\" @14:17) @6:18\n\
         \n"
    );
    assert_eq!(art.diagnostics, vec![]);
}
