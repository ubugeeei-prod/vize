//! One authentic attribute check and retained original directive decomposition.

use vize_l0::Span;
use vize_l1::{
    dialect::vue3::VueDirectives,
    markup::{
        ArgSyntax, DirectiveName, DirectivePrefix, DirectiveSyntax, NativeAttribute, NativeElement,
    },
};

use super::super::{NativeLintRefusal, attribute};
use super::Binding;

pub(in crate::native) enum AttributeBinding<'a> {
    Static {
        name: &'a str,
        value: Option<&'a str>,
    },
    Bind {
        name: &'a str,
        argument: Span,
    },
    DynamicBind {
        argument: Span,
    },
    Other {
        full_directive_name: Option<&'a str>,
    },
}

/// A checked original projection. Fields and construction remain private.
/// The wider bare profile still requires its normally owned clean component.
pub(in crate::native) struct CheckedAttribute<'o, 'a> {
    original: NativeAttribute<'o, 'a>,
    head: Option<DirectiveName>,
    head_span: Span,
    range: Span,
    literal: bool,
}

impl<'o, 'a> CheckedAttribute<'o, 'a> {
    pub(in crate::native) fn check<const STRICT: bool>(
        element: &NativeElement<'_, 'a>,
        original: NativeAttribute<'o, 'a>,
    ) -> Result<Self, NativeLintRefusal> {
        if !core::ptr::eq(original.component(), element.component())
            || !core::ptr::eq(original.element(), element.surface())
        {
            return Err(NativeLintRefusal::SourceMismatch);
        }
        let block = element.component().block();
        let attribute = original.surface();
        let head_span = attribute::attribute(block, attribute)?;
        let range = attribute::full_span(block, attribute, head_span)?;
        let verbatim = element.surface().open.is_verbatim();
        if verbatim
            && (!STRICT
                || element
                    .lint_tag()
                    .map_err(|reason| NativeLintRefusal::LintTag { reason })?
                    .header_is_literal())
        {
            return Ok(Self {
                original,
                head: None,
                head_span,
                range,
                literal: true,
            });
        }
        let head = VueDirectives
            .decompose(attribute.name.text, head_span.start)
            .map_err(|_| NativeLintRefusal::UnsupportedDirective { span: head_span })?;
        if let Some(head) = &head
            && STRICT
            && !head.modifiers.is_empty()
        {
            let modifiers = attribute::project(block, head.modifiers)?;
            if modifiers
                .strip_prefix('.')
                .is_none_or(|tail| tail.split('.').any(str::is_empty))
            {
                return Err(NativeLintRefusal::UnsupportedDirective { span: head_span });
            }
        }
        Ok(Self {
            original,
            head,
            head_span,
            range,
            literal: verbatim,
        })
    }

    pub(in crate::native) fn original(&self) -> &NativeAttribute<'o, 'a> {
        &self.original
    }
    pub(in crate::native) fn range(&self) -> Span {
        self.range
    }
    pub(in crate::native) fn directive(&self) -> Option<DirectiveName> {
        self.head
    }

