//! The hoist-static pass's const-classification of expression positions
//! — the P2-5b [`ExprRef`] capability consumed for real, split from
//! `pass/hoist.rs` under the source budget.
//!
//! # The pessimal law's first real consumer
//!
//! [`constant_for_hoist`] is the first S2 pass code that *acts* on
//! P2-5b's const-classification contract: an [`ExprRef::Opaque`] is
//! **never** constant ([`OpaqueExpr::is_constant`], pessimal law 3 —
//! "no folding, hoisting, or caching may be justified by an opaque
//! expression"), and this pass is the first place where that answer
//! changes a published fact rather than a document. A retained-`None`
//! bind value therefore blocks its element's hoistability, exactly as
//! the law prescribes.
//!
//! # The JS rule (self-bound locals ported, free names still weaker)
//!
//! For a retained [`ExprRef::Js`] the shipped classifier
//! (`vize_atelier_core::codegen::is_constant_simple_expression`, called
//! with `bindings: None` by `hoist_static/props.rs`) admits an
//! expression whose identifiers are locals it binds itself, allowlisted
//! globals, or helper aliases, and — a measured quirk — admits `this`.
//! This crate stays off the `vize_croquis` allowlist. The rule here: a
//! retained expression is constant iff its walk meets
//!
//! - **every identifier reference is a local the expression binds**
//!   (an arrow or function parameter, or a binding declared in its
//!   body). Free names, including allowlisted globals, stay
//!   non-constant — strictly narrower than the shipped allowlist,
//! - **no `this`** (narrower than the shipped quirk),
//! - **no TS-only construct** (`as` / `satisfies` / `<T>` assertion /
//!   `!` non-null / explicit instantiation): the shipped classifier
//!   re-parses under an **mjs** source type and refuses these,
//! - and none of the shipped classifier's four literal context
//!   substrings (`_ctx.` and kin), mirrored byte-for-byte.
//!
//! Self-bound locals are the shipped answer, so `(v) => v.toFixed(2)`
//! hoists on both lanes (P3-17 `hoist_constant_gap`). One-sidedness
//! (`constant_for_hoist` ⇒ shipped-constant) still holds: every
//! remaining divergence is an S2 *under*-hoist, counted as
//! `consts_templates` instead of compared.
//!
//! [`OpaqueExpr::is_constant`]: vize_s2::expr::OpaqueExpr::is_constant

use alloc::vec::Vec as StdVec;

use oxc_ast::ast as js;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_binding_pattern, walk_function};
use oxc_syntax::scope::ScopeFlags;
use vize_s0::camelize;
use vize_s2::expr::ExprRef;
use vize_s2::op::{Attribute, BindingOp, DynamicName, SlotOp};

/// Classify one expression position for hoisting (module docs: the
/// pessimal law on opaque payloads, the recorded weaker JS rule).
#[must_use]
pub fn constant_for_hoist(expr: &ExprRef<'_>) -> bool {
    match expr {
        // Pessimal law 3, consumed: never constant, no exceptions.
        ExprRef::Opaque(opaque) => opaque.is_constant(),
        ExprRef::Foreign(_) | ExprRef::Filter(_) => false,
        ExprRef::Js(retained) => self_bound_js_constant(retained.ast, retained.source),
    }
}

/// Why one attached binding fails the legacy `is_hoistable_static_prop`
/// rule — `None` when it survives it. The `v-bind` shape gates are
/// mirrored (static name, modifiers within `camel`/`prop`/`attr`, the
/// prefixed key not `ref`/`class`), then the value goes through
/// [`constant_for_hoist`]; every non-`ui.bind` binding is unhoistable,
/// exactly as every non-`bind` legacy directive is.
///
/// The answer is `(op mnemonic, ui.bind rule)`: a non-`ui.bind` op is its
/// own blocker, and a `ui.bind` names the first gate it failed
/// (`dynamic-name`, `modifier`, `reserved-key`, `no-value`,
/// `non-constant`). The one implementation of the rule — the fact reads
/// its `is_none` through [`props_blocker`] — so a remark can never
/// explain a decision the analysis did not make.
#[must_use]
pub(super) fn binding_blocker(
    binding: &BindingOp<'_>,
) -> Option<(&'static str, Option<&'static str>)> {
    const fn bind_rule(rule: &'static str) -> Option<(&'static str, Option<&'static str>)> {
        Some(("ui.bind", Some(rule)))
    }
    let BindingOp::Bind(bind) = binding else {
        return Some((binding.mnemonic(), None));
    };
    let Some(DynamicName::Static(name)) = &bind.name else {
        return bind_rule("dynamic-name");
    };
    let mut has_camel = false;
    let mut has_prop = false;
    let mut has_attr = false;
    for modifier in bind.modifiers.iter() {
        match *modifier {
            "camel" => has_camel = true,
            "prop" => has_prop = true,
            "attr" => has_attr = true,
            _ => return bind_rule("modifier"),
        }
    }
    // The legacy precedence chain, mirrored: `camel` wins over `prop`
    // over `attr`, and the *prefixed* key is what the ref/class check
    // reads (`.ref` deliberately passes — the shipped quirk).
    let key = if has_camel {
        camelize(name)
    } else if has_prop {
        prefixed('.', name)
    } else if has_attr {
        prefixed('^', name)
    } else {
        vize_s0::ToCompactString::to_compact_string(name)
    };
    if matches!(key.as_str(), "ref" | "class") {
        return bind_rule("reserved-key");
    }
    let Some(value) = &bind.value else {
        return bind_rule("no-value");
    };
    if constant_for_hoist(value) {
        None
    } else {
        bind_rule("non-constant")
    }
}

