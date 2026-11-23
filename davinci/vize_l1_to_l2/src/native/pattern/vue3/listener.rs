//! Vue method references are expressions in the enclosing template scope.
use super::{ConstructionFactory, Context, Directive, NativeHoleKind};
use vize_l1::embed::prepare_attribute_value;
use vize_l1::framework::vue::directive::{ArgSyntax, DirectiveName, DirectivePrefix};

pub(in crate::native::pattern) fn accepts(source: &str, head: DirectiveName) -> bool {
    head.prefix == DirectivePrefix::On
        || head.prefix == DirectivePrefix::Full && head.name.slice(source) == "on"
}

pub(in crate::native::pattern) fn lower<'a, R: ConstructionFactory<'a>>(
    cx: &mut Context<'_, 'a>,
    region: &mut R,
    directive: Directive,
) {
    let span = directive.span;
    let head = directive.head;
    let Some(ArgSyntax::Static(name)) = head.arg else {
        cx.hole(region, NativeHoleKind::Directive, span);
        return;
    };
    let prefix_end = if head.prefix == DirectivePrefix::On {
        directive.name_span.start
    } else {
        head.name.end
    };
    if prefix_end.checked_add(1) != Some(name.start)
        || name.start == name.end
        || name.end != head.modifiers.start
        || head.modifiers.end != directive.name_span.end
        || directive.missing
    {
        cx.hole(region, NativeHoleKind::DirectiveSyntax, span);
        return;
    }
    let source = cx.block().root_source();
    let mut modifiers = vize_l0::Vec::new_in(&cx.allocator);
    if let Some(rest) = head.modifiers.slice(source).strip_prefix('.') {
        for modifier in rest.split('.') {
            if modifier.is_empty() {
                cx.hole(region, NativeHoleKind::DirectiveSyntax, span);
                return;
            }
            modifiers.push(modifier);
        }
    }
    let Some(value) = directive.value else {
        if span != directive.name_span {
            cx.hole(region, NativeHoleKind::MissingMarkup, span);
            return;
        }
        let result = region.event_reference(name, modifiers, None, span);
        cx.produced(region, result, "native.listener", span, "ui.on");
        return;
    };
    let value = match prepare_attribute_value(cx.allocator, source, value) {
        Ok(value) if value.decode_map().is_none() => value,
        Ok(_) => {
            cx.hole(region, NativeHoleKind::Directive, span);
            return;
        }
        Err(error) => {
            cx.hole(region, NativeHoleKind::Source(error), span);
            return;
        }
    };
    cx.expression(
        region,
        value,
        span,
        ("native.listener.reference", "ui.on"),
        |syntax| match syntax.expression() {
            Some(oxc_ast::ast::Expression::Identifier(id))
                if id.name.as_str() == syntax.source().text()
                    && !matches!(id.name.as_str(), "$event" | "arguments") =>
            {
                Ok(())
            }
            _ => Err(NativeHoleKind::Directive),
        },
        |region, js| region.event_reference(name, modifiers, Some(js), span),
    );
}
