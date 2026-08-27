//! Constant-expression classification for patch flags and hoisting.
//!
//! Split from `helpers.rs`; the P1-7 retained-AST gate lives here. The
//! legacy path parses the wrapped `(content)`; the visitor is span-free
//! (names and binding metadata only), so walking the retained AST is
//! decision-equal whenever it still describes the node's bytes and the
//! dialect gate holds.

use crate::SimpleExpressionNode;
use crate::options::{BindingMetadata, BindingType};
use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_function},
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_croquis::builtins::is_global_allowed;
use vize_s0::{FxHashSet, String};

fn is_constant_binding(binding_type: BindingType) -> bool {
    matches!(
        binding_type,
        BindingType::SetupConst
            | BindingType::LiteralConst
            | BindingType::ExternalModule
            | BindingType::JsGlobalUniversal
            | BindingType::JsGlobalBrowser
            | BindingType::JsGlobalNode
            | BindingType::JsGlobalDeno
            | BindingType::JsGlobalBun
    )
}

fn is_runtime_helper_ident(name: &str) -> bool {
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

#[derive(Default)]
struct RuntimeDependencyVisitor<'a> {
    bindings: Option<&'a BindingMetadata>,
    scopes: Vec<FxHashSet<vize_s0::String>>,
    has_dynamic_dependency: bool,
}

impl<'a> RuntimeDependencyVisitor<'a> {
    fn new(bindings: Option<&'a BindingMetadata>) -> Self {
        Self {
            bindings,
            scopes: vec![FxHashSet::default()],
            has_dynamic_dependency: false,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(FxHashSet::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn is_local(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }

    fn add_binding_pattern(&mut self, pattern: &oxc_ast_types::BindingPattern<'_>) {
        match pattern {
            oxc_ast_types::BindingPattern::BindingIdentifier(ident) => {
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(vize_s0::String::new(ident.name.as_str()));
                }
            }
            oxc_ast_types::BindingPattern::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    self.add_binding_pattern(&prop.value);
                }
                if let Some(rest) = &obj.rest {
                    self.add_binding_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::ArrayPattern(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.add_binding_pattern(elem);
                }
                if let Some(rest) = &arr.rest {
                    self.add_binding_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::AssignmentPattern(assign) => {
                self.add_binding_pattern(&assign.left);
            }
        }
    }
}

impl<'a> Visit<'_> for RuntimeDependencyVisitor<'a> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast_types::IdentifierReference<'_>) {
        if self.has_dynamic_dependency {
            return;
        }

        let name = ident.name.as_str();
        if self.is_local(name) || is_global_allowed(name) || is_runtime_helper_ident(name) {
            return;
        }

        if matches!(name, "_ctx" | "$setup" | "__props" | "$props") {
            self.has_dynamic_dependency = true;
            return;
        }

        if let Some(bindings) = self.bindings {
            match bindings.bindings.get(name).copied() {
                Some(binding_type) if is_constant_binding(binding_type) => {}
                Some(_) | None => {
                    self.has_dynamic_dependency = true;
                }
            }
        } else {
            self.has_dynamic_dependency = true;
        }
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast_types::ArrowFunctionExpression<'_>,
    ) {
        self.push_scope();
        for param in &arrow.params.items {
            self.add_binding_pattern(&param.pattern);
        }
        walk_arrow_function_expression(self, arrow);
        self.pop_scope();
    }

    fn visit_function(&mut self, func: &oxc_ast_types::Function<'_>, flags: ScopeFlags) {
        self.push_scope();
        for param in &func.params.items {
            self.add_binding_pattern(&param.pattern);
        }
        walk_function(self, func, flags);
        self.pop_scope();
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast_types::VariableDeclarator<'_>) {
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.add_binding_pattern(&declarator.id);
    }
}

/// Returns true when a non-static simple expression is still a compile-time constant.
///
/// This is used by patch-flag generation and style normalization decisions.
/// If parsing fails, this conservatively returns `false` so dynamic updates are preserved.
pub fn is_constant_simple_expression(
    exp: &SimpleExpressionNode<'_>,
    bindings: Option<&BindingMetadata>,
) -> bool {
    if exp.is_static {
        return true;
    }

    // Expressions that already reference runtime context/setup/props are dynamic.
    let content = exp.content;
    if !crate::steps::expression::expression_is_safe_to_parse(content) {
        return false;
    }
    if content.contains("_ctx.")
        || content.contains("$setup.")
        || content.contains("__props.")
        || content.contains("$props.")
    {
        return false;
    }

    // Retained fast path (P1-7): the parse-once AST still describes these
    // exact bytes and the dialect gate holds — walk it instead of re-parsing.
    if let Some(js) = crate::retained::retained_whole_expression(exp)
        && crate::retained::js_module_compatible(js)
    {
        let mut visitor = RuntimeDependencyVisitor::new(bindings);
        visitor.visit_expression(js.ast);
        let result = !visitor.has_dynamic_dependency;
        #[cfg(any(test, feature = "davinci-differential"))]
        {
            // Dual-run in an uncounted arena; divergence panics.
            let legacy =
                is_constant_parsed(content, bindings, &oxc_allocator::Allocator::default());
            assert_eq!(
                Some(result),
                legacy,
                "davinci-differential (P1-7): retained constant check diverged from the legacy re-parse for expression {content:?}"
            );
            crate::retained::differential::record_shape_comparison();
        }
        return result;
    }

    let allocator = crate::expr_parse_probe::parse_arena();
    is_constant_parsed(content, bindings, &allocator).unwrap_or(false)
}

/// The legacy wrapped parse + visitor walk; `None` when the parse fails.
fn is_constant_parsed(
    content: &str,
    bindings: Option<&BindingMetadata>,
    allocator: &oxc_allocator::Allocator,
) -> Option<bool> {
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');

    let parser = Parser::new(allocator, &wrapped, SourceType::default().with_module(true));
    let expr = parser.parse_expression().ok()?;

    let mut visitor = RuntimeDependencyVisitor::new(bindings);
    visitor.visit_expression(&expr);
    Some(!visitor.has_dynamic_dependency)
}
