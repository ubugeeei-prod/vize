use super::support::{
    LOCALES, NeverLookup, complete, error, expected, parity, registered, selected, span,
};
use vize_l0::{Allocator, cstr};
use vize_l1::SurfaceChild;
use vize_patina::native::{NativeSyntaxLint, NativeTextareaLintError};

fn refused(source: &str, depth: usize) {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let mut element = owner.children().next().unwrap().into_element().unwrap();
    for _ in 1..depth {
        element = element.children().next().unwrap().into_element().unwrap();
    }
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let child = element.children().next().unwrap();
    match child.surface() {
        SurfaceChild::Text(token) => assert_eq!(token.text, "{{x}}"),
        _ => panic!("expected the authentic selected literal child"),
    }
    let facts = header.child_facts(child).unwrap();
    assert!(core::ptr::eq(facts.header().original(), element.surface()));
    assert_eq!(facts.markers().unwrap().len(), 0);
    assert_eq!(facts.textarea_mustache().unwrap().len(), 0);
    assert_eq!(
        facts.no_textarea_mustache(&NeverLookup).err(),
        Some(NativeTextareaLintError::InheritedLiteralContext {
            span: span(source, "<textarea>")
        })
    );
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
}

#[test]
fn original_slot_pre_can_expose_a_registered_marker_that_selected_l1_freezes() {
    for (source, depth) in [
        (
            "<template><slot v-pre><textarea>{{x}}</textarea></slot></template>",
            2,
        ),
        (
            "<template><slot v-pre><div><textarea>{{x}}</textarea></div></slot></template>",
            3,
        ),
    ] {
        refused(source, depth);
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(source, locale)),
                expected(vec![error(locale, span(source, "{{x}}"))])
            );
        }
    }
}

#[test]
fn an_outer_pre_scope_can_keep_the_registered_slot_descendant_clean_but_still_refuses() {
    let source = "<template><div v-pre><slot><textarea>{{x}}</textarea></slot></div></template>";
    refused(source, 3);
    for locale in LOCALES {
        assert_eq!(complete(&registered(source, locale)), expected(vec![]));
    }
}

#[test]
fn own_exact_textarea_pre_proves_the_local_freeze_even_under_a_slot_pre() {
    for source in [
        "<template><slot v-pre><textarea v-pre>{{x}}</textarea></slot></template>",
        "<template><slot v-pre><div><textarea v-pre>{{x}}</textarea></div></slot></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(parity(source, locale), expected(vec![]));
        }
    }
}

#[test]
fn ordinary_slot_fallback_retains_its_original_direct_textarea_marker() {
    let source = "<template><slot><textarea>{{x}}</textarea></slot></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![error(locale, span(source, "{{x}}"))])
        );
    }
}

#[test]
fn other_admitted_directive_bindings_preserve_original_child_callbacks() {
    for head in [
        "v-html='x'",
        "v-text='x'",
        "v-if='x'",
        "v-for='x in xs'",
        "#default",
    ] {
        let source = cstr!("<template><textarea {head}>{{{{x}}}}</textarea></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![error(locale, span(&source, "{{x}}"))])
            );
        }
    }
}
