#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use crate::{JsxLang, lower_source, lower_source_for_typecheck};
use vize_l0::{Allocator, cstr};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

#[test]
fn typecheck_slot_parameters_retain_authored_types_and_utf8_spans() {
    for (slot, authored) in [
        (
            "(props: { value: string }) => props.value",
            "props: { value: string }",
        ),
        (
            "function(props: { value: string }) { return props.value; }",
            "props: { value: string }",
        ),
        (
            "({ value }: { value: string }) => value",
            "{ value }: { value: string }",
        ),
        (
            "(props?: { value: string }) => props?.value",
            "props?: { value: string }",
        ),
        (
            "(props: { value: string } = { value: 'fallback' }) => props.value",
            "props: { value: string } = { value: 'fallback' }",
        ),
    ] {
        for children in [
            cstr!("{{ item: {slot} }}"),
            cstr!("{{ ['item']: {slot} }}"),
            slot.into(),
        ] {
            let source = cstr!("const 前 = '😀';\r\nconst view = <Host>{{{children}}}</Host>;");
            let allocator = Allocator::new();
            let output =
                lower_source_for_typecheck(&allocator, allocator.as_oxc(), &source, JsxLang::Tsx);
            assert!(!output.has_errors(), "{slot}: {:?}", output.diagnostics);
            let TemplateChildNode::Element(host) = &output.roots[0].root.children[0] else {
                panic!("host")
            };
            let TemplateChildNode::Element(template) = &host.children[0] else {
                panic!("slot")
            };
            let PropNode::Directive(slot) = &template.props[0] else {
                panic!("slot directive")
            };
            let Some(ExpressionNode::Simple(parameter)) = &slot.exp else {
                panic!("slot parameter")
            };
            assert_eq!(parameter.content, authored);
            assert_eq!(
                &source[parameter.loc.span.start as usize..parameter.loc.span.end as usize],
                authored
            );
        }
    }
}

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
