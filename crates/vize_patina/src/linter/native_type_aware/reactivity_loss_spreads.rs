use super::{
    Argument, ArrowFunctionExpression, BindingPattern, CallExpression, ChainElement, Expression,
    Function, FunctionBody, FxHashSet, ImportDeclaration, ImportDeclarationSpecifier,
    ModuleExportName, ObjectExpression, ObjectPropertyKind, OxcAllocator, OxcParser, PropertyKey,
    PropertyKind, ScopeFlags, SourceType, SpreadElement, String, ToCompactString,
    VariableDeclarator, Visit, profile, walk_arrow_function_expression, walk_call_expression,
    walk_function, walk_import_declaration, walk_spread_element, walk_variable_declarator,
};

#[path = "reactivity_loss_diagnostic.rs"]
mod diagnostic;
pub(super) use diagnostic::{diagnostic_key, reactivity_loss_diagnostic};

pub(super) fn computed_value_spread_spans(script: &str) -> FxHashSet<(u32, u32)> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let parsed = profile!(
        "patina.type_aware.computed_spread.parse",
        OxcParser::new(&allocator, script, source_type).parse()
    );
    if parsed.panicked {
        return FxHashSet::default();
    }

    let mut collector = ComputedValueSpreadCollector::default();
    collector.visit_program(&parsed.program);
    collector.exempt
}

#[derive(Default)]
struct ComputedValueSpreadCollector {
    aliases: FxHashSet<String>,
    exempt: FxHashSet<(u32, u32)>,
    in_getter: bool,
    nested: u32,
}

impl ComputedValueSpreadCollector {
    fn is_computed_name(&self, name: &str) -> bool {
        name == "computed" || self.aliases.contains(name)
    }

    fn remember_alias(&mut self, name: &str) {
        self.aliases.insert(name.to_compact_string());
    }

    fn visit_computed_argument(&mut self, argument: &Argument<'_>) {
        if let Some(expression) = argument.as_expression() {
            self.visit_computed_expression(expression);
        } else {
            self.visit_argument(argument);
        }
    }

    fn visit_computed_expression(&mut self, expression: &Expression<'_>) {
        match expression {
            Expression::ArrowFunctionExpression(arrow) => self.visit_arrow_getter(arrow),
            Expression::FunctionExpression(function) => self.visit_function_getter(function),
            Expression::ObjectExpression(object) => self.visit_computed_options(object),
            Expression::ParenthesizedExpression(paren) => {
                self.visit_computed_expression(&paren.expression);
            }
            Expression::TSAsExpression(ts_as) => self.visit_computed_expression(&ts_as.expression),
            Expression::TSSatisfiesExpression(ts_satisfies) => {
                self.visit_computed_expression(&ts_satisfies.expression);
            }
            Expression::TSNonNullExpression(ts_non_null) => {
                self.visit_computed_expression(&ts_non_null.expression);
            }
            other => self.visit_expression(other),
        }
    }

    fn visit_arrow_getter(&mut self, arrow: &ArrowFunctionExpression<'_>) {
        if let Some(type_parameters) = &arrow.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        self.visit_formal_parameters(&arrow.params);
        if let Some(return_type) = &arrow.return_type {
            self.visit_ts_type_annotation(return_type);
        }
        self.visit_getter_body(&arrow.body);
    }

    fn visit_function_getter(&mut self, function: &Function<'_>) {
        self.visit_formal_parameters(&function.params);
        if let Some(body) = &function.body {
            self.visit_getter_body(body);
        }
    }

    fn visit_getter_body(&mut self, body: &FunctionBody<'_>) {
        let previous = self.in_getter;
        self.in_getter = true;
        for statement in &body.statements {
            self.visit_statement(statement);
        }
        self.in_getter = previous;
    }

    fn visit_computed_options(&mut self, object: &ObjectExpression<'_>) {
        for property in &object.properties {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let is_getter = property.kind == PropertyKind::Get
                        || property_name(&property.key) == Some("get");
                    if let Some(key) = property.key.as_expression() {
                        self.visit_expression(key);
                    }
                    if is_getter {
                        self.visit_computed_expression(&property.value);
                    } else {
                        self.visit_expression(&property.value);
                    }
                }
                ObjectPropertyKind::SpreadProperty(spread) => self.visit_spread_element(spread),
            }
        }
    }
}

impl<'a> Visit<'a> for ComputedValueSpreadCollector {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        if declaration.source.value.as_str() == "vue"
            && let Some(specifiers) = &declaration.specifiers
        {
            for specifier in specifiers {
                let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                    continue;
                };
                let imported = match &specifier.imported {
                    ModuleExportName::IdentifierName(name) => name.name.as_str(),
                    ModuleExportName::IdentifierReference(name) => name.name.as_str(),
                    ModuleExportName::StringLiteral(name) => name.value.as_str(),
                };
                if imported == "computed" {
                    self.remember_alias(specifier.local.name.as_str());
                }
            }
        }
        walk_import_declaration(self, declaration);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(binding) = &declarator.id
            && let Some(init) = &declarator.init
            && let Some(name) = identifier_name(init)
            && self.is_computed_name(name)
        {
            self.remember_alias(binding.name.as_str());
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if callee_is_computed(&call.callee, self) {
            self.visit_expression(&call.callee);
            for (index, argument) in call.arguments.iter().enumerate() {
                if index == 0 {
                    self.visit_computed_argument(argument);
                } else {
                    self.visit_argument(argument);
                }
            }
            return;
        }
        walk_call_expression(self, call);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.in_getter {
            self.nested += 1;
            walk_arrow_function_expression(self, arrow);
            self.nested -= 1;
            return;
        }
        walk_arrow_function_expression(self, arrow);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if self.in_getter {
            self.nested += 1;
            walk_function(self, function, flags);
            self.nested -= 1;
            return;
        }
        walk_function(self, function, flags);
    }

    fn visit_spread_element(&mut self, spread: &SpreadElement<'a>) {
        if self.in_getter && self.nested == 0 && is_ref_value_expression(&spread.argument) {
            self.exempt.insert((spread.span.start, spread.span.end));
        }
        walk_spread_element(self, spread);
    }
}

fn callee_is_computed(callee: &Expression<'_>, collector: &ComputedValueSpreadCollector) -> bool {
    let mut expression = callee;
    loop {
        match expression {
            Expression::Identifier(identifier) => {
                return collector.is_computed_name(identifier.name.as_str());
            }
            Expression::ParenthesizedExpression(paren) => expression = &paren.expression,
            _ => return false,
        }
    }
}

fn identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    let mut expression = expression;
    loop {
        match expression {
            Expression::Identifier(identifier) => return Some(identifier.name.as_str()),
            Expression::ParenthesizedExpression(paren) => expression = &paren.expression,
            _ => return None,
        }
    }
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn is_ref_value_expression(expression: &Expression<'_>) -> bool {
    match unwrap_value_expression(expression) {
        Expression::StaticMemberExpression(member) => member.property.name.as_str() == "value",
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => {
                member.property.name.as_str() == "value"
            }
            _ => false,
        },
        _ => false,
    }
}

fn unwrap_value_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    let mut expression = expression;
    loop {
        expression = match expression {
            Expression::ParenthesizedExpression(paren) => &paren.expression,
            Expression::TSAsExpression(ts_as) => &ts_as.expression,
            Expression::TSSatisfiesExpression(ts_satisfies) => &ts_satisfies.expression,
            Expression::TSNonNullExpression(ts_non_null) => &ts_non_null.expression,
            other => return other,
        };
    }
}
