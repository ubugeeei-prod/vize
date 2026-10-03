use super::*;

fn contexts<'a>(owner: &NativeTemplateComponent<'a>) -> Vec<(&'a str, bool)> {
    fn collect<'a>(children: NativeChildren<'_, 'a>, output: &mut Vec<(&'a str, bool)>) {
        for child in children {
            if let Some(element) = child.into_element() {
                let receipt = element.lint_tag().unwrap();
                assert!(core::ptr::eq(receipt.component(), element.component()));
                assert!(core::ptr::eq(receipt.element(), element.surface()));
                output.push((element.surface().tag(), receipt.in_table_context()));
                collect(element.children(), output);
            }
        }
    }
    let mut output = Vec::new();
    collect(owner.children(), &mut output);
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    assert_eq!(
        rendered(&owner.component().carrier().tree).as_str(),
        owner.component().block().source()
    );
    output
}

#[test]
fn original_live_table_context_is_retained_without_moving_authored_elements() {
    let source = "<template><table><html role></html><tr><td><input autofocus /></td></tr></table><html role></html></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert_eq!(
        contexts(&owner),
        [
            ("table", false),
            ("html", true),
            ("tr", true),
            ("td", true),
            ("input", true),
            ("html", false)
        ]
    );
    let original = NativeComponent::parse_in(&arena, owner.component().block()).unwrap();
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().tree.children),
        cstr!("{:?}", original.carrier().tree.children)
    );
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().errors),
        cstr!("{:?}", original.carrier().errors)
    );
    let table = owner.children().next().unwrap().into_element().unwrap();
    let html = table.children().next().unwrap().into_element().unwrap();
    assert!(core::ptr::eq(
        html.parent_element().unwrap().surface(),
        table.surface()
    ));
    assert_eq!(html.lint_tag().unwrap().kind(), Kind::Element);
}

#[test]
fn self_closing_and_case_insensitive_table_frames_follow_original_scope() {
    let arena = Allocator::default();
    let owner = selected(
        &arena,
        "<template><table/><input/><TABLE><input/></table><input/></template>",
    );
    assert_eq!(
        contexts(&owner),
        [
            ("table", false),
            ("input", false),
            ("TABLE", false),
            ("input", true),
            ("input", false)
        ]
    );
}

#[test]
fn table_context_is_conservative_across_original_foreign_namespace_frames() {
    let arena = Allocator::default();
    let owner = selected(
        &arena,
        "<template><svg><table><html role/></table><html role/></svg><math><table><input/></table></math><input/></template>",
    );
    assert_eq!(
        contexts(&owner),
        [
            ("svg", false),
            ("table", false),
            ("html", true),
            ("html", false),
            ("math", false),
            ("table", false),
            ("input", true),
            ("input", false)
        ]
    );
}

#[test]
fn actual_interactive_recovery_ends_table_context_before_the_next_header() {
    let arena = Allocator::default();
    let owner = selected(
        &arena,
        "<template><a><table><tr><td><a></a></td></tr></table></a><input/></template>",
    );
    assert_eq!(
        contexts(&owner),
        [
            ("a", false),
            ("table", false),
            ("tr", true),
            ("td", true),
            ("a", false),
            ("input", false)
        ]
    );
    let first = owner.children().next().unwrap().into_element().unwrap();
    assert!(matches!(first.surface().close, ElementClose::Implicit));
}
