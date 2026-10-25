//! Native L3 consumes the same literal evaluator without reconstructing an AST.

use vize_atelier_core::{codegen::document::EmitDocument, literal_object_style};
use vize_carton::{Span, String};

use super::super::{AuthoredSpan, BindingKind, Content, Node};
use super::{Emitter, escaped, spans::argument_offset};

pub(super) struct StaticStyle {
    css: String,
    name: AuthoredSpan,
    value: AuthoredSpan,
}

pub(super) fn fold(node: &mut Node<'_>, source: &str) -> Option<StaticStyle> {
    let Content::Element { attributes, .. } = &node.content else {
        return None;
    };
    if attributes
        .iter()
        .any(|(name, ..)| name.eq_ignore_ascii_case("style"))
    {
        return None;
    }
    let mut selected = None;
    for (index, binding) in node.bindings.iter().enumerate() {
        if binding.kind == BindingKind::Spread || binding.dynamic_name.is_some() {
            return None;
        }
        if binding.kind == BindingKind::Prop && binding.name.eq_ignore_ascii_case("style") {
            if binding.name != "style"
                || selected.is_some()
                || !binding.modifiers.is_empty()
                || binding.merge.is_some()
            {
                return None;
            }
            selected = Some(index);
        }
    }
    let index = selected?;
    let binding = node.bindings.get(index)?;
    let expression = binding.value.js?.as_expression()?;
    if expression.raw != binding.value.text {
        return None;
    }
    let css = literal_object_style(expression.ast)?;
    let [name, value] = binding.spans;
    // Reuse the native lane's verified directive argument locator. The
    // generated CSS maps to the actual whole expression, not fabricated
    // per-property positions in the serialized text.
    let raw_name = source.get(name.0 as usize..name.1 as usize)?;
    let offset = u32::try_from(argument_offset(raw_name, "style")?).ok()?;
    let name = (name.0 + offset, name.0 + offset + 5);
    node.bindings.remove(index);
    Some(StaticStyle { css, name, value })
}

impl Emitter<'_, '_> {
    pub(super) fn write_static_style(&self, index: usize, template: &mut EmitDocument) {
        let Some(style) = self.static_styles.get(&index) else {
            return;
        };
        template.push_str(" ");
        if self.source.is_some() {
            template.push_linked("style", Span::new(style.name.0, style.name.1));
        } else {
            template.push_str("style");
        }
        template.push_str("=\"");
        if self.source.is_some() {
            template.push_linked(
                &escaped(&style.css),
                Span::new(style.value.0, style.value.1),
            );
        } else {
            template.push_str(&escaped(&style.css));
        }
        template.push_char('"');
    }

    pub(super) fn static_style_before(&self, index: usize, offset: u32) -> bool {
        self.static_styles
            .get(&index)
            .is_some_and(|style| style.name.0 < offset)
    }
}
