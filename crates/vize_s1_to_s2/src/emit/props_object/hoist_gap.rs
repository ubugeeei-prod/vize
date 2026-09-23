//! The props-hoist classifier gap, refused rather than guessed (P3-17).
//!
//! The shipped `has_static_props` hoists a props surface when every
//! attribute but `ref` is static and every directive is a static-name
//! `v-bind` whose value `is_constant_simple_expression` admits — an
//! expression whose identifiers are all locals it binds itself, allowlisted
//! globals, or helper aliases. The S2 hoist pass admits the self-bound
//! locals and still refuses a free name, so `:format="(v) => v.toFixed(2)"`
//! hoists on both lanes while `:format="(v) => Math.max(v, 1)"` would
//! diverge. The production-path parity oracle measured that disagreement
//! as byte divergence.
//!
//! A root-position surface of an inlined render function on which the two
//! classifiers still disagree refuses the S2 emit, so the compile stays on
//! the lane that produced the bytes.
//! That is the shipped `is_root && inline` props-hoist arm the production
//! SFC shape reaches; the non-inline lanes keep their corpus-proven output
//! (the remaining non-inline root case is recorded in the P3-17 record). The gate is a superset of the divergent
//! cases on unerased expressions (the shipped hoist also depends on
//! position); `this` counts as constant, as it does for the shipped visitor.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast as js;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_function};
use oxc_syntax::scope::ScopeFlags;
use vize_s0::String;
use vize_s2::expr::ExprRef;
use vize_s2::op::{Attribute, BindingOp, DynamicName};

use super::super::EmitError;
use super::super::error::UnsupportedReason as Reason;
use super::super::prefix::globals::is_global_allowed;
use crate::pass::hoist::constant_for_hoist;

/// Refuse a props surface the shipped classifier hoists and S2's does not.
pub(in crate::emit) fn check(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<(), EmitError> {
    if bindings.is_empty() || attributes.iter().any(|attribute| attribute.name == "ref") {
        return Ok(());
    }
    let mut gap = None;
    for binding in bindings.iter() {
        let BindingOp::Bind(bind) = binding else {
            return Ok(());
        };
        let Some(DynamicName::Static(name)) = &bind.name else {
            return Ok(());
        };
        if matches!(*name, "ref" | "class")
            || bind
                .modifiers
                .iter()
                .any(|modifier| !matches!(*modifier, "camel" | "prop" | "attr"))
        {
            return Ok(());
        }
        let Some(value) = &bind.value else {
            return Ok(());
        };
        let ExprRef::Js(js) = value else {
            return Ok(());
        };
        let Some(reads_local) = shipped_constant(js.ast, js.source) else {
            return Ok(());
        };
        // Globals-only values are already spelled the shipped way by the
        // emitter's own legacy-global rules; the unmirrored class is a value
        // reading a local it binds itself (an arrow or function parameter).
        if gap.is_none() && reads_local && !constant_for_hoist(value) {
            gap = Some(bind.span);
        }
    }
    match gap {
        Some(span) => Err(EmitError::unsupported_at(Reason::HoistConstantGap, span)),
        None => Ok(()),
    }
}

/// `is_constant_simple_expression(exp, None)` over the supplied AST and
/// expression bytes: `Some(reads_local)` when the shipped classifier admits it.
pub(in crate::emit) fn shipped_constant(expr: &js::Expression<'_>, source: &str) -> Option<bool> {
    if source.contains("_ctx.")
        || source.contains("$setup.")
        || source.contains("__props.")
        || source.contains("$props.")
    {
        return None;
    }
    let mut walk = ShippedConstWalk {
        locals: StdVec::new(),
        dynamic: false,
        reads_local: false,
    };
    walk.visit_expression(expr);
    (!walk.dynamic).then_some(walk.reads_local)
}

struct ShippedConstWalk {
    locals: StdVec<StdVec<String>>,
    dynamic: bool,
    reads_local: bool,
}

impl ShippedConstWalk {
    fn bind_pattern(&mut self, pattern: &js::BindingPattern<'_>) {
        match pattern {
            js::BindingPattern::BindingIdentifier(ident) => {
                if let Some(frame) = self.locals.last_mut() {
                    frame.push(String::from(ident.name.as_str()));
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

impl<'a> Visit<'a> for ShippedConstWalk {
    fn visit_identifier_reference(&mut self, ident: &js::IdentifierReference<'a>) {
        let name = ident.name.as_str();
        let local = self
            .locals
            .iter()
            .any(|frame| frame.iter().any(|bound| bound.as_str() == name));
        self.reads_local |= local;
        if !(local || is_global_allowed(name) || is_helper(name)) {
            self.dynamic = true;
        }
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
    }

    // TypeScript syntax: the shipped classifier re-parses under an mjs
    // source type and refuses it unless erasure already ran, which this
    // gate cannot see, so it answers "not constant" — the refusal stays a
    // superset of the unerased lane and the erased-TS remainder is recorded
    // in the P3-17 record.
    fn visit_ts_type_annotation(&mut self, _annotation: &js::TSTypeAnnotation<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_type_parameter_declaration(
        &mut self,
        _declaration: &js::TSTypeParameterDeclaration<'a>,
    ) {
        self.dynamic = true;
    }

    fn visit_ts_type_parameter_instantiation(
        &mut self,
        _instantiation: &js::TSTypeParameterInstantiation<'a>,
    ) {
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

fn is_helper(name: &str) -> bool {
    matches!(
        name,
        "_unref"
            | "_normalizeClass"
            | "_normalizeStyle"
            | "_toDisplayString"
            | "_toHandlerKey"
            | "_mergeProps"
            | "_toHandlers"
            | "_guardReactiveProps"
            | "_normalizeProps"
    )
}
