use vize_l1::container::vue::{Vue2DescriptorObservation, Vue2DescriptorRefusal, Vue2TemplateView};

use super::super::{Doc, print};
use super::{NativeVue2SfcOptions, NativeVue2SfcRefusal};
use crate::FormatResult;

pub(super) enum Outcome<'a> {
    Document(Doc<'a>),
    Refused(NativeVue2SfcRefusal),
}

/// One original normal descriptor owns every historical CST/stock observation.
/// The private document never escapes its live owner's borrow.
/// The initial family covers static div/span/p/section/a/br/input headers and
/// genuine admitted original interpolations; all other tags/directives refuse.
/// Whole historical grammar, native classic runtime and defaults stay separate.
///
/// ```
/// use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in};
/// use vize_l0::Allocator;
/// let arena = Allocator::default();
/// let owner = observe_native_vue2_sfc_in(&arena,
///     "<template><div>{{a+b|upper}}</div></template>", NativeVue2SfcOptions::default());
/// assert_eq!(owner.format().unwrap().code,
///     "<template><div>{{a + b|upper}}</div></template>");
/// assert!(core::ptr::eq(owner.selected().unwrap().observation(), owner.descriptor()));
/// ```
///
/// ```compile_fail,E0616
/// use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in};
/// use vize_l0::Allocator;
/// let arena = Allocator::default();
/// let mut owner = observe_native_vue2_sfc_in(&arena, "<template/>", NativeVue2SfcOptions::default());
/// owner.options = NativeVue2SfcOptions::default();
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::NativeVue2SfcObservation;
/// fn copy(owner: NativeVue2SfcObservation<'_>) { let _ = owner.clone(); }
/// ```
/// ```compile_fail,E0515
/// use vize_glyph::native_doc::{Doc, NativeVue2SfcObservation};
/// fn escape<'a>(owner: NativeVue2SfcObservation<'a>) -> &'a Doc<'a> {
///     owner.document().unwrap()
/// }
/// ```
/// ```compile_fail,E0505
/// use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in, print, PrintOptions};
/// use vize_l0::Allocator;
/// let arena = Allocator::default();
/// let owner = observe_native_vue2_sfc_in(&arena, "<template>{{a}}</template>", NativeVue2SfcOptions::default());
/// let document = owner.document().unwrap();
/// drop(owner);
/// println!("{}", print(document, &PrintOptions::default()));
/// ```
/// ```compile_fail,E0505
/// use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in};
/// use vize_l0::Allocator;
/// let arena = Allocator::default();
/// let owner = observe_native_vue2_sfc_in(&arena, "<template>{{a}}</template>", NativeVue2SfcOptions::default());
/// let selected = owner.selected().unwrap();
/// let moved = owner;
/// println!("{:?}", selected.component().block());
/// drop(moved);
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in};
/// use vize_l0::Allocator;
/// let arena = Allocator::default();
/// let owner = {
///     let source = String::from("<template>{{a}}</template>");
///     observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default())
/// };
/// println!("{:?}", owner.format());
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in};
/// use vize_l0::Allocator;
/// let owner = {
///     let arena = Allocator::default();
///     observe_native_vue2_sfc_in(&arena, "<template>{{a}}</template>", NativeVue2SfcOptions::default())
/// };
/// println!("{:?}", owner.format());
/// ```
/// ```compile_fail,E0507
/// use vize_glyph::native_doc::{Doc, NativeVue2SfcObservation};
/// fn take<'a>(owner: &NativeVue2SfcObservation<'a>) -> Doc<'a> {
///     *owner.document().unwrap()
/// }
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in};
/// use vize_l0::Allocator;
/// use vize_l1::container::vue::Vue2DescriptorObservation;
/// fn forge<'a>(arena: &'a Allocator, raw: Vue2DescriptorObservation<'a>) {
///     let _ = observe_native_vue2_sfc_in(arena, raw, NativeVue2SfcOptions::default());
/// }
/// ```
pub struct NativeVue2SfcObservation<'a> {
    pub(super) descriptor: Vue2DescriptorObservation<'a>,
    pub(super) options: NativeVue2SfcOptions,
    pub(super) outcome: Outcome<'a>,
}

impl core::fmt::Debug for NativeVue2SfcObservation<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeVue2SfcObservation")
            .field("source_length", &self.source().len())
            .field("options", &self.options)
            .field("has_component", &self.descriptor.component().is_some())
            .field("refusal", &self.refusal())
            .finish()
    }
}

impl<'a> NativeVue2SfcObservation<'a> {
    pub fn source(&self) -> &'a str {
        self.descriptor.source()
    }
    pub fn descriptor(&self) -> &Vue2DescriptorObservation<'a> {
        &self.descriptor
    }
    pub fn options(&self) -> NativeVue2SfcOptions {
        self.options
    }
    /// A new short borrow of the same original selected envelope, not a reparse.
    pub fn selected(&self) -> Result<Vue2TemplateView<'_, 'a>, Vue2DescriptorRefusal<'_>> {
        self.descriptor.selected()
    }
    pub fn refusal(&self) -> Option<NativeVue2SfcRefusal> {
        match &self.outcome {
            Outcome::Document(_) => None,
            Outcome::Refused(refusal) => Some(*refusal),
        }
    }
    pub fn document(&self) -> Result<&Doc<'a>, NativeVue2SfcRefusal> {
        match &self.outcome {
            Outcome::Document(document) => Ok(document),
            Outcome::Refused(refusal) => Err(*refusal),
        }
    }
    /// One complete root Doc is printed with captured native policy.
    /// No parser, trim, legacy fallback or default formatter is called.
    pub fn format(&self) -> Result<FormatResult, NativeVue2SfcRefusal> {
        let code = print(self.document()?, &self.options.print);
        Ok(FormatResult {
            changed: code != self.source(),
            code,
        })
    }
}
