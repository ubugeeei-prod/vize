//! Checked original element ranges and header binding names, shared by syntax rules.

use vize_l0::Span;
use vize_l1::{
    ElementClose,
    dialect::vue3::VueDirectives,
    markup::{ArgSyntax, DirectivePrefix, DirectiveSyntax, NativeAttribute, NativeElement},
};

use super::{NativeLintRefusal, attribute};

pub(super) fn opening_range(element: &NativeElement<'_, '_>) -> Result<Span, NativeLintRefusal> {
    let block = element.component().block();
    let surface = element.surface();
    attribute::token(block, &surface.open.lt_name)?;
    attribute::token(block, &surface.open.gt)?;
    if let Some(slash) = &surface.open.slash {
        attribute::token(block, slash)?;
    }
    let start = attribute::span(block, surface.open.lt_name.text)?.start;
    let opening = Span::new(start, attribute::span(block, surface.open.gt.text)?.end);
    let end = match &surface.close {
        ElementClose::Present(close) => {
            attribute::token(block, &close.lt_slash_name)?;
            attribute::token(block, &close.gt)?;
            attribute::span(block, close.gt.text)?.end
        }
        ElementClose::NotExpected => opening.end,
        ElementClose::Missing | ElementClose::Implicit => return Err(NativeLintRefusal::Hole),
    };
    let authored = Span::new(start, end);
    if !block.contains_block_span(authored) {
        return Err(NativeLintRefusal::SourceMismatch);
    }
    Ok(opening)
}

pub(super) enum Binding<'a> {
    Static {
        name: &'a str,
        value: Option<&'a str>,
        range: Span,
    },
    Bind {
        name: &'a str,
        range: Span,
    },
    Other {
        range: Span,
    },
}

pub(super) fn binding<'a>(
    element: &NativeElement<'_, 'a>,
    original: &NativeAttribute<'_, 'a>,
) -> Result<Binding<'a>, NativeLintRefusal> {
    binding_with_modifiers::<false>(element, original)
}

/// New full-result consumers refuse parser errors from empty modifier segments.
/// Prior syntax rule admission keeps its original const-false behavior.
pub(super) fn strict_binding<'a>(
    element: &NativeElement<'_, 'a>,
    original: &NativeAttribute<'_, 'a>,
) -> Result<Binding<'a>, NativeLintRefusal> {
    binding_with_modifiers::<true>(element, original)
}

fn binding_with_modifiers<'a, const STRICT: bool>(
    element: &NativeElement<'_, 'a>,
    original: &NativeAttribute<'_, 'a>,
) -> Result<Binding<'a>, NativeLintRefusal> {
    if !core::ptr::eq(original.component(), element.component())
        || !core::ptr::eq(original.element(), element.surface())
    {
        return Err(NativeLintRefusal::SourceMismatch);
    }
    let block = element.component().block();
    let attribute = original.surface();
    let head_span = attribute::attribute(block, attribute)?;
    let range = attribute::full_span(block, attribute, head_span)?;
    let static_binding = || Binding::Static {
        name: attribute.name.text,
        value: attribute.value.as_ref().map(|value| value.content.text),
        range,
    };
    let verbatim = element.surface().open.is_verbatim();
    if verbatim
        && (!STRICT
            || element
                .lint_tag()
                .map_err(|reason| NativeLintRefusal::LintTag { reason })?
                .header_is_literal())
    {
        return Ok(static_binding());
    }
    let head = VueDirectives
        .decompose(attribute.name.text, head_span.start)
        .map_err(|_| NativeLintRefusal::UnsupportedDirective { span: head_span })?;
    let Some(head) = head else {
        return Ok(static_binding());
    };
    if STRICT && !head.modifiers.is_empty() {
        let modifiers = attribute::project(block, head.modifiers)?;
        if modifiers
            .strip_prefix('.')
            .is_none_or(|tail| tail.split('.').any(str::is_empty))
        {
            return Err(NativeLintRefusal::UnsupportedDirective { span: head_span });
        }
    }
    if verbatim {
        return Ok(static_binding());
    }
    if matches!(head.arg, Some(ArgSyntax::Dynamic(_))) {
        return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
    }
    let binds = match head.prefix {
        DirectivePrefix::Bind | DirectivePrefix::Prop => true,
        DirectivePrefix::Full => {
            let name = attribute::project(block, head.name)?;
            match name {
                "bind" => true,
                "on" | "slot" | "if" | "else-if" | "else" | "for" | "show" | "html" | "text"
                | "once" | "memo" | "model" | "cloak" | "pre" => false,
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
    if !binds {
        return Ok(Binding::Other { range });
    }
    let Some(ArgSyntax::Static(argument)) = head.arg else {
        return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
    };
    let name = attribute::project(block, argument)?;
    if name.is_empty() {
        return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
    }
    Ok(Binding::Bind { name, range })
}
