//! Original ordinary static attributes must exhaust before an Element body.

use super::super::{
    NativeVisibility, attribute_value::PreparedAttributes, conditional::ObservedConditional,
    for_head::ObservedFor, handler::ObservedHandler,
};
use crate::artifact::RegionBuilder;
use crate::file::region::FileRegion;
use crate::lang::js::file::native::NativeTemplateIssueKind as Kind;
use crate::op::Attribute;
use core::ops::DerefMut;
use vize_l0::Vec;
use vize_l1::{
    dialect::vue3::VueDirectives,
    markup::{
        NativeAttribute, NativeConditionKind, NativeElement, NativeTemplateComponent,
        directive::{DirectivePrefix, DirectiveSyntax},
    },
};

pub(in crate::file::region::native) struct Header<'a> {
    pub(in crate::file::region::native) attributes: Vec<'a, Attribute<'a>>,
    pub(in crate::file::region::native) values: PreparedAttributes<'a>,
    pub(in crate::file::region::native) handlers: alloc::vec::Vec<ObservedHandler<'a>>,
    pub(in crate::file::region::native) for_head: Option<ObservedFor>,
    pub(in crate::file::region::native) conditional: Option<ObservedConditional>,
}

pub(super) fn construct<'a: 'b, 'b, R>(
    original: &NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<Header<'a>, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    construct_with_policy(original, selected, region, false)
}

pub(in crate::file::region::native) fn construct_for_pair<'a: 'b, 'b, R>(
    original: &NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<Header<'a>, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    construct_with_policy(original, selected, region, true)
}

fn construct_with_policy<'a: 'b, 'b, R>(
    original: &NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
    paired: bool,
) -> Result<Header<'a>, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let value_start = region.facts.native_attribute_values.len();
    let mut attributes = Vec::new_in(&original.component().allocator());
    let mut handlers: alloc::vec::Vec<ObservedHandler<'a>> = alloc::vec::Vec::new();
    let mut for_head = None;
    let mut conditional = None;
    // One actual full header iterator. Only its normal end permits the Element
    // factory and original callback; pending owners remain parked on refusal.
    for (ordinal, attribute) in original.attributes().enumerate() {
        if !core::ptr::eq(selected.component(), original.component())
            || !core::ptr::eq(attribute.component(), original.component())
            || !core::ptr::eq(attribute.element(), original.surface())
            || attribute.ordinal() != ordinal
            || !original
                .surface()
                .open
                .attrs
                .get(ordinal)
                .is_some_and(|surface| core::ptr::eq(surface, attribute.surface()))
        {
            return Err(Kind::InvalidEvent);
        }
        let block = original.component().block();
        let name_span = block
            .span_of(attribute.surface().name.text)
            .ok_or(Kind::InvalidEvent)?;
        match VueDirectives.decompose(attribute.surface().name.text, name_span.start) {
            Ok(None) => {
                let attribute =
                    if attribute.surface().eq.is_some() || attribute.surface().value.is_some() {
                        region.observe_attribute_value(selected, attribute, attributes.len())?
                    } else {
                        ordinary(attribute)?
                    };
                if attributes
                    .iter()
                    .any(|previous: &Attribute<'_>| previous.name == attribute.name)
                {
                    return Err(Kind::UnsupportedChild);
                }
                attributes.push(attribute);
            }
            Ok(Some(directive))
                if directive.prefix == DirectivePrefix::On
                    || (directive.prefix == DirectivePrefix::Full
                        && directive.name.slice(block.root_source()) == "on") =>
            {
                let handler = region.observe_handler(selected, attribute)?;
                if handlers
                    .iter()
                    .any(|previous| previous.name == handler.name)
                {
                    return Err(Kind::UnsupportedChild);
                }
                handlers.push(handler);
            }
            Ok(Some(directive))
                if directive.prefix == DirectivePrefix::Full
                    && directive.name.slice(block.root_source()) == "for" =>
            {
                let observed = region.observe_for(selected, attribute)?;
                if for_head.is_some() {
                    return Err(Kind::UnsupportedChild);
                }
                for_head = Some(observed);
            }
            Ok(Some(directive))
                if directive.prefix == DirectivePrefix::Full
                    && directive.name.slice(block.root_source()) == "if" =>
            {
                let head = selected
                    .observe_attribute_head(attribute.reborrow())
                    .map_err(|_| Kind::UnsupportedChild)?;
                if head.condition_kind() != Some(NativeConditionKind::If) {
                    return Err(Kind::UnsupportedChild);
                }
                let observed = region.observe_conditional(selected, &head)?;
                if conditional.is_some() {
                    return Err(Kind::UnsupportedChild);
                }
                conditional = Some(observed);
            }
            Ok(Some(_)) | Err(_) => return Err(Kind::UnsupportedChild),
        }
    }
    let values = region.prepare_attribute_storage(value_start, attributes.as_slice())?;
    if !paired
        && conditional.is_some()
        && (!attributes.is_empty() || !handlers.is_empty() || for_head.is_some())
    {
        return Err(Kind::UnsupportedChild);
    }
    Ok(Header {
        values,
        attributes,
        handlers,
        for_head,
        conditional,
    })
}

pub(in crate::file::region::native) struct ReadyHeader<'a> {
    pub(in crate::file::region::native) attributes: Vec<'a, Attribute<'a>>,
    pub(in crate::file::region::native) values: PreparedAttributes<'a>,
    pub(in crate::file::region::native) handlers: alloc::vec::Vec<ObservedHandler<'a>>,
}

impl<'a> Header<'a> {
    pub(in crate::file::region::native) fn resolve_handlers<'b, R>(
        self,
        region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
    ) -> Result<ReadyHeader<'a>, Kind>
    where
        'a: 'b,
        R: DerefMut<Target = RegionBuilder<'b, 'a>>,
    {
        if self.for_head.is_some() || self.conditional.is_some() {
            return Err(Kind::InvalidEvent);
        }
        for handler in &self.handlers {
            let _ = region.resolve_handler(handler)?;
        }
        Ok(ReadyHeader {
            values: self.values,
            attributes: self.attributes,
            handlers: self.handlers,
        })
    }
}

fn ordinary<'a>(original: NativeAttribute<'_, 'a>) -> Result<Attribute<'a>, Kind> {
    let surface = original.surface();
    let block = original.component().block();
    let name = surface.name.text;
    let name_span = block.span_of(name).ok_or(Kind::InvalidEvent)?;
    if surface.name.is_missing()
        || name.is_empty()
        || matches!(name, "style" | "key" | "ref" | "is")
    {
        return Err(Kind::UnsupportedChild);
    }
    if surface.eq.is_some() || surface.value.is_some() {
        return Err(Kind::UnsupportedChild);
    }
    Ok(Attribute {
        name,
        value: None,
        span: name_span,
    })
}