fn prefixed(prefix: char, name: &str) -> vize_s0::String {
    let mut out = vize_s0::String::with_capacity(1 + name.len());
    out.push(prefix);
    out.push_str(name);
    out
}

/// Whether a retained JS expression is constant under the self-bound
/// rule (module docs). The DOM props hoist spells the same answer.
#[must_use]
pub(crate) fn self_bound_js_constant(expr: &js::Expression<'_>, source: &str) -> bool {
    if source.contains("_ctx.")
        || source.contains("$setup.")
        || source.contains("__props.")
        || source.contains("$props.")
    {
        return false;
    }
    let mut walk = ConstWalk {
        locals: StdVec::new(),
        dynamic: false,
    };
    walk.visit_expression(expr);
    !walk.dynamic
}

/// The retained-AST walk. A reference is constant only when the
/// expression itself binds that name; `this` and TS-only constructs
/// stay non-constant (module docs).
struct ConstWalk {
    locals: StdVec<StdVec<vize_s0::String>>,
    dynamic: bool,
}

impl ConstWalk {
    fn is_local(&self, name: &str) -> bool {
        self.locals
            .iter()
            .any(|frame| frame.iter().any(|bound| bound.as_str() == name))
    }

    fn bind_pattern(&mut self, pattern: &js::BindingPattern<'_>) {
        match pattern {
            js::BindingPattern::BindingIdentifier(ident) => {
                if let Some(frame) = self.locals.last_mut() {
                    frame.push(vize_s0::String::from(ident.name.as_str()));
                }
            }
            js::BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.bind_pattern(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            js::BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.bind_pattern(element);
                }
                if let Some(rest) = &array.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            js::BindingPattern::AssignmentPattern(assignment) => {
                self.bind_pattern(&assignment.left);
            }
        }
    }
}

impl<'a> Visit<'a> for ConstWalk {
    fn visit_identifier_reference(&mut self, ident: &js::IdentifierReference<'a>) {
        if !self.is_local(ident.name.as_str()) {
            self.dynamic = true;
        }
    }

    fn visit_this_expression(&mut self, _this: &js::ThisExpression) {
        self.dynamic = true;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &js::ArrowFunctionExpression<'a>) {
        self.locals.push(StdVec::new());
        for param in &arrow.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_arrow_function_expression(self, arrow);
        self.locals.pop();
    }

    fn visit_function(&mut self, function: &js::Function<'a>, flags: ScopeFlags) {
        self.locals.push(StdVec::new());
        for param in &function.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_function(self, function, flags);
        self.locals.pop();
    }

    fn visit_variable_declarator(&mut self, declarator: &js::VariableDeclarator<'a>) {
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.bind_pattern(&declarator.id);
        // Types and default values live on the pattern. Walking them
        // after the bind keeps a TS annotation dynamic (the shipped
        // mjs re-parse refuses it) without treating the bound name as
        // a free reference.
        walk_binding_pattern(self, &declarator.id);
    }

    fn visit_ts_as_expression(&mut self, _expr: &js::TSAsExpression<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_satisfies_expression(&mut self, _expr: &js::TSSatisfiesExpression<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_type_assertion(&mut self, _expr: &js::TSTypeAssertion<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_non_null_expression(&mut self, _expr: &js::TSNonNullExpression<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_instantiation_expression(&mut self, _expr: &js::TSInstantiationExpression<'a>) {
        self.dynamic = true;
    }
}

/// `props_are_static_attrs` / the prop half of `has_static_props`:
/// every attribute except `ref`, every binding through the mirrored
/// `is_hoistable_static_prop` rule. Defined as [`props_blocker`]'s
/// `is_none`, so the remark and the fact share one rule.
#[must_use]
pub(super) fn props_all_hoistable(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> bool {
    props_blocker(attributes, bindings).is_none()
}

/// The first thing failing [`props_all_hoistable`]: a `ref` attribute,
/// else the first unhoistable binding's [`binding_blocker`].
#[must_use]
pub(super) fn props_blocker(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Option<PropsBlocker> {
    if attributes.iter().any(|attribute| attribute.name == "ref") {
        return Some(PropsBlocker::RefAttribute);
    }
    bindings
        .iter()
        .find_map(binding_blocker)
        .map(|(op, rule)| PropsBlocker::Binding { op, rule })
}

/// Why a props surface is not hoistable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PropsBlocker {
    /// A static `ref` attribute.
    RefAttribute,
    /// A binding failing the hoist rule: its mnemonic, and for `ui.bind`
    /// the shape gate it failed.
    Binding {
        op: &'static str,
        rule: Option<&'static str>,
    },
}

/// The outlet's static-nested-child rule: its props surface (the
/// separated `name` position included) must be fully static.
#[must_use]
pub(super) fn slot_props_static(slot: &SlotOp<'_>) -> bool {
    let name_static = match &slot.name {
        DynamicName::Static(_) => true,
        DynamicName::Dynamic(expr) => constant_for_hoist(expr),
    };
    name_static && props_all_hoistable(&slot.attributes, &slot.bindings)
}
