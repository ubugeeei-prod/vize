//! Generate `.art.vue` text from extracted CSF metadata.

use oxc_ast::ast::{
    ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
};
use oxc_span::GetSpan;
use vize_carton::{String, append};

use super::csf::{CsfModule, CsfStory, unwrap_expression};
use super::jsx::convert_render;
use super::text::{escape_attr, escape_js_string, quote_directive_expression};

const TODO_COMMENT: &str = "<!-- TODO(vize musea migrate): unsupported story; port manually -->";

/// Outcome of generating one `.art.vue` file.
pub(super) struct EmitResult {
    /// Full `.art.vue` file content.
    pub content: String,
    /// Number of variants emitted.
    pub variants: usize,
    /// Number of variants that fell back to the manual-port TODO.
    pub todos: usize,
}

/// Render the `.art.vue` content for a CSF module.
///
/// `component_tag` is the element name used inside variants (the component's
/// local import name, or a fallback derived from the title).
pub(super) fn emit_art(
    module: &CsfModule<'_>,
    component_tag: &str,
    component_path: &str,
    source: &str,
) -> EmitResult {
    let mut content = String::default();

    content.push_str("<script setup lang=\"ts\">\n");
    append!(
        content,
        "defineArt(\"{}\", {{\n",
        escape_js_string(component_path)
    );
    let (category, title) = split_title(module.title.as_deref(), component_tag);
    if let Some(category) = category {
        append!(content, "  category: \"{}\",\n", escape_js_string(category));
    }
    append!(content, "  title: \"{}\",\n", escape_js_string(title));
    content.push_str("});\n");
    content.push_str("</script>\n\n");

    content.push_str("<art>\n");

    let mut variants = 0usize;
    let mut todos = 0usize;
    for (index, story) in module.stories.iter().enumerate() {
        let is_default = index == 0;
        let (inner, is_todo) = emit_variant_inner(story, module.meta_args, component_tag, source);
        if is_todo {
            todos += 1;
        }
        variants += 1;

        if is_default {
            append!(content, "  <variant name=\"{}\" default>\n", story.name);
        } else {
            append!(content, "  <variant name=\"{}\">\n", story.name);
        }
        for line in inner.lines() {
            if line.is_empty() {
                content.push('\n');
            } else {
                append!(content, "    {line}\n");
            }
        }
        content.push_str("  </variant>\n");
    }

    content.push_str("</art>\n");

    EmitResult {
        content,
        variants,
        todos,
    }
}

/// Build the inner markup of a `<variant>`. Returns `(markup, is_todo)`.
fn emit_variant_inner(
    story: &CsfStory<'_>,
    meta_args: Option<&ObjectExpression<'_>>,
    component_tag: &str,
    source: &str,
) -> (String, bool) {
    if story.unsupported {
        return (emit_todo_element(component_tag), true);
    }

    if let Some(render) = story.render {
        let Some(markup) = convert_render(render, source) else {
            return (emit_todo_element(component_tag), true);
        };
        return match inline_story_args_spread(markup, meta_args, story.args, source) {
            Some(markup) => (markup, false),
            None => (emit_todo_element(component_tag), true),
        };
    }

    if meta_args.is_some() || story.args.is_some() {
        if args_contain_unmigrated_bindings(meta_args, story.args) {
            return (emit_todo_element(component_tag), true);
        }
        if let Some(element) = emit_args_element(meta_args, story.args, component_tag, source) {
            return (element, false);
        }
        return (emit_todo_element(component_tag), true);
    }

    (emit_todo_element(component_tag), true)
}

fn emit_todo_element(component_tag: &str) -> String {
    let mut out = String::default();
    append!(out, "<{component_tag} />\n{TODO_COMMENT}");
    out
}

/// Emit `<Component ...props />` from an `args` object literal.
fn emit_args_element(
    meta_args: Option<&ObjectExpression<'_>>,
    story_args: Option<&ObjectExpression<'_>>,
    component_tag: &str,
    source: &str,
) -> Option<String> {
    let mut out = String::default();
    append!(out, "<{component_tag}");
    out.push_str(&emit_args_attributes(meta_args, story_args, source)?);
    out.push_str(" />");
    Some(out)
}

