use super::rewrite::rewrite_expression;
use super::scope::PrefixScope;
use super::strip::strip_scope_prefixes_for_slot_params;
use crate::emit::options::{BindingKind, BindingTable};

#[test]
fn a_destructured_for_alias_keeps_object_shorthand() {
    let table = BindingTable::new([("open", BindingKind::SetupConst)], [], true);
    let mut scope = PrefixScope::new(Some(&table), true, false, false);
    scope.push_for([Some("{ id, description }"), None, None]);
    let content = "open({\n  id,\n  description: description ?? '',\n})";
    let rewritten = rewrite_expression(content, None, &scope, false);
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
    let mut exprs = crate::emit::TransformExpressions::new("fixture", Some(&table), true, false);
    let _mark = exprs.enter_for([Some("[name, extensions]"), None, None]);
    let content = "extensions.map((ext) => `.${ext}`).join(\"、\")";
    let rewritten = exprs.text(content).expect("rewrite");
    assert!(rewritten.text.contains('、'), "text={:?}", rewritten.text);
}
