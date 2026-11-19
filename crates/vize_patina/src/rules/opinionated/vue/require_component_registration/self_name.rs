//! Helpers for resolving the component's *own* name so a recursive
//! self-reference in the template is not reported as unregistered (#4953).
//!
//! A `<script setup>` component can reference itself by its filename-derived
//! name or by the name declared via `defineOptions({ name })` — a documented
//! Vue feature for recursive components that needs no import.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Declaration, ExportDefaultDeclarationKind,
    Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
    VariableDeclaration,
};
use vize_croquis::Croquis;
use vize_croquis::macros::MacroKind;

/// The file name without directories or its last extension
/// (`"src/tree-item.vue"` -> `"tree-item"`). Vue's template compiler drops
/// only the last one, so `router-link.demo.vue` is `RouterLink.demo`, never
/// `<router-link>`.
pub(super) fn file_stem(filename: &str) -> &str {
    let base = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    base.rsplit_once('.').map_or(base, |(stem, _)| stem)
}

/// The component name declared via `defineOptions({ name: "..." })`, if any.
pub(super) fn define_options_name(analysis: &Croquis) -> Option<&str> {
    let call = analysis
        .macros
        .all_calls()
        .iter()
        .find(|call| matches!(call.kind, MacroKind::DefineOptions))?;
    options_name_property(call.runtime_args.as_ref()?.as_str())
}

/// The static component name declared by the Options API default export.
pub(super) fn options_api_name<'a>(program: &'a Program<'_>) -> Option<&'a str> {
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let object = match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
            ExportDefaultDeclarationKind::CallExpression(call) => component_factory_object(call),
            ExportDefaultDeclarationKind::Identifier(identifier) => {
                component_binding_object(program, identifier.name.as_str())
            }
            _ => None,
        }?;
        return object_name(object);
    }
    None
}

fn component_binding_object<'a, 'alloc>(
    program: &'a Program<'alloc>,
    binding_name: &str,
) -> Option<&'a ObjectExpression<'alloc>> {
    for statement in &program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => declaration.as_ref(),
            _ => continue,
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                continue;
            };
            if identifier.name != binding_name {
                continue;
            }
            let Some(Expression::CallExpression(call)) = declarator
                .init
                .as_ref()
                .map(Expression::get_inner_expression)
            else {
                return None;
            };
            return component_factory_object(call);
        }
    }
    None
}

fn component_factory_object<'a, 'alloc>(
    call: &'a CallExpression<'alloc>,
) -> Option<&'a ObjectExpression<'alloc>> {
    let Expression::Identifier(identifier) = call.callee.get_inner_expression() else {
        return None;
    };
    if !matches!(
        identifier.name.as_str(),
        "defineComponent" | "defineNuxtComponent"
    ) {
        return None;
    }
    call.arguments.first().and_then(argument_object)
}

/// Component factory bindings authored in the paired normal script live at
/// module scope and are visible to the script-setup template compiler.
pub(super) fn component_factory_bindings<'a>(program: &'a Program<'a>) -> Vec<&'a str> {
    let mut bindings = Vec::new();
    for statement in &program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => Some(declaration.as_ref()),
            Statement::ExportNamedDeclaration(export) => {
                export
                    .declaration
                    .as_ref()
                    .and_then(|declaration| match declaration {
                        Declaration::VariableDeclaration(declaration) => Some(declaration.as_ref()),
                        _ => None,
                    })
            }
            _ => None,
        };
        if let Some(declaration) = declaration {
            component_variables(declaration, &mut bindings);
        }
    }
    bindings
}

fn component_variables<'a>(declaration: &'a VariableDeclaration<'a>, bindings: &mut Vec<&'a str>) {
    for declarator in &declaration.declarations {
        let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
            continue;
        };
        let Some(Expression::CallExpression(call)) = declarator
            .init
            .as_ref()
            .map(Expression::get_inner_expression)
        else {
            continue;
        };
        let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
            continue;
        };
        if matches!(
            callee.name.as_str(),
            "defineComponent" | "defineNuxtComponent" | "defineAsyncComponent"
        ) {
            bindings.push(identifier.name.as_str());
        }
    }
}

