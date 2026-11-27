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

/// What the `<script>` blocks of one SFC do with template refs, gathered in
/// one walk per block for every ref of the template.
#[derive(Default)]
struct ScriptRefUses {
    /// Ref names the scripts read: `useTemplateRef(key)` keys, static `$refs`
    /// reads and destructuring, names returned from Options API `setup()`, and
    /// template-visible `<script setup>` top-level bindings.
    used: FxHashSet<CompactString>,
    /// `<script setup>` top-level bindings initialized by Vue's
    /// `useTemplateRef(key)`, mapped to their key. The key wires the ref, not
    /// the binding name: a `ref="binding"` attribute stays null at runtime.
    handles: FxHashMap<CompactString, CompactString>,
    /// Some ref may be read without a static name: computed `$refs` access,
    /// `$refs` escaping as a whole, or a `useTemplateRef` key that is not a
    /// static string.
    opaque: bool,
}

/// How a script uses one template ref name.
enum RefUsage<'u> {
    Used,
    /// Unused; carries the key of a same-named `useTemplateRef` handle.
    Unused(Option<&'u str>),
}

impl ScriptRefUses {
    fn collect(ctx: &LintContext<'_>) -> Self {
        let mut uses = Self::default();
        let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
            return uses;
        };
        if let Some(parsed) = scripts.script() {
            uses.scan(
                &parsed.program,
                Self::block_content(descriptor.script.as_ref()),
                true,
            );
        }
        if let Some(parsed) = scripts.script_setup() {
            uses.scan(
                &parsed.program,
                Self::block_content(descriptor.script_setup.as_ref()),
                false,
            );
        }
        uses
    }

    /// The content of an optional script block, empty when absent.
    fn block_content<'b>(block: Option<&'b vize_atelier_sfc::SfcScriptBlock<'_>>) -> &'b str {
        block.map_or("", |block| block.content.as_ref())
    }

    fn scan(&mut self, program: &Program<'_>, source: &str, export_default: bool) {
        let built = SemanticBuilder::new().with_build_nodes(true).build(program);
        let semantic = &built.semantic;
        if !export_default {
            self.collect_setup_bindings(program, semantic);
        }
        let objects = VueObjects::collect(program, source, semantic, export_default);
        let mut visitor = RefUseVisitor {
            uses: self,
            objects: &objects,
            semantic,
        };
        visitor.visit_program(program);
    }

    fn usage(&self, ctx: &LintContext<'_>, name: &str) -> RefUsage<'_> {
        let handle = self.handles.get(name);
        if self.opaque
            || self.used.contains(name)
            || (handle.is_none() && binding_uses_template_ref(ctx, name))
        {
            RefUsage::Used
        } else {
            RefUsage::Unused(handle.map(CompactString::as_str))
        }
    }

    /// Record the template-visible top-level bindings of `<script setup>`.
    fn collect_setup_bindings(&mut self, program: &Program<'_>, semantic: &Semantic<'_>) {
        for statement in &program.body {
            match statement {
                Statement::ImportDeclaration(import) if !import.import_kind.is_type() => {
                    for specifier in import.specifiers.iter().flatten() {
                        if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
                            && specifier.import_kind.is_type()
                        {
                            continue;
                        }
                        self.used.insert(specifier.local().name.as_str().into());
                    }
                }
                Statement::ExportNamedDeclaration(export) if !export.export_kind.is_type() => {
                    if let Some(declaration) = &export.declaration {
                        self.collect_setup_declaration(declaration, semantic);
                    }
                }
                _ => {
                    if let Some(declaration) = statement.as_declaration() {
                        self.collect_setup_declaration(declaration, semantic);
                    }
                }
            }
        }
    }

    fn collect_setup_declaration(
        &mut self,
        declaration: &Declaration<'_>,
        semantic: &Semantic<'_>,
    ) {
        match declaration {
            Declaration::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if let BindingPattern::BindingIdentifier(binding) = &declarator.id
                        && let Some(Expression::CallExpression(call)) = declarator
                            .init
                            .as_ref()
                            .map(Expression::get_inner_expression)
                        && is_vue_use_template_ref(&call.callee, semantic)
                    {
                        // The visitor records the key itself, or marks a
                        // dynamic key opaque.
                        if let Some(key) = call
                            .arguments
                            .first()
                            .and_then(Argument::as_expression)
                            .and_then(|key| static_key(key, semantic, 0))
                        {
                            self.handles.insert(binding.name.as_str().into(), key);
                        }
                        continue;
                    }
                    for identifier in declarator.id.get_binding_identifiers() {
                        self.used.insert(identifier.name.as_str().into());
                    }
                }
            }
            Declaration::FunctionDeclaration(function) => {
                if let Some(identifier) = &function.id {
                    self.used.insert(identifier.name.as_str().into());
                }
            }
            Declaration::ClassDeclaration(class) => {
                if let Some(identifier) = &class.id {
                    self.used.insert(identifier.name.as_str().into());
                }
            }
            _ => {}
        }
    }
}

/// Modules that export Vue's `useTemplateRef`.
fn is_vue_api_module(source: &str) -> bool {
    crate::rules::script::composition_api::is_composition_api_module(source)
        || source.starts_with("@vue/runtime-")
}

