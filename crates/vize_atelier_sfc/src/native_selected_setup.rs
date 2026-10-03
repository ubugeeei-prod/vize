//! Explicit whole original primitive setup SFC output, with retained custody.

use crate::{NativeSelectedSfcDomOptions, NativeSelectedSfcDomOutput};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::{
    NativeSelectedSetupSfcObservation, lower_selected_setup_sfc_native,
};
use vize_l3::decision::{DecisionBuildError, native::build_native_selected_setup_dom_decisions};
use vize_l4::{
    module::{
        AssemblyError, ModuleParts, RenderPlacement, RenderProperty, ScriptPart, assemble,
        setup::{COMPONENT_BINDING, SetupEmitError, emit_selected_setup},
    },
    runtime::Runtime,
    targets::dom::{DomError, emit_selected_setup_template},
    write::{LinkSink, NoLinks, Recorded},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSelectedSetupSfcDomError {
    Observation,
    Analysis(DecisionBuildError),
    Setup(SetupEmitError),
    Dom(DomError),
    Assembly(AssemblyError),
}

/// The original whole Descriptor, normally owned Program and actual template
/// File stay owned even when script or target emission refuses.
/// This is additive; the compiler's default/legacy entry is unchanged.
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;
/// use vize_atelier_sfc::{NativeSelectedSetupSfcDomCompilation,
///     NativeSelectedSetupSfcDomError, NativeSelectedSfcDomOutput};
/// fn forge<'a>(observation: NativeSelectedSetupSfcObservation<'a>,
///     result: Result<NativeSelectedSfcDomOutput, NativeSelectedSetupSfcDomError>) {
///     let _ = NativeSelectedSetupSfcDomCompilation { observation, result };
/// }
/// ```
/// ```compile_fail
/// use vize_atelier_sfc::NativeSelectedSetupSfcDomCompilation;
/// fn discard(compilation: NativeSelectedSetupSfcDomCompilation<'_>) {
///     let view = compilation.observation().admitted().unwrap();
///     drop(compilation);
///     let _ = view.setup();
/// }
/// ```
/// The original allocator also stays alive with the retained syntax:
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_atelier_sfc::{compile_native_selected_setup_sfc_dom, NativeSelectedSfcDomOptions};
/// fn discard_arena() {
///     let arena = Allocator::default();
///     let compiled = compile_native_selected_setup_sfc_dom(&arena,
///         "<script setup>let count=1</script><template>{{count}}</template>",
///         NativeSelectedSfcDomOptions::default());
///     drop(arena);
///     let _ = compiled.observation();
/// }
/// ```
#[derive(Debug)]
pub struct NativeSelectedSetupSfcDomCompilation<'a> {
    observation: NativeSelectedSetupSfcObservation<'a>,
    result: Result<NativeSelectedSfcDomOutput, NativeSelectedSetupSfcDomError>,
}
impl<'a> NativeSelectedSetupSfcDomCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSelectedSetupSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeSelectedSfcDomOutput, NativeSelectedSetupSfcDomError> {
        self.result.as_ref().map_err(|error| *error)
    }
}

/// Observe the original SFC once; consume its same-owner setup and DOM decisions.
/// Direct normalized Identifier/primitive literal template roots are bounded
/// reads. Imports, wider TS, compound/nested reads, outer handler accesses and
/// For loops remain precise lower/target refusals with no partial module.
#[must_use]
pub fn compile_native_selected_setup_sfc_dom<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSelectedSfcDomOptions<'_>,
) -> NativeSelectedSetupSfcDomCompilation<'a> {
    let observation = lower_selected_setup_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options)
    } else {
        emit::<NoLinks>(&observation, options)
    };
    NativeSelectedSetupSfcDomCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedSetupSfcObservation<'_>,
    options: NativeSelectedSfcDomOptions<'_>,
) -> Result<NativeSelectedSfcDomOutput, NativeSelectedSetupSfcDomError> {
    let admitted = observation
        .admitted()
        .ok_or(NativeSelectedSetupSfcDomError::Observation)?;
    let source = admitted.observation().original().descriptor().source();
    let analysis = build_native_selected_setup_dom_decisions(admitted.setup())
        .map_err(NativeSelectedSetupSfcDomError::Analysis)?;
    let mut parts =
        ModuleParts::for_runtime(Runtime::VueDom, options.runtime_version, COMPONENT_BINDING)
            .map_err(NativeSelectedSetupSfcDomError::Assembly)?;
    parts.script = Some(ScriptPart::Body(
        emit_selected_setup::<L>(analysis.setup())
            .map_err(NativeSelectedSetupSfcDomError::Setup)?,
    ));
    parts.render = Some(
        emit_selected_setup_template::<L>(&analysis)
            .map_err(NativeSelectedSetupSfcDomError::Dom)?,
    );
    parts.placement = RenderPlacement::Function {
        binding: "render",
        property: RenderProperty::Render,
    };
    let document = assemble(parts)
        .map_err(NativeSelectedSetupSfcDomError::Assembly)?
        .into_document();
    let map = options
        .source_map
        .then(|| document.source_map(options.filename, source));
    Ok(NativeSelectedSfcDomOutput::new(document, map))
}

#[cfg(test)]
mod tests;
