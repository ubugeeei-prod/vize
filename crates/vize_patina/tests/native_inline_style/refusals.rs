use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, parser, registered, selected, span, warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn later_dynamic_object_custom_and_empty_modifier_heads_refuse_whole_headers() {
    for (head, unresolved) in [
        (":[key]", true),
        (".[key]", true),
        ("@[key]", true),
        ("v-bind:[key]", true),
        ("v-bind", true),
        ("v-bind.prop", true),
        ("v-unknown", false),
        (":title.", false),
    ] {
        let source = cstr!("<template><Foo style='yes' {head}='broken(' /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let range = span(&source, head);
        let refusal = if unresolved {
            NativeLintRefusal::UnresolvedBinding { span: range }
        } else {
            NativeLintRefusal::UnsupportedDirective { span: range }
        };
        assert_eq!(
            lint.no_inline_style(&element, &NeverLookup).err(),
            Some(refusal)
        );
        for locale in LOCALES {
            let mut diagnostics = vec![warning(locale, span(&source, "style='yes'"))];
            if head == ":title." {
                diagnostics.push(parser(
                    "error",
                    "Directive modifier is expected.",
                    Span::new(range.end, range.end + 1),
                ));
            }
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(diagnostics)
            );
        }
    }
}

#[test]
fn duplicate_attributes_keep_every_original_warning_and_parser_notice() {
    for (header, repeated, second_style) in [
        ("style='one' style='two'", "style", true),
        ("style='one' STYLE='two'", "STYLE", false),
        ("style='one' id='one' ID='two'", "ID", false),
    ] {
        let source = cstr!("<template><Foo {header} /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let start = u32::try_from(source.rfind(repeated).unwrap()).unwrap();
        let range = Span::new(start, start + u32::try_from(repeated.len()).unwrap());
        assert_eq!(
            lint.no_inline_style(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::DuplicateAttribute { span: range })
        );
        let message = cstr!(
            "Duplicate attribute `{repeated}`. Keeping the repeated attribute so parsing can continue."
        );
        for locale in LOCALES {
            let mut diagnostics = vec![
                warning(locale, span(&source, "style='one'")),
                parser("error", &message, range),
            ];
            if second_style {
                diagnostics.push(warning(locale, span(&source, "style='two'")));
            }
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(diagnostics)
            );
        }
    }
}

#[test]
fn original_foster_and_valid_table_contexts_refuse_before_lookup() {
    for (source, foster) in [
        (
            "<template><table><html style></html></table></template>",
            true,
        ),
        (
            "<template><table><tbody><tr><td><Foo style /></td></tr></tbody></table></template>",
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
            lint.no_inline_style(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::TableContext {
                span: receipt.span()
            })
        );
        for locale in LOCALES {
            let diagnostics = vec![warning(locale, span(source, "style"))];
            assert_eq!(complete(&registered(source, locale)), expected(diagnostics));
        }
    }
    let source = "<template><table style='own'></table><div style='outside' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                warning(locale, span(source, "style='own'")),
                warning(locale, span(source, "style='outside'"))
            ])
        );
    }
}

#[test]
fn nested_form_refuses_natively_and_lints_the_inner_form_completely() {
    // Vue keeps the nested form, so its attributes are linted and no parser
    // error is reported.
    let source = "<template><form><form style='x' /></form></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let outer = owner.children().next().unwrap().into_element().unwrap();
    let element = outer.children().next().unwrap().into_element().unwrap();
    let receipt = element.tag_facts().unwrap();
    assert!(receipt.in_recovery_context());
    assert_eq!(
        lint.no_inline_style(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::RecoveryContext {
            span: receipt.span()
        })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![warning(locale, span(source, "style='x'"))])
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
            lint.no_inline_style(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::RecoveryContext {
                span: receipt.span()
            })
        );
        for locale in LOCALES {
            assert_eq!(complete(&registered(source, locale)), expected(Vec::new()));
        }
    }
}
