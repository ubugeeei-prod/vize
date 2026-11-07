use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, Expression, NewExpression, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, Statement,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_export_default_declaration, walk_new_expression},
};
use oxc_parser::ParserReturn;
use oxc_span::Span;
use vize_l0::{FxHashMap, String, ToCompactString};

#[derive(Default)]
pub(super) struct Scan {
    pub(super) has_vue: bool,
    pub(super) has_name: bool,
    pub(super) has_non_vue_program: bool,
    pub(super) findings: Vec<NameFinding>,
}

pub(super) struct NameFinding {
    pub(super) name: String,
    pub(super) start: u32,
    pub(super) end: u32,
}

pub(super) fn scan_normal_script(parsed: &ParserReturn<'_>, offset: u32, scan: &mut Scan) {
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    scan.has_non_vue_program |= !parsed.program.body.is_empty();
    let bindings = object_bindings(&parsed.program);
    Scanner {
        setup: false,
        offset,
        bindings,
        scan,
    }
    .visit_program(&parsed.program);
}

pub(super) fn scan_setup_script(parsed: &ParserReturn<'_>, offset: u32, scan: &mut Scan) {
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    scan.has_vue = true;
    let bindings = object_bindings(&parsed.program);
    Scanner {
        setup: true,
        offset,
        bindings,
        scan,
    }
    .visit_program(&parsed.program);
}

struct Scanner<'ast, 'scan> {
    setup: bool,
    offset: u32,
    bindings: FxHashMap<&'ast str, &'ast ObjectExpression<'ast>>,
    scan: &'scan mut Scan,
}

impl Scanner<'_, '_> {
    fn note_component(&mut self, object: &ObjectExpression<'_>) {
        self.scan.has_vue = true;
        self.note_name_property(object);
    }

    fn note_name_property(&mut self, object: &ObjectExpression<'_>) {
        let Some(property) = object.properties.iter().find_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            is_name_key(property.computed, &property.key).then_some(property)
        }) else {
            return;
        };
        self.scan.has_name = true;
        if let Some((name, span)) = static_name(&property.value) {
            self.push(name, span);
        }
    }

    fn push(&mut self, name: String, span: Span) {
        self.scan.findings.push(NameFinding {
            name,
            start: self.offset + span.start,
            end: self.offset + span.end,
        });
    }

    fn note_registration(&mut self, call: &CallExpression<'_>) -> bool {
        let Some(member) = call.callee.get_inner_expression().as_member_expression() else {
            return false;
        };
        if member.static_property_name() != Some("component") || call.arguments.len() != 2 {
            return false;
        }
        self.scan.has_vue = true;
        self.scan.has_name = true;
        if let Some((name, span)) = call.arguments.first().and_then(static_argument_name) {
            self.push(name, span);
        }
        if let Some(object) = call.arguments.last().and_then(object_argument) {
            self.note_component(object);
        }
        true
    }
}

impl<'a> Visit<'a> for Scanner<'a, '_> {
    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => self.note_component(object),
            ExportDefaultDeclarationKind::Identifier(identifier) => {
                if let Some(object) = self.bindings.get(identifier.name.as_str()).copied() {
                    self.note_component(object);
                }
            }
            _ => {}
        }
        walk_export_default_declaration(self, export);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.note_registration(call) {
            walk_call_expression(self, call);
            return;
        }
        if let Expression::Identifier(identifier) = call.callee.get_inner_expression()
            && matches!(
                identifier.name.as_str(),
                "defineComponent" | "defineNuxtComponent" | "createApp"
            )
            && let Some(object) = call.arguments.last().and_then(object_argument)
        {
            self.note_component(object);
        } else if self.setup
            && let Expression::Identifier(identifier) = call.callee.get_inner_expression()
            && identifier.name == "defineOptions"
            && let Some(object) = call.arguments.first().and_then(object_argument)
        {
            self.note_name_property(object);
        } else if let Some(member) = call.callee.get_inner_expression().as_member_expression()
            && matches!(member.static_property_name(), Some("extend" | "mixin"))
            && let Some(object) = call.arguments.last().and_then(object_argument)
        {
            self.note_component(object);
        }
        walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, expression: &NewExpression<'a>) {
        if identifier_name(&expression.callee) == Some("Vue")
            && let Some(object) = expression.arguments.first().and_then(object_argument)
        {
            self.note_component(object);
        }
        walk_new_expression(self, expression);
    }
}

fn object_bindings<'a>(program: &'a Program<'a>) -> FxHashMap<&'a str, &'a ObjectExpression<'a>> {
    let mut bindings = FxHashMap::default();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let (BindingPattern::BindingIdentifier(identifier), Some(init)) =
                (&declarator.id, declarator.init.as_ref())
            else {
                continue;
            };
            if let Expression::ObjectExpression(object) = init.get_inner_expression() {
                bindings.insert(identifier.name.as_str(), object.as_ref());
            }
        }
    }
    bindings
}

fn object_argument<'a>(argument: &'a Argument<'a>) -> Option<&'a ObjectExpression<'a>> {
    match argument.as_expression()?.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn static_argument_name(argument: &Argument<'_>) -> Option<(String, Span)> {
    static_name(argument.as_expression()?)
}

fn static_name(expression: &Expression<'_>) -> Option<(String, Span)> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => {
            Some((literal.value.as_str().to_compact_string(), literal.span))
        }
        Expression::TemplateLiteral(template) => Some((
            template.single_quasi()?.as_str().to_compact_string(),
            template.span,
        )),
        Expression::NumericLiteral(literal) => {
            Some((literal.value.to_string().into(), literal.span))
        }
        Expression::BooleanLiteral(literal) => Some((
            if literal.value { "true" } else { "false" }.into(),
            literal.span,
        )),
        Expression::NullLiteral(literal) => Some(("null".into(), literal.span)),
        _ => None,
    }
}

fn identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn is_name_key(computed: bool, key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(identifier) if !computed => identifier.name == "name",
        PropertyKey::StringLiteral(literal) => literal.value == "name",
        PropertyKey::TemplateLiteral(template) => template
            .single_quasi()
            .is_some_and(|value| value.as_str() == "name"),
        _ => false,
    }
}
