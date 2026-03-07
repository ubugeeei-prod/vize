//! Utility functions for code generation.

use crate::{
    ast::{RuntimeHelper, SimpleExpressionNode},
    options::{BindingMetadata, BindingType},
};
use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{
    walk::{walk_arrow_function_expression, walk_function},
    Visit,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::FxHashSet;
use vize_croquis::builtins::is_global_allowed;

/// Decode HTML entities (numeric character references) in a string
/// Supports &#xHHHH; (hex) and &#NNNN; (decimal) formats
pub fn decode_html_entities(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' && chars.peek() == Some(&'#') {
            chars.next(); // consume '#'
            let is_hex = chars.peek() == Some(&'x') || chars.peek() == Some(&'X');
            if is_hex {
                chars.next(); // consume 'x' or 'X'
            }

            let mut num_str = String::default();
            while let Some(&ch) = chars.peek() {
                if ch == ';' {
                    chars.next(); // consume ';'
                    break;
                }
                let is_valid_char =
                    (is_hex && ch.is_ascii_hexdigit()) || (!is_hex && ch.is_ascii_digit());
                if is_valid_char {
                    num_str.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }

            if !num_str.is_empty() {
                let codepoint = if is_hex {
                    u32::from_str_radix(&num_str, 16).ok()
                } else {
                    num_str.parse::<u32>().ok()
                };

                if let Some(cp) = codepoint {
                    if let Some(decoded_char) = char::from_u32(cp) {
                        result.push(decoded_char);
                        continue;
                    }
                }
            }

            // If decoding failed, output the original sequence
            result.push('&');
            result.push('#');
            if is_hex {
                result.push('x');
            }
            result.push_str(&num_str);
        } else {
            result.push(c);
        }
    }

    result
}

/// Escape a string for use in JavaScript string literals
pub fn escape_js_string(s: &str) -> String {
    // First decode HTML entities, then escape for JS
    let decoded = decode_html_entities(s);
    let mut result = String::with_capacity(decoded.len());
    fn push_hex4(out: &mut String, value: u32) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        out.push_str("\\u");
        out.push(HEX[((value >> 12) & 0xF) as usize] as char);
        out.push(HEX[((value >> 8) & 0xF) as usize] as char);
        out.push(HEX[((value >> 4) & 0xF) as usize] as char);
        out.push(HEX[(value & 0xF) as usize] as char);
    }
    for c in decoded.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"), // backspace
            '\x0C' => result.push_str("\\f"), // form feed
            c if c.is_control() => {
                // Other control characters as unicode escape
                push_hex4(&mut result, c as u32);
            }
            c => result.push(c),
        }
    }
    result
}

