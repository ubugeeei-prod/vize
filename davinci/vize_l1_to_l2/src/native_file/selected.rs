//! Whole scriptless SFC custody through its once-selected original template.

use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Allocator, Span};
use vize_l1::{
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions, ScriptRole},
    },
    markup::{ComponentSourceError, NativeInterpolationFailure, NativeTemplateComponent},
};
use vize_l2::{
    artifact::ArtifactError,
    lang::js::{
        NativeTemplateFile, NativeTemplateIssue, NativeTemplateOwner, NativeTemplateView,
        RejectedNativeTemplateOwner,
    },
};

#[cfg(test)]
mod tests;
mod walk;

/// This additive family does not omit or compile unsupported sibling blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSelectedSfcIssueKind {
    Descriptor,
    Script(ScriptRole),
    Style,
    MissingTemplate,
    MissingSetup,
    Source(ComponentSourceError),
    FileCreation(ArtifactError),
    Template(NativeTemplateIssue),
    RootText(vize_l1::markup::NativeRootTextError),
    Interpolation(vize_l1::markup::NativeInterpolationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSelectedSfcIssue {
    pub container_index: Option<usize>,
    pub span: Span,
    pub kind: NativeSelectedSfcIssueKind,
}

/// The actual Descriptor and selected File remain normally owned together.
/// A refusal preserves the original capture and any already-created File/body.
/// No caller-supplied File, source block or syntax can construct this owner.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSfcObservation;
/// fn alter(owner: &mut NativeSelectedSfcObservation<'_>) {
///     owner.template.take();
/// }
/// ```
pub struct NativeSelectedSfcObservation<'a> {
    descriptor: DescriptorObservation<'a>,
    template: Option<NativeTemplateFile<'a>>,
    rejected_creation: Option<Box<RejectedNativeTemplateOwner<'a>>>,
    interpolation_failure: Option<NativeInterpolationFailure<'a>>,
    issues: Vec<NativeSelectedSfcIssue>,
}

impl core::fmt::Debug for NativeSelectedSfcObservation<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeSelectedSfcObservation")
            .field("descriptor", &self.descriptor)
            .field("issues", &self.issues)
            .field("has_template", &self.template.is_some())
            .field("interpolation_failure", &self.interpolation_failure)
            .finish_non_exhaustive()
    }
}

impl<'a> NativeSelectedSfcObservation<'a> {
    #[must_use]
    pub fn descriptor(&self) -> &DescriptorObservation<'a> {
        &self.descriptor
    }
    #[must_use]
    pub fn template(&self) -> Option<&NativeTemplateFile<'a>> {
        self.template.as_ref()
    }
    #[must_use]
    pub fn rejected_creation(&self) -> Option<&RejectedNativeTemplateOwner<'a>> {
        self.rejected_creation.as_deref()
    }
    #[must_use]
    pub fn interpolation_failure(&self) -> Option<&NativeInterpolationFailure<'a>> {
        self.interpolation_failure.as_ref()
    }
    #[must_use]
    pub fn issues(&self) -> &[NativeSelectedSfcIssue] {
        &self.issues
    }

    /// Move only this original File's normal-completion view to a target.
    /// Product eligibility remains a separate L3/L4 obligation.
    #[must_use]
    pub fn admitted(&self) -> Option<NativeSelectedSfc<'_, 'a>> {
        if !self.issues.is_empty() {
            return None;
        }
        let descriptor = self.descriptor.admitted().ok()?;
        let original = self.template.as_ref()?;
        let selected = descriptor.template()?;
        let block = original.selected().component().block();
        let template = original.view().ok()?;
        let file = template.file()?;
        if descriptor.ordinary().is_some()
            || descriptor.setup().is_some()
            || descriptor.styles().len() != 0
            || original.selected().ordinary().is_some()
            || original.selected().setup().is_some()
            || original.selected().has_styles()
            || !file.units().is_empty()
            || original.selected().template_index() != selected.container_index()
            || block.span() != selected.block().span()
            || !core::ptr::eq(block.source(), selected.block().source())
            || !core::ptr::eq(block.root_source(), descriptor.source())
            || !core::ptr::eq(file.artifact().source(), descriptor.source())
        {
            return None;
        }
        Some(NativeSelectedSfc {
            observation: self,
            template,
        })
    }
}

