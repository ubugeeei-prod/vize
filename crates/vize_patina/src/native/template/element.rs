//! Sealed temporary observations from the single original admitted header.

use vize_l0::{SmallVec, Span};
use vize_l1::markup::{DirectiveName, NativeAttribute, NativeElement};

use super::super::header::{AttributeBinding, CheckedAttribute};
use super::NativeTemplateAttributeProfile;

/// Source syntax only. Dynamic expressions and all values remain opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateAttributeKind<'a> {
    Static {
        name: &'a str,
        value: Option<&'a str>,
    },
    Bind {
        name: &'a str,
        argument_range: Span,
    },
    DynamicBind {
        argument_range: Span,
    },
}

/// A genuine checked attribute; callers cannot construct raw-parts proofs.
/// Its actual component, element, ordinal and authored range remain owned.
///
/// ```compile_fail
/// use vize_patina::native::template::NativeTemplateAttribute;
/// let proof = NativeTemplateAttribute {};
/// ```
/// A callback's retained proof cannot escape its actual short owner borrow:
/// ```compile_fail
/// use vize_patina::native::template::{NativeTemplateAttribute, NativeTemplateElement};
/// fn escape<'a>(view: &NativeTemplateElement<'_, 'a>)
///     -> &'static NativeTemplateAttribute<'static, 'a> {
///     &view.attributes()[0]
/// }
/// ```
pub struct NativeTemplateAttribute<'o, 'a> {
    checked: CheckedAttribute<'o, 'a>,
    kind: NativeTemplateAttributeKind<'a>,
}

impl<'o, 'a> NativeTemplateAttribute<'o, 'a> {
    pub(super) fn from_checked(
        checked: CheckedAttribute<'o, 'a>,
        binding: AttributeBinding<'a>,
        profile: NativeTemplateAttributeProfile,
    ) -> Result<Self, Span> {
        let kind = match binding {
            AttributeBinding::Static { name, value }
                if name.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.')
                }) =>
            {
                NativeTemplateAttributeKind::Static { name, value }
            }
            AttributeBinding::Bind { name, argument }
                if profile == NativeTemplateAttributeProfile::Bindings =>
            {
                NativeTemplateAttributeKind::Bind {
                    name,
                    argument_range: argument,
                }
            }
            AttributeBinding::DynamicBind { argument }
                if profile == NativeTemplateAttributeProfile::Bindings =>
            {
                NativeTemplateAttributeKind::DynamicBind {
                    argument_range: argument,
                }
            }
            _ => return Err(checked.range()),
        };
        Ok(Self { checked, kind })
    }

    #[must_use]
    pub fn original(&self) -> &NativeAttribute<'o, 'a> {
        self.checked.original()
    }
    #[must_use]
    pub fn range(&self) -> Span {
        self.checked.range()
    }
    /// The exact once-computed original name/prefix/argument/modifier record.
    #[must_use]
    pub fn directive(&self) -> Option<DirectiveName> {
        self.checked.directive()
    }
    #[must_use]
    pub fn kind(&self) -> NativeTemplateAttributeKind<'a> {
        self.kind
    }
    #[must_use]
    pub fn value(&self) -> Option<&'a str> {
        self.original()
            .surface()
            .value
            .as_ref()
            .map(|value| value.content.text)
    }
}

/// One actual element and its ordered checked header, valid only during dispatch.
/// Reading retained observations adds no original source/header walk.
/// The temporary SmallVec may spill; the driver drops it before child recursion.
///
/// ```compile_fail
/// use vize_patina::native::template::NativeTemplateElement;
/// let proof = NativeTemplateElement {};
/// ```
pub struct NativeTemplateElement<'o, 'a> {
    original: NativeElement<'o, 'a>,
    attributes: SmallVec<[NativeTemplateAttribute<'o, 'a>; 8]>,
}

impl<'o, 'a> NativeTemplateElement<'o, 'a> {
    pub(super) fn new(
        original: NativeElement<'o, 'a>,
        attributes: SmallVec<[NativeTemplateAttribute<'o, 'a>; 8]>,
    ) -> Self {
        Self {
            original,
            attributes,
        }
    }

    #[must_use]
    pub fn original(&self) -> &NativeElement<'o, 'a> {
        &self.original
    }
    #[must_use]
    pub fn attributes(&self) -> &[NativeTemplateAttribute<'o, 'a>] {
        &self.attributes
    }
}
