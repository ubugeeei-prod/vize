use super::rewrite::rewrite_expression;
use super::scope::PrefixScope;
use super::strip::strip_scope_prefixes_for_slot_params;
use crate::targets::dom::artifact::options::{BindingKind, BindingTable};

#[test]
fn a_destructured_for_alias_keeps_object_shorthand() {
    let table = BindingTable::new([("open", BindingKind::SetupConst)], [], true);
    let mut scope = PrefixScope::new(Some(&table), true, false, false);
    scope.push_for([Some("{ id, description }"), None, None]);
    let content = "open({\n  id,\n  description: description ?? '',\n})";
    let content = super::Content {
        text: crate::targets::dom::artifact::js_comment::RawJs::Borrowed(content),
        offset: None,
    };
    let rewritten = rewrite_expression(&super::erase(&scope, &content, None), &scope, false);
    let stripped = strip_scope_prefixes_for_slot_params(&scope, rewritten.code.as_str());
    assert!(
        stripped.contains("$setup.open"),
        "setup binding should be prefixed: {stripped}"
    );
    assert!(
        stripped.contains("id,") && !stripped.contains("id: id"),
        "v-for alias shorthand must survive the scope strip: {stripped}"
    );
}

#[test]
fn an_ideographic_comma_inside_a_string_survives_the_rewrite() {
    let table = BindingTable::new([("rows", BindingKind::SetupConst)], [], true);
    let mut exprs = crate::targets::dom::artifact::TransformExpressions::new(
        "fixture",
        Some(&table),
        true,
        false,
    );
    let _mark = exprs.enter_for([Some("[name, extensions]"), None, None]);
    let content = "extensions.map((ext) => `.${ext}`).join(\"、\")";
    let rewritten = exprs.text(content).expect("rewrite");
    assert!(rewritten.text.contains('、'), "text={:?}", rewritten.text);
}

#[test]
fn assignment_shorthand_uses_the_target_write_policy() {
    let cases = [
        (BindingKind::SetupLet, false, "x: $setup.x"),
        (BindingKind::SetupMaybeRef, false, "x: $setup.x"),
        (BindingKind::SetupRef, false, "x: $setup.x"),
        (BindingKind::SetupConst, true, "x"),
        (BindingKind::SetupLet, true, "x: x.value"),
        (BindingKind::SetupMaybeRef, true, "x: x.value"),
        (BindingKind::SetupRef, true, "x: x.value"),
        (BindingKind::SetupReactiveConst, true, "x"),
        (BindingKind::LiteralConst, true, "x"),
        (BindingKind::Props, false, "x: $props.x"),
        (BindingKind::PropsAliased, false, "x: $props.x"),
        (BindingKind::Data, false, "x: $data.x"),
        (BindingKind::Options, false, "x: $options.x"),
    ];
    for (kind, inline, target) in cases {
        let table = BindingTable::new([("x", kind), ("source", BindingKind::SetupConst)], [], true);
        let scope = PrefixScope::new(Some(&table), true, false, inline);
        let source = if inline { "source" } else { "$setup.source" };
        let expected = vize_l0::cstr!("({{ {target} }} = {source})");
        let authored = "({ x } = source)";
        let content = super::Content {
            text: crate::targets::dom::artifact::js_comment::RawJs::Borrowed(authored),
            offset: None,
        };
        let rewritten = rewrite_expression(&super::erase(&scope, &content, None), &scope, false);
        assert_eq!(rewritten.code.as_str(), expected.as_str());
        assert_eq!(
            super::codegen_visitor::prefix_identifiers_with_context(authored, &scope).as_str(),
            expected.as_str()
        );

        let authored = "() => { let x; ({ x } = source); return x }";
        let content = super::Content {
            text: crate::targets::dom::artifact::js_comment::RawJs::Borrowed(authored),
            offset: None,
        };
        let expected = vize_l0::cstr!("() => {{ let x; ({{ x }} = {source}); return x }}");
        let rewritten = rewrite_expression(&super::erase(&scope, &content, None), &scope, false);
        assert_eq!(rewritten.code.as_str(), expected.as_str());
        assert_eq!(
            super::codegen_visitor::prefix_identifiers_with_context(authored, &scope).as_str(),
            expected.as_str()
        );
    }
}