fn emit_args_attributes(
    meta_args: Option<&ObjectExpression<'_>>,
    story_args: Option<&ObjectExpression<'_>>,
    source: &str,
) -> Option<String> {
    let mut out = String::default();
    if let Some(args) = meta_args {
        emit_args_object_attributes(args, story_args, source, &mut out)?;
    }
    if let Some(args) = story_args {
        emit_args_object_attributes(args, None, source, &mut out)?;
    }
    Some(out)
}

fn emit_args_object_attributes(
    args: &ObjectExpression<'_>,
    overrides: Option<&ObjectExpression<'_>>,
    source: &str,
    out: &mut String,
) -> Option<()> {
    for property in &args.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            continue;
        };
        if prop.computed {
            continue;
        }
        let Some(name) = property_key_name(&prop.key) else {
            continue;
        };
        if overrides.is_some_and(|object| args_has_static_property(object, name)) {
            continue;
        }
        out.push(' ');
        out.push_str(&attribute_from_value(name, &prop.value, source)?);
    }
    Some(())
}

fn inline_story_args_spread(
    markup: String,
    meta_args: Option<&ObjectExpression<'_>>,
    story_args: Option<&ObjectExpression<'_>>,
    source: &str,
) -> Option<String> {
    let marker = " v-bind=\"args\"";
    if !markup.contains(marker) {
        return Some(markup);
    }
    meta_args.or(story_args)?;
    if args_contain_unmigrated_bindings(meta_args, story_args) {
        return None;
    }
    let attributes = emit_args_attributes(meta_args, story_args, source)?;
    Some(markup.replace(marker, attributes.as_str()).into())
}

/// Map one `args` entry to an attribute: string literal -> `name="value"`,
/// everything else -> `:name="<expr source>"`.
fn attribute_from_value(name: &str, value: &Expression<'_>, source: &str) -> Option<String> {
    let mut out = String::default();
    if let Expression::StringLiteral(literal) = unwrap_expression(value) {
        append!(out, "{name}=\"{}\"", escape_attr(literal.value.as_str()));
    } else {
        let text = value.span().source_text(source);
        let quoted = quote_directive_expression(text)?;
        append!(out, ":{name}={}", quoted.as_str());
    }
    Some(out)
}

fn args_contain_unmigrated_bindings(
    meta_args: Option<&ObjectExpression<'_>>,
    story_args: Option<&ObjectExpression<'_>>,
) -> bool {
    if let Some(args) = meta_args
        && args_object_contains_unmigrated_bindings(args, story_args)
    {
        return true;
    }
    story_args.is_some_and(|args| args_object_contains_unmigrated_bindings(args, None))
}

fn args_object_contains_unmigrated_bindings(
    args: &ObjectExpression<'_>,
    overrides: Option<&ObjectExpression<'_>>,
) -> bool {
    args.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            return true;
        };
        if prop.computed {
            return true;
        }
        let Some(name) = property_key_name(&prop.key) else {
            return true;
        };
        if overrides.is_some_and(|object| args_has_static_property(object, name)) {
            return false;
        }
        expression_needs_module_binding(&prop.value)
    })
}

fn expression_needs_module_binding(expression: &Expression<'_>) -> bool {
    match unwrap_expression(expression) {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_) => false,
        Expression::TemplateLiteral(template) => !template.expressions.is_empty(),
        Expression::UnaryExpression(unary) => expression_needs_module_binding(&unary.argument),
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            ArrayExpressionElement::Elision(_) => false,
            ArrayExpressionElement::SpreadElement(_) => true,
            _ => element
                .as_expression()
                .is_none_or(expression_needs_module_binding),
        }),
        Expression::ObjectExpression(object) => {
            args_object_contains_unmigrated_bindings(object, None)
        }
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::CallExpression(_) => true,
        _ => true,
    }
}

fn args_has_static_property(args: &ObjectExpression<'_>, name: &str) -> bool {
    args.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            return false;
        };
        !prop.computed && property_key_name(&prop.key) == Some(name)
    })
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

/// Split a CSF title `Category/Name` into `(Some(category), name)`; a plain
/// title yields `(None, title)`. Falls back to `component_tag` if no title.
fn split_title<'a>(title: Option<&'a str>, component_tag: &'a str) -> (Option<&'a str>, &'a str) {
    let Some(title) = title else {
        return (None, component_tag);
    };
    match title.rsplit_once('/') {
        Some((category, name)) if !category.is_empty() && !name.is_empty() => {
            (Some(category), name)
        }
        _ => (None, title),
    }
}

#[cfg(test)]
mod tests;
