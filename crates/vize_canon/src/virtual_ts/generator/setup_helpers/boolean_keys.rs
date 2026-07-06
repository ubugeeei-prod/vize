use oxc_allocator::Allocator;
use oxc_ast::ast::{
    CallExpression, Declaration, Expression, PropertyKey, Statement, TSInterfaceBody,
    TSInterfaceDeclaration, TSLiteral, TSSignature, TSType, TSTypeAliasDeclaration, TSTypeLiteral,
    TSTypeName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};
use vize_croquis::macros::DEFINE_PROPS;

pub(super) fn collect_define_props_boolean_keys(script: &str) -> Option<Vec<String>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }

    let declarations = TypeDeclarations::from_statements(&parsed.program.body);
    let mut collector = DefinePropsBooleanKeyCollector {
        declarations,
        keys: FxHashSet::default(),
        resolving: Vec::new(),
        saw_type_only_define_props: false,
    };
    collector.visit_program(&parsed.program);
    if !collector.saw_type_only_define_props {
        return None;
    }

    let mut keys: Vec<String> = collector.keys.into_iter().collect();
    keys.sort_unstable();
    Some(keys)
}

struct TypeDeclarations<'a> {
    interfaces: FxHashMap<&'a str, &'a TSInterfaceDeclaration<'a>>,
    aliases: FxHashMap<&'a str, &'a TSTypeAliasDeclaration<'a>>,
}

impl<'a> TypeDeclarations<'a> {
    fn from_statements(statements: &'a oxc_allocator::Vec<'a, Statement<'a>>) -> Self {
        let mut declarations = Self {
            interfaces: FxHashMap::default(),
            aliases: FxHashMap::default(),
        };
        for statement in statements {
            match statement {
                Statement::TSInterfaceDeclaration(interface) => {
                    declarations
                        .interfaces
                        .insert(interface.id.name.as_str(), interface);
                }
                Statement::TSTypeAliasDeclaration(alias) => {
                    declarations.aliases.insert(alias.id.name.as_str(), alias);
                }
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(declaration) = &export.declaration {
                        declarations.insert_exported_declaration(declaration);
                    }
                }
                _ => {}
            }
        }
        declarations
    }

    fn insert_exported_declaration(&mut self, declaration: &'a Declaration<'a>) {
        match declaration {
            Declaration::TSInterfaceDeclaration(interface) => {
                self.interfaces
                    .insert(interface.id.name.as_str(), interface);
            }
            Declaration::TSTypeAliasDeclaration(alias) => {
                self.aliases.insert(alias.id.name.as_str(), alias);
            }
            _ => {}
        }
    }
}

struct DefinePropsBooleanKeyCollector<'a> {
    declarations: TypeDeclarations<'a>,
    keys: FxHashSet<String>,
    resolving: Vec<String>,
    saw_type_only_define_props: bool,
}

impl<'a> Visit<'a> for DefinePropsBooleanKeyCollector<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_define_props_call(call)
            && let Some(type_args) = &call.type_arguments
            && let Some(first) = type_args.params.first()
        {
            self.saw_type_only_define_props = true;
            self.collect_from_type(first);
        }

        walk::walk_call_expression(self, call);
    }
}

impl<'a> DefinePropsBooleanKeyCollector<'a> {
    fn collect_from_type(&mut self, ty: &TSType<'a>) {
        match ty {
            TSType::TSTypeLiteral(literal) => self.collect_from_type_literal(literal),
            TSType::TSTypeReference(type_ref) => {
                let Some(name) = simple_type_name(&type_ref.type_name) else {
                    return;
                };
                if self.resolving.iter().any(|resolving| resolving == name) {
                    return;
                }
                self.resolving.push(name.into());
                if let Some(interface) = self.declarations.interfaces.get(name).copied() {
                    self.collect_from_interface(interface);
                } else if let Some(alias) = self.declarations.aliases.get(name).copied() {
                    self.collect_from_type(&alias.type_annotation);
                }
                self.resolving.pop();
            }
            TSType::TSIntersectionType(intersection) => {
                for part in &intersection.types {
                    self.collect_from_type(part);
                }
            }
            TSType::TSParenthesizedType(parenthesized) => {
                self.collect_from_type(&parenthesized.type_annotation);
            }
            _ => {}
        }
    }