fn argument_object<'a, 'alloc>(
    argument: &'a Argument<'alloc>,
) -> Option<&'a ObjectExpression<'alloc>> {
    match argument.as_expression()?.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        _ => None,
    }
}

fn object_name<'a>(object: &'a ObjectExpression<'_>) -> Option<&'a str> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let name_key = match &property.key {
            PropertyKey::StaticIdentifier(identifier) if !property.computed => {
                identifier.name == "name"
            }
            PropertyKey::StringLiteral(literal) => literal.value == "name",
            _ => false,
        };
        if !name_key {
            continue;
        }
        return match property.value.get_inner_expression() {
            Expression::StringLiteral(literal) => Some(literal.value.as_str()),
            Expression::TemplateLiteral(template) => {
                template.single_quasi().map(|value| value.as_str())
            }
            _ => None,
        };
    }
    None
}

/// Pragmatic scan of a `defineOptions` argument source for a `name` property
/// with a plain string-literal value. Returns the literal's content.
fn options_name_property(args: &str) -> Option<&str> {
    let bytes = args.as_bytes();
    let mut search = 0usize;
    while let Some(found) = args.get(search..).and_then(|rest| rest.find("name")) {
        let start = search + found;
        search = start + "name".len();
        // The key may be bare (`name:`) or quoted (`"name":`); anything else
        // directly before it (`fullname:`) is a different key.
        let before = start.checked_sub(1).and_then(|i| bytes.get(i).copied());
        let quoted_key = matches!(before, Some(b'\'' | b'"'));
        if !matches!(before, None | Some(b'{' | b',' | b'\'' | b'"'))
            && !before.is_some_and(|b| b.is_ascii_whitespace())
        {
            continue;
        }
        let mut cursor = search;
        if quoted_key {
            if bytes.get(cursor).copied() != before {
                continue;
            }
            cursor += 1;
        }
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b':') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let quote = match bytes.get(cursor) {
            Some(&q @ (b'\'' | b'"' | b'`')) => q,
            _ => continue,
        };
        cursor += 1;
        let value_start = cursor;
        while let Some(&byte) = bytes.get(cursor)
            && byte != quote
        {
            // Stay conservative on escapes and template interpolation.
            if byte == b'\\' || (quote == b'`' && byte == b'$') {
                return None;
            }
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return None;
        }
        return args.get(value_start..cursor);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{file_stem, options_name_property};

    #[test]
    fn extracts_name_property() {
        assert_eq!(
            options_name_property(r#"{ name: "TreeItem" }"#),
            Some("TreeItem")
        );
        assert_eq!(
            options_name_property(r#"{ "name": 'TreeItem' }"#),
            Some("TreeItem")
        );
        assert_eq!(
            options_name_property(r#"{ inheritAttrs: false, name: `TreeItem` }"#),
            Some("TreeItem")
        );
    }

    #[test]
    fn rejects_non_name_keys_and_dynamic_values() {
        assert_eq!(options_name_property(r#"{ fullname: "x" }"#), None);
        assert_eq!(options_name_property(r#"{ name: dynamic }"#), None);
        assert_eq!(options_name_property(r#"{ name: `a${b}` }"#), None);
        assert_eq!(options_name_property(r#"{ inheritAttrs: false }"#), None);
    }

    #[test]
    fn strips_directories_and_extension() {
        assert_eq!(file_stem("src/components/tree-item.vue"), "tree-item");
        assert_eq!(file_stem("App.vue"), "App");
        assert_eq!(file_stem(r"win\path\Item.vue"), "Item");
        assert_eq!(file_stem("demos/router-link.demo.vue"), "router-link.demo");
        assert_eq!(file_stem("Item.stories.vue"), "Item.stories");
    }
}