    pub(in crate::native) fn narrow(
        &self,
        binding: AttributeBinding<'a>,
    ) -> Result<Binding<'a>, NativeLintRefusal> {
        let range = self.range;
        Ok(match binding {
            AttributeBinding::Static { name, value } => Binding::Static { name, value, range },
            AttributeBinding::Bind { name, argument } => Binding::Bind {
                name,
                argument_range: argument,
                range,
            },
            AttributeBinding::Other {
                full_directive_name,
            } => Binding::Other {
                full_directive_name,
                range,
            },
            AttributeBinding::DynamicBind { .. } => {
                return Err(NativeLintRefusal::UnresolvedBinding {
                    span: self.head_span,
                });
            }
        })
    }

    pub(in crate::native) fn binding<const WIDE: bool>(
        &self,
    ) -> Result<AttributeBinding<'a>, NativeLintRefusal> {
        let attribute = self.original.surface();
        let block = self.original.component().block();
        let Some(head) = self.head.filter(|_| !self.literal) else {
            return Ok(AttributeBinding::Static {
                name: attribute.name.text,
                value: attribute.value.as_ref().map(|value| value.content.text),
            });
        };
        // Preserve the original selected policy's refusal order for every head.
        if !WIDE && matches!(head.arg, Some(ArgSyntax::Dynamic(_))) {
            return Err(NativeLintRefusal::UnresolvedBinding {
                span: self.head_span,
            });
        }
        let (binds, full_directive_name) = match head.prefix {
            DirectivePrefix::Bind | DirectivePrefix::Prop => (true, None),
            DirectivePrefix::Full => {
                let name = attribute::project(block, head.name)?;
                let binds = match name {
                    "bind" => true,
                    "on" | "slot" | "if" | "else-if" | "else" | "for" | "show" | "html"
                    | "text" | "once" | "memo" | "model" | "cloak" | "pre" => false,
                    _ => {
                        return Err(NativeLintRefusal::UnsupportedDirective {
                            span: self.head_span,
                        });
                    }
                };
                (binds, Some(name))
            }
            DirectivePrefix::On | DirectivePrefix::Slot => {
                if head.arg.is_none() {
                    return Err(NativeLintRefusal::UnsupportedDirective {
                        span: self.head_span,
                    });
                }
                (false, None)
            }
        };
        if !binds {
            return Ok(AttributeBinding::Other {
                full_directive_name,
            });
        }
        let Some(argument) = head.arg else {
            return Err(NativeLintRefusal::UnresolvedBinding {
                span: self.head_span,
            });
        };
        if WIDE {
            self.binding_geometry(head, argument)?;
        }
        let range = match argument {
            ArgSyntax::Static(range) | ArgSyntax::Dynamic(range) => range,
        };
        let name = attribute::project(block, range)?;
        if name.is_empty() {
            return Err(NativeLintRefusal::UnresolvedBinding {
                span: self.head_span,
            });
        }
        Ok(match argument {
            ArgSyntax::Static(argument) => AttributeBinding::Bind { name, argument },
            // The wider caller has already refused every original component
            // lexer error. This is HTML-head completion, never JS validity.
            ArgSyntax::Dynamic(argument) => AttributeBinding::DynamicBind { argument },
        })
    }

    /// The existing decomposition may overwrite an earlier dynamic argument
    /// with a closed suffix or a second argument. A clean lexer alone cannot
    /// grant the wider view that category. Join only retained absolute spans
    /// and their constant boundary bytes; no head scan or decomposition repeats.
    fn binding_geometry(
        &self,
        head: DirectiveName,
        argument: ArgSyntax,
    ) -> Result<(), NativeLintRefusal> {
        let refusal = || NativeLintRefusal::UnresolvedBinding {
            span: self.head_span,
        };
        let block = self.original.component().block();
        let origin = match head.prefix {
            DirectivePrefix::Bind | DirectivePrefix::Prop => self.head_span.start + 1,
            DirectivePrefix::Full if head.name.end < self.head_span.end => {
                if attribute::project(block, Span::new(head.name.end, head.name.end + 1))? != ":" {
                    return Err(refusal());
                }
                head.name.end + 1
            }
            _ => return Err(refusal()),
        };
        let valid = match argument {
            ArgSyntax::Static(range) => range.start == origin && range.end == head.modifiers.start,
            ArgSyntax::Dynamic(range) => {
                range.start == origin + 1
                    && range.end < self.head_span.end
                    && range.end + 1 == head.modifiers.start
                    && attribute::project(block, Span::new(origin, origin + 1))? == "["
                    && attribute::project(block, Span::new(range.end, range.end + 1))? == "]"
            }
        };
        if !valid {
            return Err(refusal());
        }
        Ok(())
    }
}

pub(super) fn binding_with_modifiers<'a, const STRICT: bool>(
    element: &NativeElement<'_, 'a>,
    original: &NativeAttribute<'_, 'a>,
) -> Result<Binding<'a>, NativeLintRefusal> {
    let checked = CheckedAttribute::check::<STRICT>(element, original.reborrow())?;
    checked.narrow(checked.binding::<false>()?)
}