/// Whether `callee` is Vue's `useTemplateRef`: imported from Vue under any
/// local name, read from a Vue namespace import, or the unbound global an
/// auto-import (Nuxt, unplugin-auto-import) provides. A local or non-Vue
/// binding named `useTemplateRef` is not Vue's.
fn is_vue_use_template_ref(callee: &Expression<'_>, semantic: &Semantic<'_>) -> bool {
    let vue_import = |identifier: &IdentifierReference<'_>, namespace: bool| {
        let symbol = identifier
            .reference_id
            .get()
            .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())?;
        let declaration = semantic.symbol_declaration(symbol);
        let imported = match declaration.kind() {
            AstKind::ImportSpecifier(specifier)
                if !namespace && !specifier.import_kind.is_type() =>
            {
                specifier.imported.name().as_str() == "useTemplateRef"
            }
            AstKind::ImportNamespaceSpecifier(_) => namespace,
            _ => false,
        };
        Some(
            imported
                && matches!(
                    semantic.nodes().parent_kind(declaration.id()),
                    AstKind::ImportDeclaration(import)
                        if !import.import_kind.is_type() && is_vue_api_module(import.source.value.as_str())
                ),
        )
    };
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => {
            vue_import(identifier, false).unwrap_or(identifier.name == "useTemplateRef")
        }
        Expression::StaticMemberExpression(member) => {
            member.property.name == "useTemplateRef"
                && matches!(&member.object, Expression::Identifier(namespace)
                    if vue_import(namespace, true) == Some(true))
        }
        _ => false,
    }
}

/// The string a `useTemplateRef` key evaluates to, when it is static: a string
/// literal or substitution-free template literal, through parentheses and
/// TypeScript wrappers (`as const`, `satisfies`, `!`), or a `const` bound to
/// one. `None` means the key is only known at runtime.
fn static_key(key: &Expression<'_>, semantic: &Semantic<'_>, depth: u8) -> Option<CompactString> {
    match key.get_inner_expression() {
        Expression::StringLiteral(literal) => Some(literal.value.as_str().into()),
        Expression::TemplateLiteral(literal) => {
            literal.single_quasi().map(|quasi| quasi.as_str().into())
        }
        Expression::Identifier(identifier) if depth < 8 => {
            let symbol = semantic
                .scoping()
                .get_reference(identifier.reference_id.get()?)
                .symbol_id()?;
            let AstKind::VariableDeclarator(declarator) =
                semantic.symbol_declaration(symbol).kind()
            else {
                return None;
            };
            if declarator.kind != VariableDeclarationKind::Const
                || !matches!(declarator.id, BindingPattern::BindingIdentifier(_))
            {
                return None;
            }
            static_key(declarator.init.as_ref()?, semantic, depth + 1)
        }
        _ => None,
    }
}

struct RefUseVisitor<'u, 's, 'a> {
    uses: &'u mut ScriptRefUses,
    objects: &'u VueObjects,
    semantic: &'s Semantic<'a>,
}

impl RefUseVisitor<'_, '_, '_> {
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

    fn use_name(&mut self, name: &str) {
        self.uses.used.insert(name.into());
    }
}

impl<'a> Visit<'a> for RefUseVisitor<'_, '_, 'a> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let Some(init) = &declarator.init
            && Self::refs_object(init)
        {
            if let BindingPattern::ObjectPattern(pattern) = &declarator.id {
                for property in &pattern.properties {
                    match property_key_name(&property.key) {
                        Some(name) if !property.computed => self.use_name(name),
                        _ => self.uses.opaque = true,
                    }
                }
                self.uses.opaque |= pattern.rest.is_some();
            } else {
                self.uses.opaque = true;
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
                if property.computed || property_key_name(&property.key) != Some("setup") {
                    continue;
                }
                let Some(returned) = returned_object(&property.value) else {
                    continue;
                };
                for property in &returned.properties {
                    if let ObjectPropertyKind::ObjectProperty(property) = property
                        && !property.computed
                        && let Some(name) = property_key_name(&property.key)
                    {
                        self.use_name(name);
                    }
                }
            }
        }
        walk::walk_object_expression(self, object);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // A project binding named `useTemplateRef` is most likely a re-export
        // of Vue's, so its key keeps the ref alive too.
        let reads_key = is_vue_use_template_ref(&call.callee, self.semantic)
            || matches!(call.callee.get_inner_expression(),
                Expression::Identifier(callee) if callee.name == "useTemplateRef");
        if reads_key && let Some(key) = call.arguments.first() {
            match key
                .as_expression()
                .and_then(|key| static_key(key, self.semantic, 0))
            {
                Some(key) => {
                    self.uses.used.insert(key);
                }
                // A key computed at runtime (a composable parameter, a
                // function result) can name any template ref.
                None => self.uses.opaque = true,
            }
        }
        if call
            .arguments
            .iter()
            .any(|argument| argument.as_expression().is_some_and(Self::refs_object))
        {
            self.uses.opaque = true;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if Self::refs_object(&member.object) {
            self.use_name(&member.property.name);
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if Self::refs_object(&member.object) {
            match member.static_property_name() {
                Some(property) => self.use_name(&property),
                None => self.uses.opaque = true,
            }
        }
        walk::walk_computed_member_expression(self, member);
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if statement.argument.as_ref().is_some_and(Self::refs_object) {
            self.uses.opaque = true;
        }
        walk::walk_return_statement(self, statement);
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        if Self::refs_object(&statement.right) {
            self.uses.opaque = true;
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
