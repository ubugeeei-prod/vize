//! What a bound value is, read the way Nuxt's template diagnostics read it.
//!
//! An expression is described from its syntax: a function, a class or a
//! class instance, looking through parentheses, TypeScript casts, branches of
//! `?:`, `&&`/`||`, object values and array elements. A name is followed only
//! to a `<script setup>` top-level declaration whose Vue binding is
//! `setup-const` and that no `v-for`/`v-slot` in scope shadows.

use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, FunctionType, ObjectExpression, ObjectPropertyKind,
    PropertyKey, Statement, VariableDeclarationKind,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::script::{analyze_script_bindings, analyze_script_setup};
use vize_atelier_sfc::{BindingMetadata, BindingType, SfcDescriptor};
use vize_l0::{Allocator, FxHashMap, String};

const FUNCTION: &str = "a function";
const CLASS: &str = "a class";
const INSTANCE: &str = "a class instance";

/// The script facts a description needs, read once per SFC on first use.
pub(super) struct Values<'s> {
    descriptor: &'s SfcDescriptor<'s>,
    bindings: Option<BindingMetadata>,
    declarations: Option<FxHashMap<String, &'static str>>,
}

impl<'s> Values<'s> {
    pub(super) fn new(descriptor: &'s SfcDescriptor<'s>) -> Self {
        Self {
            descriptor,
            bindings: None,
            declarations: None,
        }
    }

    /// Whether the template's binding metadata names `name`.
    pub(super) fn is_bound(&mut self, name: &str) -> bool {
        self.bindings().bindings.contains_key(name)
    }

    /// Describe a directive value: `content` as Vue parsed it, `authored` as
    /// written. Vue leaves a lone identifier unparsed, so it is described by
    /// its authored text.
    pub(super) fn describe_expression(
        &mut self,
        content: &str,
        authored: &str,
        scope: &[&str],
    ) -> Option<&'static str> {
        if content.trim().is_empty() {
            return None;
        }
        if is_simple_identifier(content) {
            return self.describe_name(authored.trim(), scope);
        }
        let allocator = Allocator::new();
        let expression = parse_expression(&allocator, content)?;
        self.describe(&expression, scope)
    }

    /// Each described entry of an object literal value, keyed by its static
    /// name (`None` for a computed or numeric key).
    pub(super) fn describe_object_entries(
        &mut self,
        content: &str,
        scope: &[&str],
    ) -> Vec<(Option<String>, &'static str)> {
        let mut entries = Vec::new();
        if content.trim().is_empty() || is_simple_identifier(content) {
            return entries;
        }
        let allocator = Allocator::new();
        let Some(expression) = parse_expression(&allocator, content) else {
            return entries;
        };
        let Expression::ObjectExpression(object) = without_parentheses(&expression) else {
            return entries;
        };
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if let Some(value) = self.describe(&property.value, scope) {
                let key = if property.computed {
                    None
                } else {
                    static_key(&property.key)
                };
                entries.push((key, value));
            }
        }
        entries
    }

    fn describe(&mut self, expression: &Expression<'_>, scope: &[&str]) -> Option<&'static str> {
        match expression {
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
                Some(FUNCTION)
            }
            Expression::ClassExpression(_) => Some(CLASS),
            Expression::NewExpression(_) => Some(INSTANCE),
            Expression::Identifier(identifier) => self.describe_name(&identifier.name, scope),
            Expression::ParenthesizedExpression(inner) => self.describe(&inner.expression, scope),
            Expression::TSAsExpression(inner) => self.describe(&inner.expression, scope),
            Expression::TSSatisfiesExpression(inner) => self.describe(&inner.expression, scope),
            Expression::TSNonNullExpression(inner) => self.describe(&inner.expression, scope),
            Expression::ConditionalExpression(inner) => self
                .describe(&inner.consequent, scope)
                .or_else(|| self.describe(&inner.alternate, scope)),
            Expression::LogicalExpression(inner) => self
                .describe(&inner.left, scope)
                .or_else(|| self.describe(&inner.right, scope)),
            Expression::ObjectExpression(object) => self.describe_object(object, scope),
            Expression::ArrayExpression(array) => array
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
                .find_map(|element| self.describe(element, scope)),
            _ => None,
        }
    }

    fn describe_object(
        &mut self,
        object: &ObjectExpression<'_>,
        scope: &[&str],
    ) -> Option<&'static str> {
        object
            .properties
            .iter()
            .find_map(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    self.describe(&property.value, scope)
                }
                ObjectPropertyKind::SpreadProperty(_) => None,
            })
    }

    /// A name declared at the top of `<script setup>` as a function or class,
    /// or a `const` holding one, whose binding Vue keeps constant.
    fn describe_name(&mut self, name: &str, scope: &[&str]) -> Option<&'static str> {
        if !is_ascii_identifier(name) || scope.contains(&name) {
            return None;
        }
        if self.bindings().bindings.get(name) != Some(&BindingType::SetupConst) {
            return None;
        }
        self.declarations().get(name).copied()
    }

    fn bindings(&mut self) -> &BindingMetadata {
        let descriptor = self.descriptor;
        self.bindings
            .get_or_insert_with(|| match (&descriptor.script_setup, &descriptor.script) {
                (Some(setup), _) => analyze_script_setup(&setup.content),
                (None, Some(script)) => analyze_script_bindings(&script.content),
                (None, None) => BindingMetadata::default(),
            })
    }

    fn declarations(&mut self) -> &FxHashMap<String, &'static str> {
        let descriptor = self.descriptor;
        self.declarations.get_or_insert_with(|| {
            descriptor
                .script_setup
                .as_ref()
                .map(|setup| setup_declarations(&setup.content, setup.lang.as_deref()))
                .unwrap_or_default()
        })
    }
}

