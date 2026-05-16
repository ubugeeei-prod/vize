//! Template type checking runner.
//!
//! Contains the `TypeChecker` struct and its core template validation
//! methods: interpolation checking, directive checking, event handler
//! checking, and v-bind expression checking.

use crate::{
    context::TypeContext,
    diagnostic::{TypeDiagnostic, TypeErrorCode},
    CheckResult,
};
use vize_carton::cstr;

/// Type checker for Vue SFC templates.
///
/// The TypeChecker validates template expressions against the type context
/// derived from the script block.
#[derive(Debug, Default)]
pub struct TypeChecker {
    /// Enable strict mode (no implicit any).
    pub strict: bool,
    /// Enable Vue-specific checks.
    pub vue_checks: bool,
}

impl TypeChecker {
    /// Create a new type checker with default settings.
    pub fn new() -> Self {
        Self {
            strict: false,
            vue_checks: true,
        }
    }

    /// Create a strict type checker.
    pub fn strict() -> Self {
        Self {
            strict: true,
            vue_checks: true,
        }
    }

    /// Enable or disable strict mode.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Enable or disable Vue-specific checks.
    pub fn with_vue_checks(mut self, vue_checks: bool) -> Self {
        self.vue_checks = vue_checks;
        self
    }

    /// Check a template against a type context.
    ///
    /// # Arguments
    /// * `template` - The template content to check
    /// * `ctx` - The type context from the script block
    ///
    /// # Returns
    /// A CheckResult containing any type errors found
    pub fn check_template(&self, template: &str, ctx: &TypeContext) -> CheckResult {
        let mut result = CheckResult::new();

        // Find all expression interpolations {{ ... }}
        self.check_interpolations(template, ctx, &mut result);

        // Find all directive expressions (v-if, v-for, etc.)
        self.check_directives(template, ctx, &mut result);

        // Find all event handlers (@click, v-on:click)
        self.check_event_handlers(template, ctx, &mut result);

        // Find all v-bind expressions (:prop, v-bind:prop)
        self.check_bindings(template, ctx, &mut result);

        result
    }

    /// Check interpolation expressions {{ expr }}.
    fn check_interpolations(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        let mut pos = 0;
        while let Some(start) = template[pos..].find("{{") {
            let abs_start = pos + start;
            if let Some(end) = template[abs_start..].find("}}") {
                let expr_start = abs_start + 2;
                let expr_end = abs_start + end;
                let expr = template[expr_start..expr_end].trim();

                if !expr.is_empty() {
                    self.check_expression(expr, expr_start as u32, expr_end as u32, ctx, result);
                }

                pos = abs_start + end + 2;
            } else {
                break;
            }
        }
    }

    /// Check directive expressions.
    fn check_directives(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        // Check v-if, v-else-if, v-show expressions
        for directive in ["v-if", "v-else-if", "v-show"] {
            self.check_directive_values(template, directive, ctx, result);
        }

        // v-for has special syntax: "item in items" or "(item, index) in items"
        self.check_vfor_expressions(template, ctx, result);
    }

