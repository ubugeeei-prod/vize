//! Retained-expression consumption gates (Davinci P1-7).
//!
//! P1-5 parses every template expression once, at template parse, into the
//! compile's shared oxc arena (`SimpleExpressionNode::js_ast`, TS dialect,
//! `SourceType::ts()`). The atelier transform/codegen sites migrated in P1-7
//! read that retained AST instead of re-parsing — but only where equality
//! with the legacy re-parse is a parser-determinism fact:
//!
//! - [`retained_whole_expression`] gates on `raw == content`: the retained
//!   AST must describe the node's *current* bytes. Any transform that
//!   rewrote `content` after parse (prefixing, filters, TS stripping) fails
//!   the gate and the site falls back to its legacy parse.
//! - [`js_module_compatible`] gates on dialect: the legacy sites parse with
//!   `SourceType::default().with_module(true)` (plain JS, module goal) while
//!   the retained parse is TS with the unambiguous module goal. The scan
//!   accepts an AST only when no construct is present whose parse *could*
//!   differ between those dialects — every TS-flavored node or field, every
//!   strict/module-reserved word used as an identifier, sloppy-only literal
//!   forms (legacy octal, `\digit` string escapes), HTML-like comment
//!   markers, `with`, `delete <ident>`, and whole `function`/`class`
//!   expressions (conservatively: their bodies re-open the strictness
//!   surface). For an accepted AST, the JS-module parse of the same bytes is
//!   the same parse, so consuming the retained AST is byte-equivalent to the
//!   legacy re-parse it replaces. Everything rejected keeps the legacy
//!   parse — fallback classes are recorded in `plan/phase-1.md` P1-7.
//!
//! Sites whose legacy dialect already *is* TS (the vapor generate resolver)
//! need only the module-goal half of the scan; they reuse the same predicate,
//! deliberately over-conservative there (TS-syntax expressions stay on the
//! vapor fallback parse — recorded).
//!
//! Under `cfg(any(test, feature = "davinci-differential"))` every gated
//! consumption is dual-run against the legacy parse it replaced and
//! divergence panics — the P1-6 differential-lane pattern, counters in
//! [`differential`]. The dual-run legacy parses use their own uncounted
//! arenas (`oxc_allocator::Allocator::default()`, not
//! [`crate::expr_parse_probe::parse_arena`]): they exist only while the lane
//! is armed and must not disturb the production re-parse floor the P1-7
//! acceptance pins.
//!
//! Temporary like the probe: P1-9 already drives the transform prefixer
//! from the retained AST on admitted inputs (span splicing, guard re-scans
//! skipped); the gates and the remaining legacy chains collapse together
//! once the recorded residual classes shrink, and this module shrinks with
//! them.

use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;
use vize_relief::{JsExpression, SimpleExpressionNode};

/// The node's retained AST, iff it still describes the node's exact bytes.
///
/// `raw` is the text the AST was parsed from (P1-5: equal to the content at
/// parse time), so `raw == content` means no transform rewrote the content
/// since — the self-validating staleness gate.
#[inline]
pub fn retained_whole_expression<'n, 'a>(
    node: &'n SimpleExpressionNode<'a>,
) -> Option<&'n JsExpression<'a>> {
    let js = node.js_ast.as_ref()?;
    (js.raw == node.content).then_some(js)
}

/// True when the retained TS/unambiguous parse of `js.raw` is provably the
/// same parse a `SourceType::default().with_module(true)` (JS, module goal)
/// site would produce — see the module docs for the exact reject classes.
pub fn js_module_compatible(js: &JsExpression<'_>) -> bool {
    // Lexer-level divergence: outside a module goal the lexer treats
    // `<!--`/`-->` as HTML-like comments, so token boundaries themselves can
    // differ. Reject on the raw bytes before trusting the AST at all.
    if js.raw.contains("<!--") || js.raw.contains("-->") {
        return false;
    }
    let mut scan = JsModuleCompatScan { compatible: true };
    scan.visit_expression(js.ast);
    scan.compatible
}

/// Identifiers that parse in the retained sloppy/unambiguous goal but are
/// reserved (or restricted) under the strict module goal. `eval` and
/// `arguments` are only restricted in write/binding position; rejecting
/// every use is deliberate over-approximation — the cost is fallback
/// coverage, never correctness.
fn strict_mode_divergent_identifier(name: &str) -> bool {
    matches!(
        name,
        "implements"
            | "interface"
            | "let"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "static"
            | "yield"
            | "await"
            | "eval"
            | "arguments"
    )
}

/// Legacy-octal / non-decimal-looking numeric raw (`010`, `08`): sloppy-only.
fn sloppy_only_numeric_raw(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    bytes.len() > 1 && bytes[0] == b'0' && bytes[1].is_ascii_digit()
}

