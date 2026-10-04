//! Opt-in configured native bare-template callbacks with whole-input refusal.
//! Only the genuine filename root callback is provided initially. Existing
//! selected header/child APIs and the default product route are unchanged.

use vize_l0::{SourceFrameError, Span, String, config::VueVersion};
use vize_l1::markup::{ComponentSourceError, NativeLintComponent};

use super::NativeLintRefusal;

mod admission;
mod component_name;
mod context;
mod driver;

pub use context::NativeTemplateLintContext;

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
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), NativeTemplateLintRefusal>;
}
