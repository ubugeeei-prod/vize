//! Actual Vue 3 directive patterns; no legacy capability or parser route.

use super::{Context, Directive, NativeHoleKind, Pattern, RegionBuilder};
use vize_l1::embed::prepare_attribute_value;
use vize_l1::markup::directive::{ArgSyntax, DirectiveName, DirectivePrefix};

pub(super) const PATTERNS: &[Pattern] = &[Pattern {
    accepts: is_bind,
    lower: bind,
}];

fn is_bind(source: &str, head: DirectiveName) -> bool {
    head.prefix == DirectivePrefix::Bind
        || head.prefix == DirectivePrefix::Full
            && source.get(head.name.start as usize..head.name.end as usize) == Some("bind")
}

fn bind<'a>(cx: &mut Context<'a>, region: &mut RegionBuilder<'_, 'a>, directive: Directive) {
    let head = directive.head;
    let expected = match head.prefix {
        DirectivePrefix::Bind => directive.name_span.start.checked_add(1),
        DirectivePrefix::Full => head.name.end.checked_add(1),
        _ => None,
    };
    let Some(ArgSyntax::Static(argument)) = head.arg else {
        cx.hole(region, NativeHoleKind::Directive, directive.span);
        return;
    };
    if head.modifiers.start != head.modifiers.end
        || Some(argument.start) != expected
        || argument.end != directive.name_span.end
        || argument.start == argument.end
    {
        cx.hole(region, NativeHoleKind::Directive, directive.span);
        return;
    }
    let Some(value) = directive.value else {
        // Same-name shorthand is a separate contract; do not synthesize a parse.
        cx.hole(region, NativeHoleKind::Directive, directive.span);
        return;
    };
    if directive.missing {
        cx.hole(region, NativeHoleKind::MissingMarkup, directive.span);
        return;
    }
    let file = cx.block.root_source();
    let Some(name) = file.get(argument.start as usize..argument.end as usize) else {
        cx.hole(region, NativeHoleKind::DirectiveSyntax, directive.span);
        return;
    };
    let source = match prepare_attribute_value(cx.allocator, file, value) {
        Ok(source) => source,
        Err(error) => {
            cx.hole(region, NativeHoleKind::Source(error), value);
            return;
        }
    };
    cx.expression(
        region,
        source,
        directive.span,
        ("native.bind", "ui.bind"),
        |_| Ok(()),
        |region, value| region.bind(name, argument, value, directive.span),
    );
}