/// `\<digit>` escapes (`\05`, `\8`) are sloppy-only in string literals.
/// `\0` alone is legal everywhere, but the conservative scan rejects it too.
fn sloppy_only_string_raw(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'\\' {
            if bytes[index + 1].is_ascii_digit() {
                return true;
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    false
}

struct JsModuleCompatScan {
    compatible: bool,
}

impl JsModuleCompatScan {
    #[inline]
    fn reject(&mut self) {
        self.compatible = false;
    }
}

impl<'a> Visit<'a> for JsModuleCompatScan {
    // --- TS-flavored nodes: the TS parser resolved an ambiguity (or TS-only
    // grammar) the JS parse cannot reproduce.
    fn visit_ts_as_expression(&mut self, _it: &oxc_ast_types::TSAsExpression<'a>) {
        self.reject();
    }
    fn visit_ts_satisfies_expression(&mut self, _it: &oxc_ast_types::TSSatisfiesExpression<'a>) {
        self.reject();
    }
    fn visit_ts_type_assertion(&mut self, _it: &oxc_ast_types::TSTypeAssertion<'a>) {
        self.reject();
    }
    fn visit_ts_non_null_expression(&mut self, _it: &oxc_ast_types::TSNonNullExpression<'a>) {
        self.reject();
    }
    fn visit_ts_instantiation_expression(
        &mut self,
        _it: &oxc_ast_types::TSInstantiationExpression<'a>,
    ) {
        self.reject();
    }
    fn visit_ts_type_annotation(&mut self, _it: &oxc_ast_types::TSTypeAnnotation<'a>) {
        self.reject();
    }
    fn visit_ts_type_parameter_declaration(
        &mut self,
        _it: &oxc_ast_types::TSTypeParameterDeclaration<'a>,
    ) {
        self.reject();
    }
    fn visit_ts_type_parameter_instantiation(
        &mut self,
        _it: &oxc_ast_types::TSTypeParameterInstantiation<'a>,
    ) {
        self.reject();
    }
    fn visit_ts_enum_declaration(&mut self, _it: &oxc_ast_types::TSEnumDeclaration<'a>) {
        self.reject();
    }
    fn visit_decorator(&mut self, _it: &oxc_ast_types::Decorator<'a>) {
        self.reject();
    }

    // --- Constructs that re-open the sloppy/strict surface wholesale.
    fn visit_function(&mut self, _it: &oxc_ast_types::Function<'a>, _flags: ScopeFlags) {
        self.reject();
    }
    fn visit_class(&mut self, _it: &oxc_ast_types::Class<'a>) {
        self.reject();
    }
    fn visit_with_statement(&mut self, _it: &oxc_ast_types::WithStatement<'a>) {
        self.reject();
    }

    // --- Arrow params: `(x?) => e` and friends carry TS-ness in flags, not
    // child nodes.
    fn visit_formal_parameter(&mut self, it: &oxc_ast_types::FormalParameter<'a>) {
        if !it.decorators.is_empty()
            || it.type_annotation.is_some()
            || it.optional
            || it.accessibility.is_some()
            || it.readonly
            || it.r#override
        {
            self.reject();
            return;
        }
        walk::walk_formal_parameter(self, it);
    }

    // --- Strict/module-reserved words used as identifiers.
    fn visit_identifier_reference(&mut self, it: &oxc_ast_types::IdentifierReference<'a>) {
        if strict_mode_divergent_identifier(it.name.as_str()) {
            self.reject();
        }
    }
    fn visit_binding_identifier(&mut self, it: &oxc_ast_types::BindingIdentifier<'a>) {
        if strict_mode_divergent_identifier(it.name.as_str()) {
            self.reject();
        }
    }
    fn visit_label_identifier(&mut self, it: &oxc_ast_types::LabelIdentifier<'a>) {
        if strict_mode_divergent_identifier(it.name.as_str()) {
            self.reject();
        }
    }

    // --- Sloppy-only literal spellings.
    fn visit_numeric_literal(&mut self, it: &oxc_ast_types::NumericLiteral<'a>) {
        if it
            .raw
            .as_ref()
            .is_some_and(|raw| sloppy_only_numeric_raw(raw))
        {
            self.reject();
        }
    }
    fn visit_string_literal(&mut self, it: &oxc_ast_types::StringLiteral<'a>) {
        if it
            .raw
            .as_ref()
            .is_some_and(|raw| sloppy_only_string_raw(raw))
        {
            self.reject();
        }
    }

    // --- `delete identifier` is a strict-mode SyntaxError.
    fn visit_unary_expression(&mut self, it: &oxc_ast_types::UnaryExpression<'a>) {
        if it.operator == oxc_ast_types::UnaryOperator::Delete
            && matches!(
                it.argument.get_inner_expression(),
                oxc_ast_types::Expression::Identifier(_)
            )
        {
            self.reject();
            return;
        }
        walk::walk_unary_expression(self, it);
    }
}

/// Dual-run instrumentation for the differential lane (see module docs);
/// P1-9 adds the transform-lane legacy classification counters there.
pub mod differential;

#[cfg(test)]
mod tests;