/// Check if a string is a valid JavaScript identifier (doesn't need quoting)
pub fn is_valid_js_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    // First character must be a letter, underscore, or dollar sign
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    // Remaining characters can also include digits
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Default helper alias function
pub fn default_helper_alias(helper: RuntimeHelper) -> &'static str {
    match helper {
        // Core helpers
        RuntimeHelper::Fragment => "_Fragment",
        RuntimeHelper::Teleport => "_Teleport",
        RuntimeHelper::Suspense => "_Suspense",
        RuntimeHelper::KeepAlive => "_KeepAlive",
        RuntimeHelper::BaseTransition => "_BaseTransition",
        RuntimeHelper::Transition => "_Transition",
        RuntimeHelper::TransitionGroup => "_TransitionGroup",
        RuntimeHelper::OpenBlock => "_openBlock",
        RuntimeHelper::CreateBlock => "_createBlock",
        RuntimeHelper::CreateElementBlock => "_createElementBlock",
        RuntimeHelper::CreateVNode => "_createVNode",
        RuntimeHelper::CreateElementVNode => "_createElementVNode",
        RuntimeHelper::CreateComment => "_createCommentVNode",
        RuntimeHelper::CreateText => "_createTextVNode",
        RuntimeHelper::CreateStatic => "_createStaticVNode",
        RuntimeHelper::ResolveComponent => "_resolveComponent",
        RuntimeHelper::ResolveDynamicComponent => "_resolveDynamicComponent",
        RuntimeHelper::ResolveDirective => "_resolveDirective",
        RuntimeHelper::ResolveFilter => "_resolveFilter",
        RuntimeHelper::WithDirectives => "_withDirectives",
        RuntimeHelper::VShow => "_vShow",
        RuntimeHelper::VModelText => "_vModelText",
        RuntimeHelper::VModelCheckbox => "_vModelCheckbox",
        RuntimeHelper::VModelRadio => "_vModelRadio",
        RuntimeHelper::VModelSelect => "_vModelSelect",
        RuntimeHelper::VModelDynamic => "_vModelDynamic",
        RuntimeHelper::RenderList => "_renderList",
        RuntimeHelper::RenderSlot => "_renderSlot",
        RuntimeHelper::CreateSlots => "_createSlots",
        RuntimeHelper::ToDisplayString => "_toDisplayString",
        RuntimeHelper::MergeProps => "_mergeProps",
        RuntimeHelper::NormalizeClass => "_normalizeClass",
        RuntimeHelper::NormalizeStyle => "_normalizeStyle",
        RuntimeHelper::NormalizeProps => "_normalizeProps",
        RuntimeHelper::GuardReactiveProps => "_guardReactiveProps",
        RuntimeHelper::ToHandlers => "_toHandlers",
        RuntimeHelper::Camelize => "_camelize",
        RuntimeHelper::Capitalize => "_capitalize",
        RuntimeHelper::ToHandlerKey => "_toHandlerKey",
        RuntimeHelper::SetBlockTracking => "_setBlockTracking",
        RuntimeHelper::PushScopeId => "_pushScopeId",
        RuntimeHelper::PopScopeId => "_popScopeId",
        RuntimeHelper::WithCtx => "_withCtx",
        RuntimeHelper::Unref => "_unref",
        RuntimeHelper::IsRef => "_isRef",
        RuntimeHelper::WithMemo => "_withMemo",
        RuntimeHelper::IsMemoSame => "_isMemoSame",
        RuntimeHelper::WithModifiers => "_withModifiers",
        RuntimeHelper::WithKeys => "_withKeys",

        // SSR helpers
        RuntimeHelper::SsrInterpolate => "_ssrInterpolate",
        RuntimeHelper::SsrRenderVNode => "_ssrRenderVNode",
        RuntimeHelper::SsrRenderComponent => "_ssrRenderComponent",
        RuntimeHelper::SsrRenderSlot => "_ssrRenderSlot",
        RuntimeHelper::SsrRenderSlotInner => "_ssrRenderSlotInner",
        RuntimeHelper::SsrRenderAttrs => "_ssrRenderAttrs",
        RuntimeHelper::SsrRenderAttr => "_ssrRenderAttr",
        RuntimeHelper::SsrRenderDynamicAttr => "_ssrRenderDynamicAttr",
        RuntimeHelper::SsrIncludeBooleanAttr => "_ssrIncludeBooleanAttr",
        RuntimeHelper::SsrRenderClass => "_ssrRenderClass",
        RuntimeHelper::SsrRenderStyle => "_ssrRenderStyle",
        RuntimeHelper::SsrRenderDynamicModel => "_ssrRenderDynamicModel",
        RuntimeHelper::SsrGetDynamicModelProps => "_ssrGetDynamicModelProps",
        RuntimeHelper::SsrRenderList => "_ssrRenderList",
        RuntimeHelper::SsrLooseEqual => "_ssrLooseEqual",
        RuntimeHelper::SsrLooseContain => "_ssrLooseContain",
        RuntimeHelper::SsrGetDirectiveProps => "_ssrGetDirectiveProps",
        RuntimeHelper::SsrRenderTeleport => "_ssrRenderTeleport",
        RuntimeHelper::SsrRenderSuspense => "_ssrRenderSuspense",
    }
}

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
    scopes: Vec<FxHashSet<vize_carton::String>>,
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
                    scope.insert(vize_carton::String::new(ident.name.as_str()));
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

    let mut wrapped = String::with_capacity(exp.content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(exp.content.as_str());
    wrapped.push(')');

    let allocator = oxc_allocator::Allocator::default();
    let parser = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_module(true),
    );
    let Ok(expr) = parser.parse_expression() else {
        return false;
    };

    let mut visitor = RuntimeDependencyVisitor::new(bindings);
    visitor.visit_expression(&expr);
    !visitor.has_dynamic_dependency
}

// Re-export from vize_carton for convenience
pub use vize_carton::{camelize, capitalize, String};

/// Capitalize first letter of a string (alias for capitalize)
#[inline]
pub fn capitalize_first(s: &str) -> String {
    capitalize(s)
}

/// Check if a component is a Vue built-in that should be imported directly.
/// Handles both PascalCase and kebab-case tag names.
pub fn is_builtin_component(name: &str) -> Option<RuntimeHelper> {
    match name {
        "Teleport" | "teleport" => Some(RuntimeHelper::Teleport),
        "Suspense" | "suspense" => Some(RuntimeHelper::Suspense),
        "KeepAlive" | "keep-alive" => Some(RuntimeHelper::KeepAlive),
        "BaseTransition" | "base-transition" => Some(RuntimeHelper::BaseTransition),
        "Transition" | "transition" => Some(RuntimeHelper::Transition),
        "TransitionGroup" | "transition-group" => Some(RuntimeHelper::TransitionGroup),
        _ => None,
    }
}
