#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::*;

#[test]
fn multiline_own_node_errors_do_not_consume_child_or_sibling_expectations() {
    let source = "<template><!-- @vue-expect-error -->\n<div\n :x=\"outer\">{{ child }}<!-- @vue-expect-error -->{{ inner }}</div>{{ sibling }}</template>";
    let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
    assert_eq!(plan.directives().len(), 2);
    assert!(!plan.suppresses(source.find("child").unwrap()));
    assert_eq!(plan.unused_expectations().count(), 2);
    assert!(plan.suppresses(source.find("outer").unwrap()));
    assert!(plan.suppresses(source.find("inner").unwrap()));
    assert!(!plan.suppresses(source.find("sibling").unwrap()));
    assert_eq!(plan.unused_expectations().count(), 0);
}

#[test]
fn skip_owns_whole_subtree_but_ignore_does_not() {
    for (directive, children_suppressed) in [("skip", true), ("ignore", false)] {
        let source = vize_carton::cstr!(
            "<template><!-- @vue-{directive} --><div :x=\"outer\">{{ child }}</div>{{ sibling }}</template>"
        );
        let mut plan = TemplateDiagnosticDirectives::for_sfc(&source);
        assert!(plan.suppresses(source.find("outer").unwrap()));
        assert_eq!(
            plan.suppresses(source.find("child").unwrap()),
            children_suppressed,
            "{source}: {plan:?}"
        );
        assert!(!plan.suppresses(source.find("sibling").unwrap()));
        assert_eq!(plan.unused_expectations().count(), 0);
    }
}

#[test]
fn comments_inside_a_skipped_subtree_do_not_report_unused_errors() {
    let source =
        "<template><!-- @vue-skip --><div><!-- @vue-expect-error -->{{ valid }}</div></template>";
    let plan = TemplateDiagnosticDirectives::for_sfc(source);
    assert_eq!(plan.directives().len(), 1);
    assert_eq!(plan.unused_expectations().count(), 0);
}

#[test]
fn an_outer_expectation_never_claims_nested_descendants_between_siblings() {
    let source = "<template><!-- @vue-expect-error --><div :x=\"outer\"><span>{{ nested }}</span><p>{{ other }}</p></div></template>";
    let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
    assert!(!plan.suppresses(source.find("nested").unwrap()));
    assert!(!plan.suppresses(source.find("other").unwrap()));
    assert_eq!(plan.unused_expectations().count(), 1);
    assert!(plan.suppresses(source.find("outer").unwrap()));
}

#[test]
fn token_ownership_excludes_script_strings_attributes_and_comment_prose() {
    for source in [
        "<script setup>const s = '<!-- @vue-ignore -->';</script><template>{{ missing }}</template>",
        "<template><div title=\"<!-- @vue-ignore -->\">{{ missing }}</div></template>",
        "<template><!-- prose @vue-ignore -->{{ missing }}</template>",
        "<template><!-- @vue-ignore_more -->{{ missing }}</template>",
        "<template><!-- @vue-ignore-more -->{{ missing }}</template>",
        "<template>{{ '<!-- @vue-ignore -->' }}{{ missing }}</template>",
    ] {
        let plan = TemplateDiagnosticDirectives::for_sfc(source);
        assert!(plan.directives().is_empty(), "{source}");
    }
}

#[test]
fn empty_expectations_retain_the_authored_comment_range() {
    let source = "<template>😀\r\n<!--  @vue-expect-error reason -->\r\n</template>";
    let plan = TemplateDiagnosticDirectives::for_sfc(source);
    let directive = &plan.directives()[0];
    assert_eq!(&source[directive.token.clone()], "@vue-expect-error");
    assert_eq!(
        &source[directive.comment.clone()],
        "<!--  @vue-expect-error reason -->"
    );
    assert_eq!(plan.unused_expectations().count(), 1);
}

#[test]
fn a_node_directive_owns_its_generic_argument_comment_in_either_order() {
    for source in [
        "<template><!-- @vue-expect-error --><!-- @vue-generic {boolean} --><Comp />{{ sibling }}</template>",
        "<template><!-- @vue-generic {boolean} --><!-- @vue-expect-error --><Comp />{{ sibling }}</template>",
    ] {
        let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
        assert_eq!(plan.directives().len(), 1, "{source}");
        assert!(plan.suppresses(source.find("boolean").unwrap()), "{source}");
        assert!(
            !plan.suppresses(source.find("sibling").unwrap()),
            "{source}"
        );
        assert_eq!(plan.unused_expectations().count(), 0, "{source}");
    }
    // Without a directive the generic comment's diagnostics stay reportable.
    let source = "<template><!-- @vue-generic {boolean} --><Comp /></template>";
    let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
    assert_eq!(plan.directives().len(), 0);
    assert!(!plan.suppresses(source.find("boolean").unwrap()));
}