    /// Check values of a specific directive.
    fn check_directive_values(
        &self,
        template: &str,
        directive: &str,
        ctx: &TypeContext,
        result: &mut CheckResult,
    ) {
        let pattern = cstr!("{directive}=\"");
        let mut pos = 0;

        while let Some(start) = template[pos..].find(pattern.as_str()) {
            let abs_start = pos + start + pattern.len();
            if let Some(end) = template[abs_start..].find('"') {
                let expr = &template[abs_start..abs_start + end];
                if !expr.is_empty() {
                    self.check_expression(
                        expr,
                        abs_start as u32,
                        (abs_start + end) as u32,
                        ctx,
                        result,
                    );
                }
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }
    }

    /// Check v-for expressions.
    fn check_vfor_expressions(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        let pattern = "v-for=\"";
        let mut pos = 0;

        while let Some(start) = template[pos..].find(pattern) {
            let abs_start = pos + start + pattern.len();
            if let Some(end) = template[abs_start..].find('"') {
                let expr = &template[abs_start..abs_start + end];

                // Parse "item in items" or "(item, index) in items"
                if let Some(in_pos) = expr.find(" in ") {
                    let iterable = expr[in_pos + 4..].trim();
                    let iterable_start = abs_start + in_pos + 4;
                    self.check_expression(
                        iterable,
                        iterable_start as u32,
                        (abs_start + end) as u32,
                        ctx,
                        result,
                    );
                }

                pos = abs_start + end + 1;
            } else {
                break;
            }
        }
    }

    /// Check event handlers.
    fn check_event_handlers(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        // Check @event="handler" and v-on:event="handler"
        let patterns = ["@", "v-on:"];

        for pattern in patterns {
            let mut pos = 0;
            while let Some(start) = template[pos..].find(pattern) {
                let abs_start = pos + start + pattern.len();

                // Find the end of the event name and the ="
                if let Some(eq_pos) = template[abs_start..].find("=\"") {
                    let handler_start = abs_start + eq_pos + 2;
                    if let Some(end) = template[handler_start..].find('"') {
                        let handler = &template[handler_start..handler_start + end];

                        // Simple handler (just a function name)
                        if Self::is_simple_identifier(handler) {
                            self.check_identifier(
                                handler,
                                handler_start as u32,
                                (handler_start + end) as u32,
                                ctx,
                                result,
                            );
                        } else if !handler.is_empty() {
                            // Inline handler expression
                            self.check_expression(
                                handler,
                                handler_start as u32,
                                (handler_start + end) as u32,
                                ctx,
                                result,
                            );
                        }

                        pos = handler_start + end + 1;
                    } else {
                        break;
                    }
                } else {
                    pos = abs_start + 1;
                }
            }
        }
    }

    /// Check v-bind expressions.
    fn check_bindings(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        // Check :prop="expr" and v-bind:prop="expr"
        let patterns = [(":", "="), ("v-bind:", "=")];

        for (prefix, suffix) in patterns {
            let mut pos = 0;
            while let Some(start) = template[pos..].find(prefix) {
                // Skip :: (CSS pseudo-selectors)
                if prefix == ":" && template[pos + start..].starts_with("::") {
                    pos = pos + start + 2;
                    continue;
                }

                let abs_start = pos + start + prefix.len();

                // Find ="
                if let Some(eq_pos) = template[abs_start..].find(&*cstr!("{suffix}\"")) {
                    let expr_start = abs_start + eq_pos + 2;
                    if let Some(end) = template[expr_start..].find('"') {
                        let expr = &template[expr_start..expr_start + end];
                        if !expr.is_empty() {
                            self.check_expression(
                                expr,
                                expr_start as u32,
                                (expr_start + end) as u32,
                                ctx,
                                result,
                            );
                        }
                        pos = expr_start + end + 1;
                    } else {
                        break;
                    }
                } else {
                    pos = abs_start + 1;
                }
            }
        }
    }

    /// Check a single expression.
    pub(crate) fn check_expression(
        &self,
        expr: &str,
        start: u32,
        _end: u32,
        ctx: &TypeContext,
        result: &mut CheckResult,
    ) {
        // Extract identifiers from the expression and check each one
        for (ident, offset) in Self::extract_identifiers(expr) {
            let ident_start = start + offset as u32;
            let ident_end = ident_start + ident.len() as u32;
            self.check_identifier(ident, ident_start, ident_end, ctx, result);
        }
    }

    /// Check if an identifier exists in the context.
    pub(crate) fn check_identifier(
        &self,
        ident: &str,
        start: u32,
        end: u32,
        ctx: &TypeContext,
        result: &mut CheckResult,
    ) {
        // Skip keywords and literals
        if Self::is_keyword_or_literal(ident) {
            return;
        }

        // Skip $-prefixed globals ($event, $refs, etc.)
        if ident.starts_with('$') {
            return;
        }

        // Check if the identifier is defined
        if !ctx.has_binding(ident) && !ctx.globals.contains_key(ident) {
            result.add_diagnostic(TypeDiagnostic::error(
                TypeErrorCode::UnknownIdentifier,
                cstr!("Cannot find name '{ident}'"),
                start,
                end,
            ));
        }
    }
}
