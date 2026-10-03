//! One normally owned CSS parse beside the original completed selected File.

use super::{
    NativeSelectedSfcIssue, NativeSelectedSfcIssueKind, NativeSelectedSfcObservation,
    selected::{self, SelectedMode},
};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::vue::{DescriptorObservation, DescriptorOptions},
    css::StyleSyntax,
};
use vize_l2::lang::js::{NativeScopedTemplateIssueKind as Kind, NativeScopedTemplateView};

#[cfg(test)]
mod tests;

/// Original Descriptor, original CSS parser owner and sole selected File stay
/// normally owned together. This route performs the CSS parse once, before the
/// same original template cursor; no generated CSS or scope policy lives here.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::{NativeSelectedScopedSfcObservation, NativeSelectedSfcObservation};
/// fn pair(original: NativeSelectedSfcObservation<'_>) {
///     let _ = NativeSelectedScopedSfcObservation { original };
/// }
/// ```
pub struct NativeSelectedScopedSfcObservation<'a> {
    original: NativeSelectedSfcObservation<'a>,
}
impl core::fmt::Debug for NativeSelectedScopedSfcObservation<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeSelectedScopedSfcObservation")
            .field("original", &self.original)
            .finish()
    }
}
impl<'a> NativeSelectedScopedSfcObservation<'a> {
    #[must_use]
    pub fn original(&self) -> &NativeSelectedSfcObservation<'a> {
        &self.original
    }
    /// The actual parser owner, including unsupported-token/error custody.
    #[must_use]
    pub fn style_syntax(&self) -> Option<&StyleSyntax<'a>> {
        self.original.style()
    }
    #[must_use]
    pub fn admitted(&self) -> Option<NativeSelectedScopedSfc<'_, 'a>> {
        if !self.original.issues().is_empty() {
            return None;
        }
        let template = self.original.template()?;
        let scoped = template
            .scoped_view(self.original.descriptor(), self.style_syntax()?)
            .ok()?;
        Some(NativeSelectedScopedSfc {
            observation: self,
            scoped,
        })
    }
}

/// Neither an arbitrary File nor a caller's CSS/output/eligibility pair can
/// construct this whole-envelope view. The original owners stay borrowed.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::{NativeSelectedScopedSfc, NativeSelectedScopedSfcObservation};
/// use vize_l2::lang::js::NativeScopedTemplateView;
/// fn forge<'o, 'a>(observation: &'o NativeSelectedScopedSfcObservation<'a>, scoped: NativeScopedTemplateView<'o, 'a>) {
///     let _ = NativeSelectedScopedSfc { observation, scoped };
/// }
/// ```
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedScopedSfc;
/// fn copy(view: NativeSelectedScopedSfc<'_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedScopedSfcObservation;
/// fn discard(owner: NativeSelectedScopedSfcObservation<'_>) {
///     let view = owner.admitted().unwrap(); drop(owner);
///     let _ = view.into_scoped_template_view();
/// }
/// ```
pub struct NativeSelectedScopedSfc<'owner, 'arena> {
    observation: &'owner NativeSelectedScopedSfcObservation<'arena>,
    scoped: NativeScopedTemplateView<'owner, 'arena>,
}
impl<'owner, 'arena> NativeSelectedScopedSfc<'owner, 'arena> {
    #[must_use]
    pub fn observation(&self) -> &'owner NativeSelectedScopedSfcObservation<'arena> {
        self.observation
    }
    #[must_use]
    pub fn into_scoped_template_view(self) -> NativeScopedTemplateView<'owner, 'arena> {
        self.scoped
    }
}

/// Additive scriptless Vue 3/default-profile, one bare scoped CSS block with an
/// actual literal `.class:empty` parser receipt. Existing Scriptless and Setup
/// entries retain every script/style refusal. Product decisions remain later.
#[must_use]
pub fn lower_selected_scoped_sfc_native<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSelectedScopedSfcObservation<'a> {
    NativeSelectedScopedSfcObservation {
        original: selected::observe(allocator, source, options, SelectedMode::ScriptlessScoped),
    }
}

pub(super) fn observe<'a>(
    original: &DescriptorObservation<'a>,
) -> Result<(StyleSyntax<'a>, Option<NativeSelectedSfcIssue>), NativeSelectedSfcIssue> {
    let options = original.options();
    let reject = |index, span, kind| NativeSelectedSfcIssue {
        container_index: index,
        span,
        kind: NativeSelectedSfcIssueKind::ScopedStyle(kind),
    };
    let whole = Span::new(0, original.source().len() as u32);
    if options.version != VueVersion::V3
        || options.dialect != VueDialect::Vue
        || options.template != SurfaceParseOptions::default()
    {
        return Err(reject(None, whole, Kind::Profile));
    }
    let descriptor = original
        .admitted()
        .map_err(|_| reject(None, whole, Kind::Descriptor))?;
    if descriptor.styles().len() != 1 {
        return Err(reject(None, whole, Kind::StyleCount));
    }
    let style = descriptor
        .styles()
        .next()
        .ok_or_else(|| reject(None, whole, Kind::StyleCount))?;
    if !style
        .attrs()
        .iter()
        .any(|attr| attr.name == "scoped" && attr.value.is_none())
        || style.attrs().iter().any(|attr| {
            !matches!(
                (attr.name, attr.value),
                ("scoped", None) | ("lang", Some("css"))
            )
        })
    {
        return Err(reject(
            Some(style.container_index()),
            style.block().span(),
            Kind::StyleProfile,
        ));
    }
    // Syntax refusals remain owned without starting an unsupported template.
    // Admission separately checks the actual token window, not a stored flag.
    let syntax = StyleSyntax::observe(style);
    let issue = syntax.empty_class().err().map(|issue| {
        reject(
            Some(style.container_index()),
            issue.span,
            Kind::StyleSyntax(issue),
        )
    });
    Ok((syntax, issue))
}
