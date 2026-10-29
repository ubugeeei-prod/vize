//! Vue component and instance objects, recognized as eslint-plugin-vue's
//! `getVueObjectType` does.
//!
//! An object literal is a Vue object when it is
//!
//! * `export` — the bare `export default { ... }` of an SFC `<script>`;
//! * `definition` — the last argument of `Vue.component/mixin/extend(...)`,
//!   `app.component/mixin(...)`, `defineNuxtComponent(...)`, or a
//!   `component/createApp/defineComponent(...)` that is not imported from
//!   another module;
//! * `instance` — the first argument of `new Vue(...)`;
//! * `mark` — on the line after a `@vue/component` comment.
//!
//! `as` casts around the object are skipped, as upstream does; other wrappers
//! are not.

use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, CallExpression, ExportDefaultDeclaration,
        ExportDefaultDeclarationKind, Expression, IdentifierReference, NewExpression,
        ObjectExpression, Program,
    },
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_export_default_declaration, walk_new_expression},
};
use oxc_semantic::Semantic;
use oxc_span::Span;
use vize_l0::FxHashMap;

/// How an object literal was recognized as a Vue object.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum VueObjectKind {
    Export,
    Definition,
    Instance,
    Mark,
}

/// Every Vue object of one script block, keyed by its span.
pub(super) struct VueObjects {
    kinds: FxHashMap<Span, VueObjectKind>,
}

impl VueObjects {
    /// `export_default` is false for a `<script setup>` block or a standalone
    /// script, where upstream never treats `export default {}` as a component.
    pub(super) fn collect<'a>(
        program: &Program<'a>,
        source: &str,
        semantic: &Semantic<'a>,
        export_default: bool,
    ) -> Self {
        let mut collector = Collector {
            semantic,
            export_default,
            kinds: FxHashMap::default(),
        };
        collector.visit_program(program);
        let mut kinds = collector.kinds;
        mark_objects(program, source, &mut kinds);
        Self { kinds }
    }

    pub(super) fn kind(&self, object: &ObjectExpression<'_>) -> Option<VueObjectKind> {
        self.kinds.get(&object.span).copied()
    }
}

struct Collector<'s, 'a> {
    semantic: &'s Semantic<'a>,
    export_default: bool,
    kinds: FxHashMap<Span, VueObjectKind>,
}

impl<'a> Visit<'a> for Collector<'_, 'a> {
    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        if self.export_default {
            let object = match &it.declaration {
                ExportDefaultDeclarationKind::ObjectExpression(object) => Some(&**object),
                ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
                    match paren.expression.without_parentheses() {
                        Expression::ObjectExpression(object) => Some(&**object),
                        _ => None,
                    }
                }
                _ => None,
            };
            if let Some(object) = object {
                self.kinds.insert(object.span, VueObjectKind::Export);
            }
        }
        walk_export_default_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(object) = it.arguments.last().and_then(object_argument)
            && self.definition_callee(&it.callee)
        {
            self.kinds
                .entry(object.span)
                .or_insert(VueObjectKind::Definition);
        }
        walk_call_expression(self, it);
    }

    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        if matches!(it.callee.without_parentheses(), Expression::Identifier(id) if id.name == "Vue")
            && let Some(object) = it.arguments.first().and_then(object_argument)
        {
            self.kinds
                .entry(object.span)
                .or_insert(VueObjectKind::Instance);
        }
        walk_new_expression(self, it);
    }
}

impl<'a> Collector<'_, 'a> {
    /// Upstream's `getVueComponentDefinitionType`.
    fn definition_callee(&self, callee: &Expression<'a>) -> bool {
        match callee.without_parentheses() {
            Expression::Identifier(identifier) => match identifier.name.as_str() {
                "defineNuxtComponent" => true,
                "component" | "createApp" | "defineComponent" => self.vue_global(identifier),
                _ => false,
            },
            callee => {
                let Some(member) = callee.as_member_expression() else {
                    return false;
                };
                let Expression::Identifier(object) = skip_ts_as(member.object()) else {
                    return false;
                };
                match member.static_property_name() {
                    Some("component" | "mixin") => true,
                    Some("extend") => object.name == "Vue",
                    _ => false,
                }
            }
        }
    }

    /// Upstream's `isGlobalOrImportedFromVue`: anything but a binding proven to
    /// come from elsewhere counts.
    fn vue_global(&self, identifier: &IdentifierReference<'a>) -> bool {
        let scoping = self.semantic.scoping();
        let Some(symbol) = identifier
            .reference_id
            .get()
            .and_then(|reference| scoping.get_reference(reference).symbol_id())
        else {
            return true;
        };
        if !scoping.symbol_redeclarations(symbol).is_empty() {
            return true;
        }
        let declaration = self.semantic.symbol_declaration(symbol);
        let nodes = self.semantic.nodes();
        match declaration.kind() {
            AstKind::ImportSpecifier(specifier) if specifier.imported.is_identifier() => {
                matches!(
                    nodes.parent_kind(declaration.id()),
                    AstKind::ImportDeclaration(import) if vue_module(&import.source.value)
                )
            }
            AstKind::VariableDeclarator(declarator)
                if matches!(declarator.id, BindingPattern::ObjectPattern(_)) =>
            {
                match declarator
                    .init
                    .as_ref()
                    .map(Expression::without_parentheses)
                {
                    Some(Expression::CallExpression(call)) if matches!(&call.callee, Expression::Identifier(id) if id.name == "require") =>
                    {
                        matches!(
                            call.arguments.first(),
                            Some(Argument::StringLiteral(source)) if vue_module(&source.value)
                        )
                    }
                    _ => true,
                }
            }
            AstKind::Function(_) | AstKind::Class(_) | AstKind::CatchParameter(_) => false,
            _ => true,
        }
    }
}

fn vue_module(source: &str) -> bool {
    matches!(source, "vue" | "@vue/composition-api")
}

fn object_argument<'b, 'a>(argument: &'b Argument<'a>) -> Option<&'b ObjectExpression<'a>> {
    match skip_ts_as(argument.as_expression()?) {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

/// Upstream's `skipTSAsExpression`; parentheses do not exist in ESTree.
pub(super) fn skip_ts_as<'b, 'a>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression.without_parentheses() {
        Expression::TSAsExpression(cast) => skip_ts_as(&cast.expression),
        expression => expression,
    }
}

/// Objects starting on the line after a `@vue/component` comment.
fn mark_objects(program: &Program<'_>, source: &str, kinds: &mut FxHashMap<Span, VueObjectKind>) {
    let marked_lines: Vec<usize> = program
        .comments
        .iter()
        .filter(|comment| {
            comment
                .content_span()
                .source_text(source)
                .contains("@vue/component")
        })
        .map(|comment| line_of(source, comment.span.end))
        .collect();
    if marked_lines.is_empty() {
        return;
    }
    let mut objects = ObjectSpans(Vec::new());
    objects.visit_program(program);
    for span in objects.0 {
        let line = line_of(source, span.start);
        if line > 0 && marked_lines.contains(&(line - 1)) {
            kinds.entry(span).or_insert(VueObjectKind::Mark);
        }
    }
}

fn line_of(source: &str, offset: u32) -> usize {
    let end = (offset as usize).min(source.len());
    source.as_bytes().get(..end).map_or(0, |bytes| {
        bytes.iter().filter(|byte| **byte == b'\n').count()
    })
}

struct ObjectSpans(Vec<Span>);

impl<'a> Visit<'a> for ObjectSpans {
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        self.0.push(it.span);
        oxc_ast_visit::walk::walk_object_expression(self, it);
    }
}