/// A short view joins the whole original SFC with its actual selected File.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::{NativeSelectedSfc, NativeSelectedSfcObservation};
/// fn forge<'o, 'a>(observation: &'o NativeSelectedSfcObservation<'a>) {
///     let template = observation.template().unwrap().view().unwrap();
///     let _ = NativeSelectedSfc { observation, template };
/// }
/// ```
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSfcObservation;
/// fn discard(owner: NativeSelectedSfcObservation<'_>) {
///     let view = owner.admitted().unwrap();
///     drop(owner);
///     let _ = view.into_template_view();
/// }
/// ```
pub struct NativeSelectedSfc<'o, 'a> {
    observation: &'o NativeSelectedSfcObservation<'a>,
    template: NativeTemplateView<'o, 'a>,
}
impl<'o, 'a> NativeSelectedSfc<'o, 'a> {
    #[must_use]
    pub fn observation(&self) -> &'o NativeSelectedSfcObservation<'a> {
        self.observation
    }
    #[must_use]
    pub fn into_template_view(self) -> NativeTemplateView<'o, 'a> {
        self.template
    }
}

#[derive(Clone, Copy)]
pub(super) enum SelectedMode {
    Scriptless,
    Setup,
}

/// Observe one whole SFC, parse its actual selected template once, and exhaust
/// the original root cursor. Script/style/custom/external/profile refusals do
/// not grant a template view or parse unsupported script bodies. Generic
/// `lower_sfc_native` retains its existing ordinary/setup/style contract.
#[must_use]
pub fn lower_selected_sfc_native<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSelectedSfcObservation<'a> {
    observe(allocator, source, options, SelectedMode::Scriptless)
}

pub(super) fn observe<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
    mode: SelectedMode,
) -> NativeSelectedSfcObservation<'a> {
    let descriptor = Vue.observe_descriptor(allocator, source, options);
    let mut observation = NativeSelectedSfcObservation {
        descriptor,
        template: None,
        rejected_creation: None,
        interpolation_failure: None,
        issues: Vec::new(),
    };
    let whole = Span::new(0, source.len() as u32);
    let descriptor = match observation.descriptor.admitted() {
        Ok(view) => view,
        Err(_) => {
            observation.issues.push(NativeSelectedSfcIssue {
                container_index: None,
                span: whole,
                kind: NativeSelectedSfcIssueKind::Descriptor,
            });
            return observation;
        }
    };
    for script in [descriptor.ordinary(), descriptor.setup()]
        .into_iter()
        .flatten()
    {
        if matches!(mode, SelectedMode::Setup) && script.role() == ScriptRole::Setup {
            continue;
        }
        observation.issues.push(NativeSelectedSfcIssue {
            container_index: Some(script.container_index()),
            span: script.block().span(),
            kind: NativeSelectedSfcIssueKind::Script(script.role()),
        });
    }
    for style in descriptor.styles() {
        observation.issues.push(NativeSelectedSfcIssue {
            container_index: Some(style.container_index()),
            span: style.block().span(),
            kind: NativeSelectedSfcIssueKind::Style,
        });
    }
    if matches!(mode, SelectedMode::Setup) && descriptor.setup().is_none() {
        observation.issues.push(NativeSelectedSfcIssue {
            container_index: None,
            span: whole,
            kind: NativeSelectedSfcIssueKind::MissingSetup,
        });
    }
    if !observation.issues.is_empty() {
        return observation;
    }
    let index = descriptor
        .template()
        .map(|template| template.container_index());
    let span = descriptor
        .template()
        .map_or(whole, |template| template.block().span());
    let selected = match NativeTemplateComponent::parse_in(allocator, descriptor) {
        Ok(Some(selected)) => selected,
        Ok(None) => {
            observation.issues.push(NativeSelectedSfcIssue {
                container_index: index,
                span,
                kind: NativeSelectedSfcIssueKind::MissingTemplate,
            });
            return observation;
        }
        Err(error) => {
            observation.issues.push(NativeSelectedSfcIssue {
                container_index: index,
                span,
                kind: NativeSelectedSfcIssueKind::Source(error),
            });
            return observation;
        }
    };
    let mut owner = match NativeTemplateOwner::new(selected) {
        Ok(owner) => owner,
        Err(rejected) => {
            observation.issues.push(NativeSelectedSfcIssue {
                container_index: index,
                span,
                kind: NativeSelectedSfcIssueKind::FileCreation(rejected.error()),
            });
            observation.rejected_creation = Some(rejected);
            return observation;
        }
    };
    if matches!(mode, SelectedMode::Setup)
        && let Err(issue) = owner.parse_setup_program()
    {
        observation.issues.push(NativeSelectedSfcIssue {
            container_index: index,
            span: issue.span,
            kind: NativeSelectedSfcIssueKind::Template(issue),
        });
        observation.template = Some(owner.finish());
        return observation;
    }
    let result = match mode {
        SelectedMode::Scriptless => walk::construct(&mut owner),
        SelectedMode::Setup => walk::construct_setup(&mut owner),
    };
    if let Err(failure) = result {
        observation.issues.push(NativeSelectedSfcIssue {
            container_index: index,
            span: failure.span.unwrap_or(span),
            kind: failure.kind,
        });
        observation.interpolation_failure = failure.interpolation;
    }
    observation.template = Some(owner.finish());
    observation
}
