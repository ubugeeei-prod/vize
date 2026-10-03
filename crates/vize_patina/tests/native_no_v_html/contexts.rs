use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, parser, registered, selected, span, warning,
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
        let receipt = element.lint_tag().unwrap();
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
            let mut diagnostics = Vec::new();
            if foster {
                diagnostics.push(parser(
                    "error",
                    "Foster parenting moved this element before the nearest open table.",
                    span(source, "<html v-html>"),
                ));
            }
            diagnostics.push(warning(locale, span(source, "v-html")));
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
fn ignored_form_refuses_and_keeps_the_complete_nonempty_registered_control() {
    let source = "<template><form><form v-html='x' /></form></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let outer = owner.children().next().unwrap().into_element().unwrap();
    let element = outer.children().next().unwrap().into_element().unwrap();
    let receipt = element.lint_tag().unwrap();
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
            expected(vec![
                parser(
                    "error",
                    "HTML tree construction ignored this start tag because an equivalent element is already open.",
                    span(source, "<form v-html='x' />")
                ),
                warning(locale, span(source, "v-html='x'"))
            ])
        );
    }
}

#[test]
fn five_original_pre_recovery_sources_keep_every_registered_parser_diagnostic() {
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
        let receipt = element.lint_tag().unwrap();
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
            let mut diagnostics = Vec::new();
            if let Some(opening) = ignored {
                diagnostics.push(parser("error", "HTML tree construction ignored this start tag because an equivalent element is already open.", span(source, opening)));
            }
            if let Some(closing) = invalid_end {
                let mut range = span(source, closing);
                range.end = range.start + u32::try_from(closing.find('>').unwrap() + 1).unwrap();
                diagnostics.push(parser("error", "Invalid end tag.", range));
            }
            assert_eq!(complete(&registered(source, locale)), expected(diagnostics));
        }
    }
}