/// Vue parses a directive value as `(${content})` with TypeScript enabled.
fn parse_expression<'a>(allocator: &'a Allocator, content: &str) -> Option<Expression<'a>> {
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    let text = allocator.alloc_str(&wrapped);
    Parser::new(allocator, text, SourceType::ts())
        .parse_expression()
        .ok()
}

fn without_parentheses<'e, 'a>(mut expression: &'e Expression<'a>) -> &'e Expression<'a> {
    while let Expression::ParenthesizedExpression(inner) = expression {
        expression = &inner.expression;
    }
    expression
}

fn static_key(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str().into()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str().into()),
        _ => None,
    }
}

/// The kind of value each top-level `<script setup>` declaration holds, parsed
/// as Nuxt parses the block: by its `lang`, TypeScript by default, keeping
/// parentheses, and reading what parsed when the block has errors.
fn setup_declarations(content: &str, lang: Option<&str>) -> FxHashMap<String, &'static str> {
    let source_type = match lang {
        Some("js") => SourceType::mjs(),
        Some("jsx") => SourceType::jsx(),
        Some("tsx") => SourceType::tsx(),
        _ => SourceType::ts(),
    };
    let allocator = Allocator::new();
    let parsed = Parser::new(&allocator, content, source_type).parse();
    let mut declarations = FxHashMap::default();
    for statement in &parsed.program.body {
        let declaration = match statement {
            Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
            statement => statement.as_declaration(),
        };
        match declaration {
            Some(Declaration::FunctionDeclaration(function))
                if function.r#type == FunctionType::FunctionDeclaration =>
            {
                if let Some(id) = &function.id {
                    declarations.insert(id.name.as_str().into(), FUNCTION);
                }
            }
            Some(Declaration::ClassDeclaration(class)) => {
                if let Some(id) = &class.id {
                    declarations.insert(id.name.as_str().into(), CLASS);
                }
            }
            Some(Declaration::VariableDeclaration(variables))
                if variables.kind == VariableDeclarationKind::Const =>
            {
                for declarator in &variables.declarations {
                    let (BindingPattern::BindingIdentifier(id), Some(init)) =
                        (&declarator.id, &declarator.init)
                    else {
                        continue;
                    };
                    let value = match init {
                        Expression::ArrowFunctionExpression(_)
                        | Expression::FunctionExpression(_) => FUNCTION,
                        Expression::ClassExpression(_) => CLASS,
                        Expression::NewExpression(_) => INSTANCE,
                        _ => continue,
                    };
                    declarations.insert(id.name.as_str().into(), value);
                }
            }
            _ => {}
        }
    }
    declarations
}

/// Vue's `isSimpleIdentifier`: no character outside `[$\w\xA0-￿]` and no
/// leading digit.
fn is_simple_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let allowed = |c: char| c == '$' || c == '_' || c.is_ascii_alphanumeric() || c >= '\u{a0}';
    !first.is_ascii_digit() && allowed(first) && chars.all(allowed)
}

/// Nuxt's `/^[$A-Z_][\w$]*$/i`.
fn is_ascii_identifier(text: &str) -> bool {
    let mut bytes = text.bytes();
    bytes
        .next()
        .is_some_and(|first| first == b'$' || first == b'_' || first.is_ascii_alphabetic())
        && bytes.all(is_name_byte)
}

fn is_name_byte(byte: u8) -> bool {
    byte == b'$' || byte == b'_' || byte.is_ascii_alphanumeric()
}

/// The names Nuxt takes from a `v-for` alias or `v-slot` value: every
/// `/[$A-Z_][\w$]*/gi` match.
pub(super) fn scoped_names(text: &str) -> impl Iterator<Item = &str> {
    let bytes = text.as_bytes();
    let mut index = 0;
    std::iter::from_fn(move || {
        while index < bytes.len() {
            let start = index;
            let first = bytes.get(start).copied().unwrap_or_default();
            index += 1;
            if first == b'$' || first == b'_' || first.is_ascii_alphabetic() {
                while bytes.get(index).copied().is_some_and(is_name_byte) {
                    index += 1;
                }
                return text.get(start..index);
            }
        }
        None
    })
}
