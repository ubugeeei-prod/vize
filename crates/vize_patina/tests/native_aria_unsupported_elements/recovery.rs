use super::support::{
    LOCALES, NeverLookup, collect, complete, error, expected, parser, registered, selected, span,
};
use vize_l0::{Allocator, cstr};
use vize_patina::native::{
    NativeLintRefusal, NativeSyntaxLint, header_facts::NativeHeaderFactError,
};

const IGNORED: &str =
    "HTML tree construction ignored this start tag because an equivalent element is already open.";
const INVALID_END: &str = "Invalid end tag.";

fn refused(source: &str, path: &[usize]) {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let mut element = owner
        .children()
        .nth(path[0])
        .unwrap()
        .into_element()
        .unwrap();
    let mut first_context = None;
    for ordinal in &path[1..] {
        // Complete caller traversal reaches an earlier authored sibling before
        // the path's target, including the genuine self-closing nested a.
        for sibling in element.children().take(*ordinal + 1) {
            let sibling = sibling.into_element().unwrap();
            let receipt = sibling.lint_tag().unwrap();
            if receipt.in_recovery_context() {
                first_context.get_or_insert(receipt.span());
            }
        }
        element = element
            .children()
            .nth(*ordinal)
            .unwrap()
            .into_element()
            .unwrap();
        let receipt = element.lint_tag().unwrap();
        if receipt.in_recovery_context() {
            first_context.get_or_insert(receipt.span());
        }
    }
    let receipt = element.lint_tag().unwrap();
    assert!(receipt.header_is_literal());
    assert!(receipt.in_recovery_context());
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(
            NativeLintRefusal::RecoveryContext {
                span: receipt.span()
            }
        ))
    );
    let mut output = Vec::new();
    assert_eq!(
        collect(&lint, owner.children(), &NeverLookup, &mut output),
        Err(NativeHeaderFactError::Header(
            NativeLintRefusal::RecoveryContext {
                span: first_context.unwrap()
            }
        ))
    );
    assert!(output.is_empty());
    assert!(core::ptr::eq(lint.owner(), &owner));
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
}

#[test]
fn genuine_pre_recovery_refusal_retains_every_original_parser_and_aria_error() {
    for (source, path, ignored, invalid_end) in [
        (
            "<template><form><form v-pre><meta :role='x' /></form></form></template>",
            &[0, 0, 0][..],
            Some("<form v-pre>"),
            Some("</form></template>"),
        ),
        (
            "<template><p v-pre><div><meta :role='x' /></div></p></template>",
            &[0, 0, 0][..],
            None,
            Some("</p>"),
        ),
        (
            "<template><a v-pre><a /><meta :role='x' /></a></template>",
            &[0, 1][..],
            None,
            None,
        ),
        (
            "<template><select><option v-pre><option><meta :role='x' /></option></option></select></template>",
            &[0, 0, 0, 0][..],
            None,
            Some("</option></select>"),
        ),
        (
            "<template><select><optgroup v-pre><optgroup><meta :role='x' /></optgroup></optgroup></select></template>",
            &[0, 0, 0, 0][..],
            None,
            Some("</optgroup></select>"),
        ),
    ] {
        refused(source, path);
        for locale in LOCALES {
            let mut diagnostics = Vec::new();
            if let Some(head) = ignored {
                diagnostics.push(parser("error", IGNORED, span(source, head)));
            }
            diagnostics.push(error(locale, span(source, ":role='x'"), "meta", "role"));
            if let Some(tail) = invalid_end {
                let mut range = span(source, tail);
                range.end = range.start + u32::try_from(tail.find('>').unwrap() + 1).unwrap();
                diagnostics.push(parser("error", INVALID_END, range));
            }
            assert_eq!(
                complete(&registered(source, locale)),
                expected(diagnostics),
                "{source}: {locale:?}"
            );
        }
    }
}

#[test]
fn actual_depth_guard_refusal_preserves_complete_registered_depth_and_aria_errors() {
    let source = cstr!(
        "<template>{}<div v-pre><meta :role='x' /></div>{}</template>",
        "<div>".repeat(4096),
        "</div>".repeat(4096)
    );
    let arena = Allocator::default();
    let owner = selected(&arena, &source);
    let original_source = owner.component().carrier().tree.source;
    let mut original = owner.children().next().unwrap().into_element().unwrap();
    for _ in 0..4096 {
        original = original.children().next().unwrap().into_element().unwrap();
    }
    let meta = original.children().next().unwrap().into_element().unwrap();
    let receipt = meta.lint_tag().unwrap();
    assert!(receipt.header_is_literal());
    assert!(receipt.in_recovery_context());
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.header_facts(&meta).err(),
        Some(NativeHeaderFactError::Header(
            NativeLintRefusal::RecoveryContext {
                span: receipt.span()
            }
        ))
    );
    assert!(core::ptr::eq(
        original_source,
        owner.component().carrier().tree.source
    ));
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(&source, locale)),
            expected(vec![
                parser(
                    "error",
                    "Element nesting is too deep.",
                    span(&source, "<div v-pre>")
                ),
                error(locale, span(&source, ":role='x'"), "meta", "role"),
            ])
        );
    }
}
