//! Exact-pinned structural shapes: `v-if` chains as region-owning
//! `ui.if`, `v-for` as `ui.for` under the P2-5b split decision, template
//! unwrapping, and the Vue 3 `v-if`-outside-`v-for` precedence. Every
//! oracle is whole-artifact equality on the canonical `Full`-mode folio
//! (assurance §4: no partial matching).

mod support;

use support::artifact;
use vize_davinci::diagnostic::{Diagnostic, Severity, Stage};
use vize_s0::Span;

#[test]
fn a_v_if_chain_groups_into_one_region_owning_if() {
    let art = artifact("<div v-if=\"a\">x</div><span v-else-if=\"b\">y</span><p v-else>z</p>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=7\n\
         \n\
         [disegno.ops]\n\
         ui.if @0:64\n\
         \x20 branch js(\"a\" @11:12) @0:21\n\
         \x20   ui.element div @0:21\n\
         \x20     ui.text \"x\" @14:15\n\
         \x20 branch js(\"b\" @38:39) @21:49\n\
         \x20   ui.element span @21:49\n\
         \x20     ui.text \"y\" @41:42\n\
         \x20 branch @49:64\n\
         \x20   ui.element p @49:64\n\
         \x20     ui.text \"z\" @59:60\n\
         \n"
    );
    assert_eq!(art.diagnostics, Vec::new());
    assert_eq!(art.op_count, 7);
}

#[test]
fn the_for_value_splits_at_the_first_viable_keyword_never_as_js() {
    // The recorded `a in b in c` disagreement: Vue's grammar reads alias
    // `a`, source `b in c`; a retained AST of the whole value would read
    // `(a in b) in c`. The sub-slices are admitted individually.
    let art = artifact("<i v-for=\"a in b in c\">t</i>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=3\n\
         \n\
         [disegno.ops]\n\
         ui.for source=js(\"b in c\" @15:21) value=js(\"a\" @10:11) @0:28\n\
         \x20 ui.element i @0:28\n\
         \x20   ui.text \"t\" @23:24\n\
         \n"
    );
    assert_eq!(art.diagnostics, Vec::new());
}

#[test]
fn an_unsplittable_for_value_rides_whole_as_the_classified_escape() {
    // No viable ` in `/` of `: the whole value is `opaque(for-value)`
    // with pessimal semantics, the op and its region are kept, and the
    // error names Vue's grammar violation.
    let art = artifact("<i v-for=\"items\">t</i>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=3\n\
         \n\
         [disegno.ops]\n\
         ui.for source=opaque(for-value \"items\" @10:15) value=opaque(for-value \"\" @10:10) @0:22\n\
         \x20 ui.element i @0:22\n\
         \x20   ui.text \"t\" @17:18\n\
         \n"
    );
    assert_eq!(
        art.diagnostics,
        vec![Diagnostic::new(
            Severity::Error,
            Stage::Semantic,
            Span::new(10, 15),
            "v-for has invalid expression.",
        )]
    );
}

#[test]
fn an_absent_alias_is_a_zero_width_escape_position() {
    // `v-for=" in xs"` is valid Vue (the separator is viable because the
    // split runs over the untrimmed value); the alias position exists
    // and holds the zero-width escape.
    let art = artifact("<a v-for=\" in xs\">y</a>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=3\n\
         \n\
         [disegno.ops]\n\
         ui.for source=js(\"xs\" @14:16) value=opaque(for-value \"\" @11:11) @0:23\n\
         \x20 ui.element a @0:23\n\
         \x20   ui.text \"y\" @18:19\n\
         \n"
    );
    assert_eq!(art.diagnostics, Vec::new());
}

#[test]
fn a_template_wrapper_unwraps_into_its_branch_region() {
    let art = artifact("<template v-if=\"x\"><a>1</a><b>2</b></template>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=5\n\
         \n\
         [disegno.ops]\n\
         ui.if @0:46\n\
         \x20 branch js(\"x\" @16:17) @0:46\n\
         \x20   ui.element a @19:27\n\
         \x20     ui.text \"1\" @22:23\n\
         \x20   ui.element b @27:35\n\
         \x20     ui.text \"2\" @30:31\n\
         \n"
    );
}

#[test]
fn a_slot_template_v_if_keeps_the_template_carrier() {
    // `#header` has no home if the wrapper unwraps; createSlots reads
    // the kept `ui.element template` + `ui.slot-content`.
    let art = artifact(r#"<Foo><template #header v-if="ok">x</template></Foo>"#);
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=5\n\
         \n\
         [disegno.ops]\n\
         ui.component Foo @0:51\n\
         \x20 ui.if @5:45\n\
         \x20   branch js(\"ok\" @29:31) @5:45\n\
         \x20     ui.element template @5:45\n\
         \x20       ui.slot-content name=\"header\" @15:22\n\
         \x20       ui.text \"x\" @33:34\n\
         \n"
    );
    assert_eq!(art.diagnostics, Vec::new());
}

#[test]
fn v_if_evaluates_outside_v_for_on_one_element() {
    // Vue 3 precedence: the condition cannot see the iteration scope, so
    // the branch region owns the `ui.for`, which owns the element.
    let art = artifact("<p v-if=\"ok\" v-for=\"i in is\">t</p>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=4\n\
         \n\
         [disegno.ops]\n\
         ui.if @0:34\n\
         \x20 branch js(\"ok\" @9:11) @0:34\n\
         \x20   ui.for source=js(\"is\" @25:27) value=js(\"i\" @20:21) @0:34\n\
         \x20     ui.element p @0:34\n\
         \x20       ui.text \"t\" @29:30\n\
         \n"
    );
}

#[test]
fn an_orphan_else_keeps_its_fragment_under_the_exact_error() {
    let art = artifact("<div v-else>x</div>");
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=2\n\
         \n\
         [disegno.ops]\n\
         ui.element div @0:19\n\
         \x20 ui.text \"x\" @12:13\n\
         \n"
    );
    assert_eq!(
        art.diagnostics,
        vec![Diagnostic::new(
            Severity::Error,
            Stage::Semantic,
            Span::new(5, 11),
            "v-else/v-else-if has no adjacent v-if.",
        )]
    );
}
