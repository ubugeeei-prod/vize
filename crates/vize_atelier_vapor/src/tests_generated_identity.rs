//! Compare reviewed generated names by resolved symbols, preserving authorship.

use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, CallExpression, Expression, IdentifierReference,
    ObjectProperty, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::{SourceType, Span};
use oxc_syntax::symbol::SymbolId;
use vize_carton::{Allocator, FxHashMap, String, cstr};

pub(crate) fn normalized(code: &str) -> String {
    let allocator = Allocator::new();
    let parsed = Parser::new(allocator.as_oxc(), code, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let built = SemanticBuilder::new().build(&parsed.program);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    let scoping = built.semantic.scoping();
    let mut prefix = String::from("__davinci_comparison_");
    while code.contains(prefix.as_str())
        || scoping
            .symbol_names()
            .any(|name| name.starts_with(prefix.as_str()))
        || scoping
            .root_unresolved_references()
            .keys()
            .any(|name| name.as_str().starts_with(prefix.as_str()))
    {
        prefix.push('_');
    }
    let mut names = Names {
        prefix: prefix.as_str(),
        symbols: FxHashMap::default(),
        next: 0,
    };
    names.visit_program(&parsed.program);
    let mut edits = Edits {
        scoping,
        symbols: &names.symbols,
        spans: vec![],
    };
    edits.visit_program(&parsed.program);
    edits.spans.sort_by_key(|(span, _)| span.start);
    edits.spans.dedup_by_key(|(span, _)| (span.start, span.end));
    let mut output = String::default();
    let mut cursor = 0;
    for (span, name) in edits.spans {
        output.push_str(code.get(cursor..span.start as usize).unwrap());
        output.push_str(&name);
        cursor = span.end as usize;
    }
    output.push_str(code.get(cursor..).unwrap());
    output
}

fn numbered(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix).is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

struct Names<'p> {
    prefix: &'p str,
    symbols: FxHashMap<SymbolId, String>,
    next: usize,
}

impl<'a> Visit<'a> for Names<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        assert!(
            !matches!(&call.callee, Expression::Identifier(callee) if callee.name == "eval"),
            "direct eval can observe generated binding names"
        );
        walk::walk_call_expression(self, call);
    }

    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(identifier) = &declaration.id
            && (numbered(identifier.name.as_str(), "n") || numbered(identifier.name.as_str(), "x"))
            && let Some(Expression::CallExpression(call)) = &declaration.init
            && let Expression::Identifier(callee) = &call.callee
            && (numbered(callee.name.as_str(), "t")
                || matches!(
                    callee.name.as_str(),
                    "_child"
                        | "_next"
                        | "_txt"
                        | "_createIf"
                        | "_createFor"
                        | "_createComponent"
                        | "_createComponentWithFallback"
                        | "_createDynamicComponent"
                        | "_createSlot"
                ))
        {
            let symbol = identifier.symbol_id.get().expect("semantic binding");
            self.symbols
                .insert(symbol, cstr!("{}{}", self.prefix, self.next));
            self.next += 1;
        }
        walk::walk_variable_declarator(self, declaration);
    }
}

struct Edits<'s> {
    scoping: &'s Scoping,
    symbols: &'s FxHashMap<SymbolId, String>,
    spans: std::vec::Vec<(Span, String)>,
}

impl Edits<'_> {
    fn record(&mut self, symbol: Option<SymbolId>, span: Span) {
        if let Some(name) = symbol.and_then(|symbol| self.symbols.get(&symbol)) {
            self.spans.push((span, name.clone()));
        }
    }
}

impl<'a> Visit<'a> for Edits<'_> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        // A shorthand authors its key as well as reading the resolved value.
        if property.shorthand
            && let Expression::Identifier(identifier) = &property.value
            && let Some(symbol) = identifier
                .reference_id
                .get()
                .and_then(|reference| self.scoping.get_reference(reference).symbol_id())
            && let Some(name) = self.symbols.get(&symbol)
        {
            self.spans
                .push((identifier.span, cstr!("{}: {}", identifier.name, name)));
        } else {
            walk::walk_object_property(self, property);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.record(identifier.symbol_id.get(), identifier.span);
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        let symbol = identifier
            .reference_id
            .get()
            .and_then(|reference| self.scoping.get_reference(reference).symbol_id());
        self.record(symbol, identifier.span);
    }
}

#[test]
fn generated_identity_normalization_preserves_literals_properties_and_shadowing() {
    let native = r#"function render(_ctx) { const n1 = t0(); const n2 = _child(n1); const x2 = _txt(n2); read(_ctx.n1, "n1", n2, x2); return (n1) => n1; }"#;
    let retained = r#"function render(_ctx) { const n6 = t0(); const n5 = _child(n6); const x5 = _txt(n5); read(_ctx.n1, "n1", n5, x5); return (n1) => n1; }"#;
    let actual = normalized(native);
    assert_eq!(actual, normalized(retained));
    for authored in ["_ctx.n1", "\"n1\"", "(n1) => n1"] {
        assert!(actual.contains(authored), "{actual}");
    }
    assert_ne!(
        actual,
        normalized(&retained.replace("_child(n6)", "_child(n5)"))
    );
    assert_ne!(actual, normalized(&retained.replace("_ctx.n1", "_ctx.n6")));
    assert_ne!(actual, normalized(&retained.replace("\"n1\"", "\"n6\"")));
}

#[test]
fn generated_name_comparison_preserves_authored_shorthand_keys() {
    let code = "function render() { const n1 = t0(); return {n1}; }";
    assert!(normalized(code).contains("{n1: "));
    assert_ne!(normalized(code), normalized(&code.replace("n1", "n6")));
}

#[test]
fn generated_shorthand_normalization_keeps_the_resolved_node_reference() {
    let parent = "function render() { const n1 = t0(); const n2 = _child(n1); return {n1}; }";
    let child = "function render() { const n6 = t0(); const n1 = _child(n6); return {n1}; }";
    assert_ne!(normalized(parent), normalized(child));
}

#[test]
#[should_panic(expected = "direct eval can observe generated binding names")]
fn generated_identity_comparison_refuses_direct_eval() {
    normalized("function render() { const n1 = t0(); return eval('n1'); }");
}

#[test]
fn generated_identity_comparison_avoids_escaped_binding_and_global_capture() {
    for code in [
        r"function render() { const __davinci_comparison\u005f0 = 7; const n1 = t0(); return [n1, __davinci_comparison\u005f0]; }",
        r"function render() { const n1 = t0(); return [n1, __davinci_comparison\u005f0]; }",
    ] {
        assert_eq!(
            normalized(code),
            code.replace("n1", "__davinci_comparison__0")
        );
    }
}
