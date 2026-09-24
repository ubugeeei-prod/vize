#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use crate::{JsxLang, lower_source, lower_source_for_typecheck};
use vize_s0::{Allocator, cstr};

#[test]
fn incomplete_members_preserve_diagnostics_and_all_authored_root_spans() {
    for expression in ["api.", "getApi().", "api?.", "api[0]."] {
        let source = cstr!(
            "const 前 = '😀'; const first = <div>{{{expression}}}</div>; const next = <span>{{前}}</span>;"
        );
        let allocator = Allocator::new();
        let strict = lower_source(&allocator, allocator.as_oxc(), &source, JsxLang::Tsx);
        assert!(strict.has_errors());
        let recovered =
            lower_source_for_typecheck(&allocator, allocator.as_oxc(), &source, JsxLang::Tsx);
        assert_eq!(recovered.diagnostics, strict.diagnostics);
        assert_eq!(
            recovered.roots.len(),
            2,
            "{expression}: {:?}",
            strict.diagnostics
        );
        for (root, expected) in recovered.roots.iter().zip([
            cstr!("<div>{{{expression}}}</div>"),
            "<span>{前}</span>".into(),
        ]) {
            let span = root.root.loc.span;
            assert_eq!(
                &source[span.start as usize..span.end as usize],
                expected.as_str()
            );
        }
    }
}

#[test]
fn recovery_does_not_reinterpret_spreads_strings_or_other_syntax_errors() {
    for source in [
        "const node = <div>{...}</div>",
        "const node = <div>{'api.}</div>",
        "const value = ;",
    ] {
        let allocator = Allocator::new();
        let strict = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Tsx);
        let recovered =
            lower_source_for_typecheck(&allocator, allocator.as_oxc(), source, JsxLang::Tsx);
        assert!(strict.has_errors());
        assert_eq!(recovered.diagnostics, strict.diagnostics);
        assert_eq!(recovered.roots.len(), strict.roots.len());
    }
}
