use super::support::{selected, span};
use vize_l0::{Allocator, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::native::{
    NativeLintRefusal, NativeSyntaxLint, header_facts::NativeHeaderFactError,
};

#[test]
fn unsupported_later_headers_refuse_the_whole_fact_artifact() {
    for (head, unresolved) in [
        ("v-bind", true),
        (":[key]", true),
        ("v-bind:[key]", true),
        ("@[key]", true),
        ("v-unknown", false),
        (":title.", false),
        ("@click..stop", false),
    ] {
        let source = cstr!("<template><meta aria-hidden role {head}='x' /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let range = span(&source, head);
        let refusal = if unresolved {
            NativeLintRefusal::UnresolvedBinding { span: range }
        } else {
            NativeLintRefusal::UnsupportedDirective { span: range }
        };
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(refusal))
        );
    }
}

#[test]
fn duplicate_static_attributes_refuse_even_for_exempt_components() {
    for tag in ["meta", "Meta"] {
        let source = cstr!("<template><{tag} role='x' title='one' TITLE='two' /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(
                NativeLintRefusal::DuplicateAttribute {
                    span: span(&source, "TITLE")
                }
            ))
        );
    }
}

#[test]
fn ambiguous_modified_pre_and_inherited_template_cannot_mint_counterexamples() {
    for head in ["v-pre.foo", "v-pre:argument", "v-pre:[argument]"] {
        let source = cstr!("<template><meta {head} role /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(NativeLintRefusal::LintTag {
                reason: NativeLintTagRefusal::AmbiguousVerbatim
            }))
        );
    }
    let source = "<template><div v-pre><template role></template></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(NativeLintRefusal::LintTag {
            reason: NativeLintTagRefusal::InheritedTemplate
        }))
    );
}

#[test]
fn original_table_context_cannot_mint_partial_header_facts() {
    for (source, depth) in [
        ("<template><table><html role></html></table></template>", 2),
        (
            "<template><math><annotation-xml encoding='text/html'><table><html role></html></table></annotation-xml></math></template>",
            4,
        ),
        (
            "<template><table><tbody><tr><td><html role></html></td></tr></tbody></table></template>",
            5,
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let before = cstr!("{:?}", owner.component().carrier());
        let mut element = owner.children().next().unwrap().into_element().unwrap();
        for _ in 1..depth {
            element = element.children().next().unwrap().into_element().unwrap();
        }
        let receipt = element.lint_tag().unwrap();
        assert!(receipt.in_table_context());
        assert_eq!(
            receipt.span(),
            vize_l0::Span::new(
                span(source, "html role").start,
                span(source, "html role").start + 4
            )
        );
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(
                NativeLintRefusal::TableContext {
                    span: receipt.span()
                }
            ))
        );
        assert!(owner.component().carrier().errors.is_empty());
        assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    }
}

#[test]
fn original_pre_recovery_discrepancies_refuse_before_any_fact_artifact() {
    for (source, path) in [
        (
            "<template><form><form v-pre><meta :role='x' /></form></form></template>",
            &[0, 0, 0][..],
        ),
        (
            "<template><p v-pre><div><meta :role='x' /></div></p></template>",
            &[0, 0, 0][..],
        ),
        (
            "<template><a v-pre><a /><meta :role='x' /></a></template>",
            &[0, 1][..],
        ),
        (
            "<template><select><option v-pre><option><meta :role='x' /></option></option></select></template>",
            &[0, 0, 0, 0][..],
        ),
        (
            "<template><select><optgroup v-pre><optgroup><meta :role='x' /></optgroup></optgroup></select></template>",
            &[0, 0, 0, 0][..],
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let before = cstr!("{:?}", owner.component().carrier());
        let mut original = owner
            .children()
            .nth(path[0])
            .unwrap()
            .into_element()
            .unwrap();
        for ordinal in &path[1..] {
            original = original
                .children()
                .nth(*ordinal)
                .unwrap()
                .into_element()
                .unwrap();
        }
        let receipt = original.lint_tag().unwrap();
        assert!(receipt.header_is_literal());
        assert!(receipt.in_recovery_context());
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.header_facts(&original).err(),
            Some(NativeHeaderFactError::Header(
                NativeLintRefusal::RecoveryContext {
                    span: receipt.span()
                }
            ))
        );
        assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    }
}

#[test]
fn original_over_limit_pre_cannot_issue_a_partial_counterexample_base() {
    let source = cstr!(
        "<template>{}<div v-pre><meta :role='x' /></div>{}</template>",
        "<div>".repeat(4096),
        "</div>".repeat(4096)
    );
    let arena = Allocator::default();
    let owner = selected(&arena, &source);
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
}
