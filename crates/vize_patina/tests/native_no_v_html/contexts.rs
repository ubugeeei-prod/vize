use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, registered, selected, span, warning,
};
use vize_l0::Allocator;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint, NativeVHtmlLintError};

#[test]
fn original_foster_and_valid_table_contexts_refuse_before_lookup() {
    for (source, foster) in [
        (
            "<template><table><html v-html></html></table></template>",
            true,
        ),
        (
            "<template><table><tbody><tr><td><Foo v-html /></td></tr></tbody></table></template>",
            false,
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let mut element = owner.children().next().unwrap().into_element().unwrap();
        for _ in 0..if foster { 1 } else { 4 } {
            element = element.children().next().unwrap().into_element().unwrap();
        }
        let receipt = element.tag_facts().unwrap();
        assert!(receipt.in_table_context());
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(
                NativeLintRefusal::TableContext {
                    span: receipt.span()
                }
            ))
        );
        for locale in LOCALES {
            let diagnostics = vec![warning(locale, span(source, "v-html"))];
            assert_eq!(complete(&registered(source, locale)), expected(diagnostics));
        }
    }
    let source = "<template><table v-html='own'></table><div v-html='outside' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                warning(locale, span(source, "v-html='own'")),
                warning(locale, span(source, "v-html='outside'"))
            ])
        );
    }
}

#[test]
fn nested_form_refuses_natively_and_lints_the_inner_form_completely() {
    // Vue keeps the nested form, so its attributes are linted and no parser
    // error is reported.
    let source = "<template><form><form v-html='x' /></form></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let outer = owner.children().next().unwrap().into_element().unwrap();
    let element = outer.children().next().unwrap().into_element().unwrap();
    let receipt = element.tag_facts().unwrap();
    assert!(receipt.in_recovery_context());
    assert_eq!(
        lint.no_v_html(&element, &NeverLookup).err(),
        Some(NativeVHtmlLintError::Header(
            NativeLintRefusal::RecoveryContext {
                span: receipt.span()
            }
        ))
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![warning(locale, span(source, "v-html='x'"))])
        );
    }
}

#[test]
fn five_pre_recovery_sources_keep_the_authored_v_pre_owner_without_parser_errors() {
    // Vue builds each template as written, so none has a parser error. Only
    // the native tag facts still treat the shape as a recovery context.
    for (source, path) in [
        (
            "<template><form><form v-pre><meta :role='x' /></form></form></template>",
            &[0, 0, 0][..],
        ),
        (
            "<template><p v-pre><div><meta :role='x' /></div></p></template>",
            &[1][..],
        ),
        (
            "<template><a v-pre><a /><meta :role='x' /></a></template>",
            &[0, 1][..],
        ),
        (
            "<template><select><option v-pre><option><meta :role='x' /></option></option></select></template>",
            &[0, 1][..],
        ),
        (
            "<template><select><optgroup v-pre><optgroup><meta :role='x' /></optgroup></optgroup></select></template>",
            &[0, 1][..],
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let mut element = owner
            .children()
            .nth(path[0])
            .unwrap()
            .into_element()
            .unwrap();
        for ordinal in &path[1..] {
            element = element
                .children()
                .nth(*ordinal)
                .unwrap()
                .into_element()
                .unwrap();
        }
        let receipt = element.tag_facts().unwrap();
        assert!(receipt.header_is_literal());
        assert!(receipt.in_recovery_context());
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(
                NativeLintRefusal::RecoveryContext {
                    span: receipt.span()
                }
            ))
        );
        for locale in LOCALES {
            assert_eq!(complete(&registered(source, locale)), expected(Vec::new()));
        }
    }
}
