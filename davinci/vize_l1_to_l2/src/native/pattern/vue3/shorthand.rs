//! A Vue directive argument can be the original source of a JS identifier.
use super::{ConstructionFactory, Context, Directive, NativeHoleKind};
use vize_l0::Span;
use vize_l1::embed::EmbedSource;

pub(super) fn bind<'a, R: ConstructionFactory<'a>>(
    cx: &mut Context<'_, 'a>,
    region: &mut R,
    directive: Directive,
    argument: Span,
) {
    let Ok(source) = EmbedSource::authored(cx.block().root_source(), argument) else {
        cx.hole(region, NativeHoleKind::DirectiveSyntax, directive.span);
        return;
    };
    let name = source.text();
    // Vue camelizes a hyphenated argument. Only an unchanged identifier has
    // the original bytes and positions needed by the editor and File operand.
    if !oxc_syntax::identifier::is_identifier_name(name)
        || oxc_syntax::keyword::is_reserved_keyword(name)
    {
        cx.hole(region, NativeHoleKind::Directive, directive.span);
        return;
    }
    cx.expression(
        region,
        source,
        directive.span,
        ("native.bind.same-name", "ui.bind"),
        |syntax| match syntax.expression() {
            Some(oxc_ast::ast::Expression::Identifier(id)) if id.name.as_str() == name => Ok(()),
            _ => Err(NativeHoleKind::Directive),
        },
        |region, value| region.bind(name, argument, value, directive.span),
    );
}
