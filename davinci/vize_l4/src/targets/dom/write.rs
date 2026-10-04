//! DOM spelling primitives; decisions remain in the native L3 producer.

use vize_l0::{Span, ToCompactString};
use vize_l3::decision::dom::DomDependency;

use crate::runtime::{Helper, Vocabulary};
use crate::write::{LinkSink, Writer};

/// Helpers are checked once against the actual selected DOM vocabulary.
pub(super) struct Helpers {
    pub open_block: Helper,
    pub element: Helper,
    pub element_block: Helper,
    pub text: Helper,
    pub comment: Helper,
    pub display: Helper,
    pub fragment: Helper,
    pub class: Helper,
    pub style: Helper,
}

impl Helpers {
    pub fn checked(vocabulary: &Vocabulary) -> Option<Self> {
        Some(Self {
            open_block: vocabulary.helper("openBlock")?,
            element: vocabulary.helper("createElementVNode")?,
            element_block: vocabulary.helper("createElementBlock")?,
            text: vocabulary.helper("createTextVNode")?,
            comment: vocabulary.helper("createCommentVNode")?,
            display: vocabulary.helper("toDisplayString")?,
            fragment: vocabulary.helper("Fragment")?,
            class: vocabulary.helper("normalizeClass")?,
            style: vocabulary.helper("normalizeStyle")?,
        })
    }

    /// Encode the producer's ordered semantic demand with checked runtime ids.
    pub fn dependency(&self, dependency: DomDependency, vocabulary: &Vocabulary) -> Option<Helper> {
        Some(match dependency {
            DomDependency::DisplayValue => self.display,
            DomDependency::BlockBoundary => self.open_block,
            DomDependency::NativeElementBlock => self.element_block,
            DomDependency::NativeElementValue => self.element,
            DomDependency::TextValue => self.text,
            DomDependency::CommentValue | DomDependency::ConditionalPlaceholder => self.comment,
            DomDependency::FragmentValue => self.fragment,
            DomDependency::ClassNormalization => self.class,
            DomDependency::StyleNormalization => self.style,
            DomDependency::CollectionIteration => return vocabulary.helper("renderList"),
        })
    }
}

/// Record a leaf use immediately; structural uses close after their children.
pub(super) fn helper<L: LinkSink>(writer: &mut Writer<L>, vocabulary: &Vocabulary, helper: Helper) {
    writer.use_helper(helper);
    spell(writer, vocabulary, helper);
}

/// Structural uses are registered at their already encoded semantic closure.
pub(super) fn spell<L: LinkSink>(writer: &mut Writer<L>, vocabulary: &Vocabulary, helper: Helper) {
    writer.push("_");
    // Every index comes from `Helpers::checked` on this same vocabulary.
    if let Some(name) = vocabulary.name(helper) {
        writer.push(name);
    }
}

/// JSON string quoting is JavaScript string quoting for the selected runtime.
/// The complete literal retains its exact authored owner even after escaping.
pub(super) fn quoted<L: LinkSink>(writer: &mut Writer<L>, text: &str, span: Span) {
    // Serializing a string cannot fail: it contains no numeric or map payload.
    let encoded = serde_json::Value::from(text).to_compact_string();
    writer.push_linked(encoded.as_str(), span);
}

pub(super) fn property<L: LinkSink>(writer: &mut Writer<L>, name: &str, span: Span) {
    // Property spelling follows the stock ECMAScript IdentifierName grammar,
    // including valid Unicode. This is encoding, never expression admission.
    if oxc_syntax::identifier::is_identifier_name(name) {
        writer.push_linked(name, span);
    } else {
        quoted(writer, name, span);
    }
}

pub(super) fn number<L: LinkSink>(writer: &mut Writer<L>, value: u32) {
    writer.push(value.to_compact_string().as_str());
}
