//! Opt-in configured native bare-template callbacks with whole-input refusal.
//! Generic checked element dispatch requires an actual registered capability.
//! The filename root and two focus-attribute builtins offer actual capabilities.
//! Existing selected header/child APIs and the default product route are unchanged.

use vize_l0::{SourceFrameError, Span, String, config::VueVersion};
use vize_l1::markup::{ComponentSourceError, NativeLintComponent};

use super::NativeLintRefusal;

mod component_name;
mod context;
mod driver;
mod element;
mod option;
mod owner;

pub use context::{
    NativeTemplateEventModifier, NativeTemplateEventModifiers, NativeTemplateLintContext,
};
pub use element::{NativeTemplateAttribute, NativeTemplateAttributeKind, NativeTemplateElement};
pub(in crate::native) use option::NativeFirstOptionData;
pub use option::{NativeTemplateFirstOption, NativeTemplateOptionValue};
pub use owner::{
    NativeTemplateClosedSelect, NativeTemplateClosedTextarea, NativeTemplateOwnerProfile,
};

/// Authentic enabled callbacks must all admit the wider source profile.
/// Empty callback sets and callbacks that do not opt in retain StaticOnly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NativeTemplateAttributeProfile {
    #[default]
    /// Admit only plain static attributes with opaque literal values.
    StaticOnly,
    /// Fixed and nonempty lexically completed Bind/Prop/Full bind arguments.
    /// Original component lexer errors still refuse before any callback;
    /// this says nothing about JavaScript validity or runtime argument names.
    Bindings,
    /// Static original event arguments and their exact checked modifier text.
    /// This does not grant bindings, dynamic arguments or expression validity.
    Events,
}

impl NativeTemplateAttributeProfile {
    pub(super) fn intersect(self, other: Self) -> Self {
        if self == other {
            self
        } else {
            Self::StaticOnly
        }
    }
}

/// Original literal text is admitted only when every configured callback in
/// the template-only SFC host offers that capability. This does not establish
/// interpolation, comment suppression, text decoding or runtime semantics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NativeTemplateTextProfile {
    #[default]
    /// Admit only whitespace text children.
    WhitespaceOnly,
    /// Admit closed literal text while keeping entities and runtime meaning opaque.
    LiteralText,
}

impl NativeTemplateTextProfile {
    pub(in crate::native) fn intersect(self, other: Self) -> Self {
        if self == Self::LiteralText && other == Self::LiteralText {
            Self::LiteralText
        } else {
            Self::WhitespaceOnly
        }
    }
}

/// No partial findings or silent omission can turn an unsupported input clean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeTemplateLintRefusal {
    /// The supplied source cannot establish a valid source frame.
    Source(SourceFrameError),
    /// The original template could not be parsed.
    Parse(ComponentSourceError),
    /// The configured Vue version is outside the native template contract.
    UnsupportedVueVersion {
        /// Vue version requested by the linter configuration.
        requested: VueVersion,
    },
    /// The configured Vapor mode is outside the native template contract.
    UnsupportedVaporMode {
        /// Vapor mode requested by the linter configuration.
        requested: bool,
    },
    /// An enabled configured rule does not offer a native template implementation.
    UnprovidedRule {
        /// Stable rule code that lacks the required capability.
        rule: String,
    },
    /// The original parser recovered from an error at the component-relative offset.
    Recovered {
        /// Component-relative byte offset of the first recovery.
        offset: u32,
    },
    /// The component uses syntax outside the native template contract.
    UnsupportedComponent,
    /// One retained owner or source range does not match the original template.
    SourceMismatch,
    /// An element header failed native syntax admission.
    Header(NativeLintRefusal),
    /// A rule requested an element-owner grammar it did not declare.
    UnsupportedContext {
        /// Original context range that could not be admitted.
        span: Span,
    },
    /// An attribute lies outside the intersection of declared callback profiles.
    UnsupportedAttribute {
        /// Original unsupported attribute range.
        span: Span,
    },
    /// Text lies outside the intersection of declared callback profiles.
    UnsupportedText {
        /// Original unsupported text range.
        span: Span,
    },
    /// A comment requires suppression policy unavailable to this capability.
    Comment {
        /// Original comment range.
        span: Span,
    },
    /// An interpolation is outside the admitted literal-text grammar.
    Interpolation {
        /// Original interpolation range.
        span: Span,
    },
    /// The original child is not valid for the admitted bounded owner grammar.
    UnexpectedChild {
        /// Original unexpected child range.
        span: Span,
    },
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
    /// Declares the strongest original-attribute grammar accepted by the rule.
    fn attribute_profile(&self) -> NativeTemplateAttributeProfile {
        NativeTemplateAttributeProfile::StaticOnly
    }

    /// Declares the strongest bounded owner grammar accepted by the rule.
    fn owner_profile(&self) -> NativeTemplateOwnerProfile {
        NativeTemplateOwnerProfile::Strict
    }

    /// Runs once against the authenticated whole bare-template owner.
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), NativeTemplateLintRefusal>;

    /// Runs for each admitted original element during the existing traversal.
    fn run_on_element<'a>(
        &self,
        _context: &mut NativeTemplateLintContext<'_, 'a>,
        _element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        Ok(())
    }
}
