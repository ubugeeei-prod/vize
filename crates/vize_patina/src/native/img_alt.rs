//! The first native syntax rule, retaining the existing `a11y/img-alt` code.

use vize_l0::{
    Span,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};
use vize_l1::{
    ElementClose,
    dialect::vue3::VueDirectives,
    markup::{ArgSyntax, DirectivePrefix, DirectiveSyntax, NativeElement},
};

use super::{IMG_ALT_RULE, NativeLintFinding, NativeLintRefusal, attribute};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
    let block = element.component().block();
    let surface = element.surface();
    attribute::token(block, &surface.open.lt_name)?;
    attribute::token(block, &surface.open.gt)?;
    if let Some(slash) = &surface.open.slash {
        attribute::token(block, slash)?;
    }
    let start = attribute::span(block, surface.open.lt_name.text)?.start;
    let end = match &surface.close {
        ElementClose::Present(close) => {
            attribute::token(block, &close.lt_slash_name)?;
            attribute::token(block, &close.gt)?;
            attribute::span(block, close.gt.text)?.end
        }
        ElementClose::NotExpected => attribute::span(block, surface.open.gt.text)?.end,
        ElementClose::Missing | ElementClose::Implicit => return Err(NativeLintRefusal::Hole),
    };
    let range = Span::new(start, end);
    if !block.contains_block_span(range) {
        return Err(NativeLintRefusal::SourceMismatch);
    }
    let mut has_alt = false;
    for original in element.attributes() {
        if !core::ptr::eq(original.component(), element.component())
            || !core::ptr::eq(original.element(), surface)
        {
            return Err(NativeLintRefusal::SourceMismatch);
        }
        let attribute = original.surface();
        let head_span = attribute::attribute(block, attribute)?;
        if surface.open.is_verbatim() {
            has_alt |= attribute.name.text.eq_ignore_ascii_case("alt");
            continue;
        }
        let head = VueDirectives
            .decompose(attribute.name.text, head_span.start)
            .map_err(|_| NativeLintRefusal::UnsupportedDirective { span: head_span })?;
        let Some(head) = head else {
            has_alt |= attribute.name.text.eq_ignore_ascii_case("alt");
            continue;
        };
        if matches!(head.arg, Some(ArgSyntax::Dynamic(_))) {
            return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
        }
        let binds = match head.prefix {
            DirectivePrefix::Bind | DirectivePrefix::Prop => true,
            DirectivePrefix::Full => {
                let name = attribute::project(block, head.name)?;
                match name {
                    "bind" => true,
                    "on" | "slot" | "if" | "else-if" | "else" | "for" | "show" | "html"
                    | "text" | "once" | "memo" | "model" | "cloak" | "pre" => false,
                    _ => return Err(NativeLintRefusal::UnsupportedDirective { span: head_span }),
                }
            }
            DirectivePrefix::On | DirectivePrefix::Slot => {
                if head.arg.is_none() {
                    return Err(NativeLintRefusal::UnsupportedDirective { span: head_span });
                }
                false
            }
        };
        if binds {
            let Some(ArgSyntax::Static(argument)) = head.arg else {
                return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
            };
            let name = attribute::project(block, argument)?;
            if name.is_empty() {
                return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
            }
            has_alt |= name.eq_ignore_ascii_case("alt");
        }
    }
    if !surface.tag().eq_ignore_ascii_case("img") || has_alt {
        return Ok(None);
    }
    let diagnostic = Diagnostic::new(
        Advisory::Warning,
        Stage::Surface,
        range,
        messages.lookup("a11y/img-alt.message").as_ref(),
    )
    .with_part(DiagnosticPart::new(
        PartKind::Help,
        range,
        messages.lookup("a11y/img-alt.help").as_ref(),
    ));
    Ok(Some(NativeLintFinding {
        rule_name: IMG_ALT_RULE,
        diagnostic,
    }))
}
