//! Original first-option header observations, never decoded text/value facts.

use super::{NativeTemplateAttributeKind, NativeTemplateElement, NativeTemplateLintRefusal};
use vize_l0::Span;
use vize_l1::markup::{Element, NativeComponent};

/// Raw authored attribute syntax. `Absent` does not establish a text value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateOptionValue<'a> {
    Absent,
    Static { value: Option<&'a str>, range: Span },
    Bound { range: Span },
}

/// A header candidate becomes observable only inside the completed select.
/// The one walk creates only the first direct option's constant-size record.
#[derive(Clone, Copy)]
pub(in crate::native) struct NativeFirstOptionData<'o, 'a> {
    pub(super) component: &'o NativeComponent<'a>,
    pub(super) select: &'o Element<'a>,
    pub(super) opening: Span,
    pub(super) value: NativeTemplateOptionValue<'a>,
}

impl<'o, 'a> NativeFirstOptionData<'o, 'a> {
    pub(in crate::native) fn from_checked(
        view: &NativeTemplateElement<'o, 'a>,
        opening: Span,
    ) -> Result<Self, NativeTemplateLintRefusal> {
        let original = view.original();
        let select = original
            .parent_element()
            .ok_or(NativeTemplateLintRefusal::SourceMismatch)?;
        if original.surface().tag() != "option" || select.tag() != "select" {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        let mut value = NativeTemplateOptionValue::Absent;
        for attribute in view.attributes() {
            match attribute.kind() {
                NativeTemplateAttributeKind::Static {
                    name: "value",
                    value: raw,
                } if matches!(value, NativeTemplateOptionValue::Absent) => {
                    value = NativeTemplateOptionValue::Static {
                        value: raw,
                        range: attribute.range(),
                    };
                }
                NativeTemplateAttributeKind::Bind { name: "value", .. }
                | NativeTemplateAttributeKind::DynamicBind { .. } => {
                    value = NativeTemplateOptionValue::Bound {
                        range: attribute.range(),
                    };
                }
                _ => {}
            }
        }
        Ok(Self {
            component: original.component(),
            select,
            opening,
            value,
        })
    }
}

/// A borrowed first-header token issued to one actual frozen callback.
/// Its component/select and current stored-first identity are checked again
/// before observing a value or issuing an opening-only diagnostic.
///
/// ```compile_fail
/// use vize_patina::native::template::NativeTemplateFirstOption;
/// let proof = NativeTemplateFirstOption {};
/// ```
pub struct NativeTemplateFirstOption<'s, 'o, 'a> {
    pub(super) data: &'s NativeFirstOptionData<'o, 'a>,
    pub(super) issuer: (usize, usize),
}
