//! Provider-shaped component regions, independent of export ordering. In
//! particular, a registered sibling is not hidden by the default export.
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclaration, Expression, ImportDeclarationSpecifier,
    NewExpression, ObjectExpression, Program, Statement,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_call_expression, walk_export_default_declaration, walk_new_expression,
        walk_object_expression,
    },
};
use oxc_semantic::Scoping;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashSet;

pub(super) fn regions(program: &Program<'_>, scoping: &Scoping, source: &str) -> Vec<Span> {
    let mut imports = FxHashSet::default();
    for statement in &program.body {
        if let Statement::ImportDeclaration(import) = statement
            && matches!(
                import.source.value.as_str(),
                "vue" | "@vue/composition-api" | "#imports"
            )
        {
            for specifier in import.specifiers.iter().flatten() {
                if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
                    && let Some(symbol) = specifier.local.symbol_id.get()
                {
                    imports.insert(symbol);
                }
            }
        }
    }
    let marked_lines = program
        .comments
        .iter()
        .filter(|comment| {
            source[comment.span.start as usize..comment.span.end as usize]
                .contains("@vue/component")
        })
        .map(|comment| {
            source[..comment.span.end as usize]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1
        })
        .collect();
    let mut collector = Components {
        scoping,
        imports,
        regions: Vec::new(),
        source,
        marked_lines,
    };
    collector.visit_program(program);
    collector.regions
}

struct Components<'a> {
    scoping: &'a Scoping,
    imports: FxHashSet<SymbolId>,
    regions: Vec<Span>,
    source: &'a str,
    marked_lines: Vec<usize>,
}
impl Components<'_> {
    fn object(&mut self, expression: &Expression<'_>) {
        if let Expression::ObjectExpression(object) = expression.get_inner_expression() {
            self.regions.push(object.span);
        }
    }
    fn factory(&self, expression: &Expression<'_>) -> bool {
        match expression {
            Expression::Identifier(id) => {
                if id.name == "defineNuxtComponent" {
                    return true;
                }
                if !matches!(
                    id.name.as_str(),
                    "defineComponent" | "createApp" | "component"
                ) {
                    return false;
                }
                let symbol = id
                    .reference_id
                    .get()
                    .and_then(|reference| self.scoping.get_reference(reference).symbol_id());
                symbol.is_none_or(|symbol| {
                    let flags = self.scoping.symbol_flags(symbol);
                    if flags.is_import() {
                        return self.imports.contains(&symbol);
                    }
                    !flags.is_function() && !flags.is_class() && !flags.is_catch_variable()
                })
            }
            _ => expression.as_member_expression().is_some_and(|member| {
                let Expression::Identifier(object) = member.object().get_inner_expression() else {
                    return false;
                };
                matches!(member.static_property_name(), Some("component" | "mixin"))
                    || (object.name == "Vue" && member.static_property_name() == Some("extend"))
            }),
        }
    }
}
impl<'a> Visit<'a> for Components<'_> {
    fn visit_export_default_declaration(&mut self, declaration: &ExportDefaultDeclaration<'a>) {
        if let Some(expression) = declaration.declaration.as_expression() {
            self.object(expression);
        }
        walk_export_default_declaration(self, declaration);
    }
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.factory(&call.callee)
            && let Some(expression) = call.arguments.last().and_then(Argument::as_expression)
        {
            self.object(expression);
        }
        walk_call_expression(self, call);
    }
    fn visit_new_expression(&mut self, expression: &NewExpression<'a>) {
        if matches!(&expression.callee, Expression::Identifier(id) if id.name == "Vue")
            && let Some(argument) = expression
                .arguments
                .first()
                .and_then(Argument::as_expression)
        {
            self.object(argument);
        }
        walk_new_expression(self, expression);
    }
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        if !self.marked_lines.is_empty() {
            let preceding_newlines = self.source[..object.span.start as usize]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count();
            if self.marked_lines.contains(&preceding_newlines) {
                self.regions.push(object.span);
            }
        }
        walk_object_expression(self, object);
    }
}
