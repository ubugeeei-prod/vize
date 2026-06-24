//! Generate `.art.vue` text from extracted CSF metadata.

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey};
use oxc_span::GetSpan;
use vize_carton::{String, append};

use super::csf::{CsfModule, CsfStory, unwrap_expression};
use super::jsx::convert_render;
use super::text::{escape_attr, escape_js_string};

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
        let (inner, is_todo) = emit_variant_inner(story, component_tag, source);
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
fn emit_variant_inner(story: &CsfStory<'_>, component_tag: &str, source: &str) -> (String, bool) {
    if let Some(render) = story.render
        && let Some(markup) = convert_render(render, source)
    {
        return (markup, false);
    }

    if let Some(args) = story.args {
        return (emit_args_element(args, component_tag, source), false);
    }

    let mut out = String::default();
    append!(out, "<{component_tag} />\n{TODO_COMMENT}");
    (out, true)
}

/// Emit `<Component ...props />` from an `args` object literal.
fn emit_args_element(args: &ObjectExpression<'_>, component_tag: &str, source: &str) -> String {
    let mut out = String::default();
    append!(out, "<{component_tag}");
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
        out.push(' ');
        out.push_str(&attribute_from_value(name, &prop.value, source));
    }
    out.push_str(" />");
    out
}

/// Map one `args` entry to an attribute: string literal -> `name="value"`,
/// everything else -> `:name="<expr source>"`.
fn attribute_from_value(name: &str, value: &Expression<'_>, source: &str) -> String {
    let mut out = String::default();
    if let Expression::StringLiteral(literal) = unwrap_expression(value) {
        append!(out, "{name}=\"{}\"", escape_attr(literal.value.as_str()));
    } else {
        let text = value.span().source_text(source);
        append!(out, ":{name}=\"{}\"", escape_attr(text));
    }
    out
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
