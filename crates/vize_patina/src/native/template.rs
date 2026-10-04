//! Opt-in configured native bare-template callbacks with whole-input refusal.
//! Generic checked element dispatch requires an actual registered capability.
//! The filename root callback remains the only provided builtin here. Existing
//! selected header/child APIs and the default product route are unchanged.

use vize_l0::{SourceFrameError, Span, String, config::VueVersion};
use vize_l1::markup::{ComponentSourceError, NativeLintComponent};

use super::NativeLintRefusal;

mod admission;
mod component_name;
mod context;
mod driver;
mod element;

pub use context::NativeTemplateLintContext;
pub use element::{NativeTemplateAttribute, NativeTemplateAttributeKind, NativeTemplateElement};

/// Authentic enabled callbacks must all admit the wider source profile.
/// Empty callback sets and callbacks that do not opt in retain StaticOnly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NativeTemplateAttributeProfile {
    #[default]
    StaticOnly,
    /// Fixed and nonempty lexically completed Bind/Prop/Full bind arguments.
    /// Original component lexer errors still refuse before any callback;
    /// this says nothing about JavaScript validity or runtime argument names.
    Bindings,
}

impl NativeTemplateAttributeProfile {
    pub(super) fn intersect(self, other: Self) -> Self {
        if self == Self::Bindings && other == Self::Bindings {
            Self::Bindings
        } else {
            Self::StaticOnly
        }
    }
}

/// No partial findings or silent omission can turn an unsupported input clean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeTemplateLintRefusal {
    Source(SourceFrameError),
    Parse(ComponentSourceError),
    UnsupportedVueVersion { requested: VueVersion },
    UnsupportedVaporMode { requested: bool },
    UnprovidedRule { rule: String },
    Recovered { offset: u32 },
    UnsupportedComponent,
    SourceMismatch,
    Header(NativeLintRefusal),
    UnsupportedContext { span: Span },
    UnsupportedAttribute { span: Span },
    UnsupportedText { span: Span },
    Comment { span: Span },
    Interpolation { span: Span },
    UnexpectedChild { span: Span },
}

impl From<NativeLintRefusal> for NativeTemplateLintRefusal {
    fn from(error: NativeLintRefusal) -> Self {
        Self::Header(error)
    }
}

/// Authentic callback capability of an actual registered rule instance.
/// The root is the original bare lint owner, never a synthetic Relief root or
/// Descriptor-selected wrapper. Results remain private until the driver's
/// single original traversal admits the entire supported input profile.
pub trait NativeTemplateRule: Send + Sync {
    fn attribute_profile(&self) -> NativeTemplateAttributeProfile {
        NativeTemplateAttributeProfile::StaticOnly
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), NativeTemplateLintRefusal>;

    fn run_on_element<'a>(
        &self,
        _context: &mut NativeTemplateLintContext<'_, 'a>,
        _element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        Ok(())
    }
}
