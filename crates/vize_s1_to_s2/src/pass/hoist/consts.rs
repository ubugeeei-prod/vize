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
//! # The deliberately weaker JS rule (recorded, counted)
//!
//! For a retained [`ExprRef::Js`] the shipped classifier
//! (`vize_atelier_core::codegen::is_constant_simple_expression`, called
//! with `bindings: None` by `hoist_static/props.rs`) admits an
//! expression whose free identifiers are all in `vize_croquis`'s global
//! allowlist, and — a measured quirk — admits `this`-expressions, since
//! its visitor inspects identifier references only. This crate is
//! `no_std + alloc` and must not grow a `vize_croquis` (std) edge, and
//! the #4365 precedent (pattern-params scope names) is to record a
//! **strictly weaker** rule loudly rather than duplicate a contested
//! scanner. The rule here: a retained expression is constant iff its
//! walk meets
//!
//! - **no identifier reference at all** (strictly narrower than the
//!   allowlist: every allowlisted global is an identifier),
//! - **no `this`** (narrower than the shipped quirk),
//! - **no TS-only construct** (`as` / `satisfies` / `<T>` assertion /
//!   `!` non-null / explicit instantiation): the shipped classifier
//!   re-parses under an **mjs** source type and refuses these, and the
//!   exclusion is what keeps this rule one-sided against it,
//! - and none of the shipped classifier's four literal context
//!   substrings (`_ctx.` and kin), mirrored byte-for-byte — they refuse
//!   even inside string literals, and imitating that keeps string-only
//!   payloads decision-equal.
//!
//! One-sidedness (`constant_for_hoist` ⇒ shipped-constant) is the
//! designed invariant: every divergence is an S2 *under*-hoist, which
//! the differential lane counts as one class (`consts_templates`)
//! instead of comparing — measured, never averaged.
//!
//! [`OpaqueExpr::is_constant`]: vize_s2::expr::OpaqueExpr::is_constant

use oxc_ast::ast as js;
use oxc_ast_visit::Visit;
use vize_s0::camelize;
use vize_s2::expr::ExprRef;
use vize_s2::op::{BindingOp, DynamicName};

/// Classify one expression position for hoisting (module docs: the
/// pessimal law on opaque payloads, the recorded weaker JS rule).
#[must_use]
pub fn constant_for_hoist(expr: &ExprRef<'_>) -> bool {
    match expr {
        // Pessimal law 3, consumed: never constant, no exceptions.
        ExprRef::Opaque(opaque) => opaque.is_constant(),
        ExprRef::Foreign(_) | ExprRef::Filter(_) => false,
        ExprRef::Js(retained) => {
            let source = retained.source;
            // The shipped classifier's literal substring refusals,
            // mirrored byte-for-byte (they apply inside string
            // literals too, deliberately).
            if source.contains("_ctx.")
                || source.contains("$setup.")
                || source.contains("__props.")
                || source.contains("$props.")
            {
                return false;
            }
            let mut walk = ConstWalk { dynamic: false };
            walk.visit_expression(retained.ast);
            !walk.dynamic
        }
    }
}

/// Whether one attached binding survives the legacy
/// `is_hoistable_static_prop` rule — the `v-bind` shape gates mirrored
/// (static name, modifiers within `camel`/`prop`/`attr`, the prefixed
/// key not `ref`/`class`), then the value through
/// [`constant_for_hoist`]. Every non-`ui.bind` binding is unhoistable,
/// exactly as every non-`bind` legacy directive is.
#[must_use]
pub(super) fn hoistable_binding(binding: &BindingOp<'_>) -> bool {
    let BindingOp::Bind(bind) = binding else {
        return false;
    };
    let Some(DynamicName::Static(name)) = &bind.name else {
        return false;
    };
    let mut has_camel = false;
    let mut has_prop = false;
    let mut has_attr = false;
    for modifier in bind.modifiers.iter() {
        match *modifier {
            "camel" => has_camel = true,
            "prop" => has_prop = true,
            "attr" => has_attr = true,
            _ => return false,
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
        return false;
    }
    let Some(value) = &bind.value else {
        return false;
    };
    constant_for_hoist(value)
}

fn prefixed(prefix: char, name: &str) -> vize_s0::String {
    let mut out = vize_s0::String::with_capacity(1 + name.len());
    out.push(prefix);
    out.push_str(name);
    out
}

/// The retained-AST walk: any identifier reference, `this`, or TS-only
/// construct makes the expression non-constant (module docs).
struct ConstWalk {
    dynamic: bool,
}

impl<'a> Visit<'a> for ConstWalk {
    fn visit_identifier_reference(&mut self, _ident: &js::IdentifierReference<'a>) {
        self.dynamic = true;
    }

    fn visit_this_expression(&mut self, _this: &js::ThisExpression) {
        self.dynamic = true;
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
