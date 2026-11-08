/// Script-setup top-level bindings and values returned from Options API
/// `setup()` are exposed to the template under their binding name.
fn binding_uses_template_ref(ctx: &LintContext<'_>, name: &str) -> bool {
    let Some(analysis) = ctx.analysis() else {
        return false;
    };
    let Some(kind) = analysis.bindings.get(name) else {
        return false;
    };
    analysis.bindings.is_script_setup
        && matches!(
            kind,
            BindingType::SetupLet
                | BindingType::SetupMaybeRef
                | BindingType::SetupRef
                | BindingType::SetupReactiveConst
                | BindingType::SetupConst
                | BindingType::LiteralConst
                | BindingType::ExternalModule
        )
}
fn program_uses_template_ref(
    program: &Program<'_>,
    source: &str,
    export_default: bool,
    name: &str,
) -> bool {
    if !export_default && script_setup_defines(program, name) {
        return true;
    }
    let built = SemanticBuilder::new().with_build_nodes(true).build(program);
    let objects = VueObjects::collect(program, source, &built.semantic, export_default);
    let mut visitor = RefUseVisitor {
        name,
        used: false,
        opaque: false,
        objects: &objects,
    };
    visitor.visit_program(program);
    visitor.used || visitor.opaque
}

fn script_setup_defines(program: &Program<'_>, name: &str) -> bool {
    let declaration_defines = |declaration: &Declaration<'_>| match declaration {
        Declaration::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .any(|declarator| binding_declares(&declarator.id, name)),
        Declaration::FunctionDeclaration(function) => function
            .id
            .as_ref()
            .is_some_and(|identifier| identifier.name == name),
        Declaration::ClassDeclaration(class) => class
            .id
            .as_ref()
            .is_some_and(|identifier| identifier.name == name),
        _ => false,
    };
    program.body.iter().any(|statement| match statement {
        Statement::ImportDeclaration(import) if !import.import_kind.is_type() => {
            import.specifiers.as_ref().is_some_and(|specifiers| {
                specifiers.iter().any(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type() =>
                    {
                        specifier.local.name == name
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        specifier.local.name == name
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        specifier.local.name == name
                    }
                    _ => false,
                })
            })
        }
        Statement::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .any(|declarator| binding_declares(&declarator.id, name)),
        Statement::FunctionDeclaration(function) => function
            .id
            .as_ref()
            .is_some_and(|identifier| identifier.name == name),
        Statement::ClassDeclaration(class) => class
            .id
            .as_ref()
            .is_some_and(|identifier| identifier.name == name),
        Statement::ExportNamedDeclaration(export) if !export.export_kind.is_type() => {
            export.declaration.as_ref().is_some_and(declaration_defines)
        }
        _ => false,
    })
}

fn binding_declares(pattern: &BindingPattern<'_>, name: &str) -> bool {
    pattern
        .get_binding_identifiers()
        .into_iter()
        .any(|identifier| identifier.name == name)
}

struct RefUseVisitor<'n> {
    name: &'n str,
    used: bool,
    opaque: bool,
    objects: &'n VueObjects,
}

impl RefUseVisitor<'_> {
    fn refs_object(expression: &Expression<'_>) -> bool {
        match expression.get_inner_expression() {
            Expression::Identifier(identifier) => identifier.name == "$refs",
            Expression::StaticMemberExpression(member) => member.property.name == "$refs",
            Expression::ComputedMemberExpression(member) => {
                member.static_property_name().as_deref() == Some("$refs")
            }
            _ => false,
        }
    }
}

impl<'a> Visit<'a> for RefUseVisitor<'_> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let Some(init) = &declarator.init
            && Self::refs_object(init)
        {
            if let BindingPattern::ObjectPattern(pattern) = &declarator.id {
                self.used |= pattern.properties.iter().any(|property| {
                    !property.computed && property_key_name(&property.key) == Some(self.name)
                });
                self.opaque |= pattern.rest.is_some()
                    || pattern.properties.iter().any(|property| property.computed);
            } else {
                self.opaque = true;
            }
        }
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        if self.objects.contains(object) {
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if !property.computed
                    && property_key_name(&property.key) == Some("setup")
                    && returned_object(&property.value).is_some_and(|returned| {
                        returned.properties.iter().any(|property| {
                            let ObjectPropertyKind::ObjectProperty(property) = property else {
                                return false;
                            };
                            !property.computed
                                && property_key_name(&property.key) == Some(self.name)
                        })
                    })
                {
                    self.used = true;
                }
            }
        }
        walk::walk_object_expression(self, object);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && callee.name == "useTemplateRef"
            && let Some(Argument::StringLiteral(argument)) = call.arguments.first()
            && argument.value == self.name
        {
            self.used = true;
        }
        if call
            .arguments
            .iter()
            .any(|argument| argument.as_expression().is_some_and(Self::refs_object))
        {
            self.opaque = true;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if Self::refs_object(&member.object) && member.property.name == self.name {
            self.used = true;
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if Self::refs_object(&member.object) {
            if let Some(property) = member.static_property_name() {
                self.used |= property == self.name;
            } else {
                self.opaque = true;
            }
        }
        walk::walk_computed_member_expression(self, member);
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if statement.argument.as_ref().is_some_and(Self::refs_object) {
            self.opaque = true;
        }
        walk::walk_return_statement(self, statement);
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        if Self::refs_object(&statement.right) {
            self.opaque = true;
        }
        walk::walk_for_in_statement(self, statement);
    }
}

fn returned_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()?
            .statements
            .iter()
            .find_map(returned_object_statement),
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            let Statement::ExpressionStatement(statement) = arrow.body.statements.first()? else {
                return None;
            };
            object_expression(&statement.expression)
        }
        Expression::ArrowFunctionExpression(arrow) => arrow
            .body
            .statements
            .iter()
            .find_map(returned_object_statement),
        _ => None,
    }
}

fn returned_object_statement<'a>(statement: &'a Statement<'a>) -> Option<&'a ObjectExpression<'a>> {
    let Statement::ReturnStatement(statement) = statement else {
        return None;
    };
    object_expression(statement.argument.as_ref()?)
}

fn object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}