    fn collect_from_interface(&mut self, interface: &TSInterfaceDeclaration<'a>) {
        for heritage in &interface.extends {
            if let Some(name) = simple_expression_name(&heritage.expression) {
                if self.resolving.iter().any(|resolving| resolving == name) {
                    continue;
                }
                self.resolving.push(name.into());
                if let Some(parent) = self.declarations.interfaces.get(name).copied() {
                    self.collect_from_interface(parent);
                } else if let Some(alias) = self.declarations.aliases.get(name).copied() {
                    self.collect_from_type(&alias.type_annotation);
                }
                self.resolving.pop();
            }
        }
        self.collect_from_interface_body(&interface.body);
    }

    fn collect_from_type_literal(&mut self, literal: &TSTypeLiteral<'a>) {
        self.collect_from_signatures(&literal.members);
    }

    fn collect_from_interface_body(&mut self, body: &TSInterfaceBody<'a>) {
        self.collect_from_signatures(&body.body);
    }

    fn collect_from_signatures(&mut self, members: &oxc_allocator::Vec<'a, TSSignature<'a>>) {
        for member in members {
            let TSSignature::TSPropertySignature(property) = member else {
                continue;
            };
            let Some(type_annotation) = &property.type_annotation else {
                continue;
            };
            if !is_boolean_prop_type(&type_annotation.type_annotation) {
                continue;
            }
            if let Some(name) = property_key_name(&property.key) {
                self.keys.insert(name);
            }
        }
    }
}

fn is_define_props_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == DEFINE_PROPS
    )
}

fn is_boolean_prop_type(ty: &TSType<'_>) -> bool {
    match ty {
        TSType::TSBooleanKeyword(_) => true,
        TSType::TSLiteralType(literal) => {
            matches!(literal.literal, TSLiteral::BooleanLiteral(_))
        }
        TSType::TSUnionType(union) => {
            let mut has_boolean = false;
            for part in &union.types {
                match part {
                    TSType::TSUndefinedKeyword(_) => {}
                    TSType::TSBooleanKeyword(_) => has_boolean = true,
                    TSType::TSLiteralType(literal)
                        if matches!(literal.literal, TSLiteral::BooleanLiteral(_)) =>
                    {
                        has_boolean = true;
                    }
                    TSType::TSParenthesizedType(parenthesized)
                        if is_boolean_prop_type(&parenthesized.type_annotation) =>
                    {
                        has_boolean = true;
                    }
                    _ => return false,
                }
            }
            has_boolean
        }
        TSType::TSParenthesizedType(parenthesized) => {
            is_boolean_prop_type(&parenthesized.type_annotation)
        }
        _ => false,
    }
}

fn property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str().into()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str().into()),
        PropertyKey::NumericLiteral(literal) => Some(literal.value.to_compact_string()),
        _ => None,
    }
}

fn simple_type_name<'a>(type_name: &'a TSTypeName<'_>) -> Option<&'a str> {
    match type_name {
        TSTypeName::IdentifierReference(id) => Some(id.name.as_str()),
        _ => None,
    }
}

fn simple_expression_name<'a>(expression: &'a Expression<'_>) -> Option<&'a str> {
    match expression {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::collect_define_props_boolean_keys;

    #[test]
    fn collects_boolean_keys_from_local_type_ast() {
        let keys = collect_define_props_boolean_keys(
            r#"
interface Base {
  disabled?: boolean | undefined;
}
interface Props<T> extends Base {
  as?: string;
  active: true | false;
  value: T;
}
defineProps<Props<Record<string, unknown>>>();
"#,
        )
        .expect("type-only defineProps should be detected");

        assert_eq!(keys, vec!["active", "disabled"]);
    }

    #[test]
    fn ignores_generic_object_props_when_collecting_boolean_keys() {
        let keys = collect_define_props_boolean_keys(
            r#"
interface Props<T> {
  as?: string;
  value: T;
}
defineProps<Props<Record<string, unknown>>>();
"#,
        )
        .expect("type-only defineProps should be detected");

        assert!(keys.is_empty());
    }
}
