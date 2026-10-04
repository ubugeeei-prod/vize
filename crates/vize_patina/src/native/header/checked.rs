//! Original checked binding projection, extracted without policy changes.

use vize_l1::{
    dialect::vue3::VueDirectives,
    markup::{ArgSyntax, DirectivePrefix, DirectiveSyntax, NativeAttribute, NativeElement},
};

use super::super::{NativeLintRefusal, attribute};
use super::Binding;

pub(super) fn binding_with_modifiers<'a, const STRICT: bool>(
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
    let (binds, full_directive_name) = match head.prefix {
        DirectivePrefix::Bind | DirectivePrefix::Prop => (true, None),
        DirectivePrefix::Full => {
            let name = attribute::project(block, head.name)?;
            let binds = match name {
                "bind" => true,
                "on" | "slot" | "if" | "else-if" | "else" | "for" | "show" | "html" | "text"
                | "once" | "memo" | "model" | "cloak" | "pre" => false,
                _ => return Err(NativeLintRefusal::UnsupportedDirective { span: head_span }),
            };
            (binds, Some(name))
        }
        DirectivePrefix::On | DirectivePrefix::Slot => {
            if head.arg.is_none() {
                return Err(NativeLintRefusal::UnsupportedDirective { span: head_span });
            }
            (false, None)
        }
    };
    if !binds {
        return Ok(Binding::Other {
            full_directive_name,
            range,
        });
    }
    let Some(ArgSyntax::Static(argument)) = head.arg else {
        return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
    };
    let name = attribute::project(block, argument)?;
    if name.is_empty() {
        return Err(NativeLintRefusal::UnresolvedBinding { span: head_span });
    }
    Ok(Binding::Bind {
        name,
        argument_range: argument,
        range,
    })
}
