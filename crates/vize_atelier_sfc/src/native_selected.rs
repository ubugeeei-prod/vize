//! Explicit scriptless component output with whole original selected custody.

use vize_l0::{Allocator, String};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::{NativeSelectedSfcObservation, lower_selected_sfc_native};
use vize_l3::decision::{DecisionBuildError, native::build_native_dom_file_decisions};
use vize_l4::{
    module::{
        AssemblyError, ModuleParts, RenderPlacement, RenderProperty, assemble,
        setup::COMPONENT_BINDING,
    },
    runtime::Runtime,
    targets::dom::{DomError, emit_template},
    write::{EmitDocument, LinkSink, NoLinks, Recorded},
};

#[derive(Debug, Clone, Copy)]
pub struct NativeSelectedSfcDomOptions<'o> {
    pub descriptor: DescriptorOptions,
    pub filename: &'o str,
    pub runtime_version: &'o str,
    pub source_map: bool,
}
impl Default for NativeSelectedSfcDomOptions<'static> {
    fn default() -> Self {
        Self {
            descriptor: DescriptorOptions {
                version: vize_l0::config::VueVersion::V3,
                dialect: vize_l0::config::VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
            filename: "anonymous.vue",
            runtime_version: "3.5.35",
            source_map: false,
        }
    }
}

/// Refusal never returns a partial component or falls back to legacy compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSelectedSfcDomError {
    Observation,
    Analysis(DecisionBuildError),
    Dom(DomError),
    Assembly(AssemblyError),
}

#[derive(Debug)]
pub struct NativeSelectedSfcDomOutput {
    document: EmitDocument,
    source_map: Option<String>,
}
impl NativeSelectedSfcDomOutput {
    #[must_use]
    pub fn code(&self) -> &str {
        self.document.as_str()
    }
    #[must_use]
    pub fn document(&self) -> &EmitDocument {
        &self.document
    }
    #[must_use]
    pub fn source_map(&self) -> Option<&str> {
        self.source_map.as_deref()
    }
}

/// One original SFC and the output derived only from its own complete template.
/// Caller-paired Descriptor/File/output fields cannot construct this result.
///
/// ```compile_fail
/// use vize_atelier_sfc::NativeSelectedSfcDomCompilation;
/// use vize_l1_to_l2::native_file::NativeSelectedSfcObservation;
/// fn forge(observation: NativeSelectedSfcObservation<'_>) {
///     let _ = NativeSelectedSfcDomCompilation { observation, result: Err(
///         vize_atelier_sfc::NativeSelectedSfcDomError::Observation) };
/// }
/// ```
#[derive(Debug)]
pub struct NativeSelectedSfcDomCompilation<'a> {
    observation: NativeSelectedSfcObservation<'a>,
    result: Result<NativeSelectedSfcDomOutput, NativeSelectedSfcDomError>,
}
impl<'a> NativeSelectedSfcDomCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSelectedSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeSelectedSfcDomOutput, NativeSelectedSfcDomError> {
        self.result.as_ref().map_err(|error| *error)
    }
}

/// Observe once and compile through the actual selected File's sole L3 walk.
/// Scripts/styles/custom/external blocks and unsupported source profiles are
/// explicit lower refusals; target controls/aliases, runtime reads and broader
/// events remain typed target refusals. This additive entry never changes the
/// default compiler or existing ordinary/setup/style product contracts.
#[must_use]
pub fn compile_native_selected_sfc_dom<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSelectedSfcDomOptions<'_>,
) -> NativeSelectedSfcDomCompilation<'a> {
    let observation = lower_selected_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options)
    } else {
        emit::<NoLinks>(&observation, options)
    };
    NativeSelectedSfcDomCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedSfcObservation<'_>,
    options: NativeSelectedSfcDomOptions<'_>,
) -> Result<NativeSelectedSfcDomOutput, NativeSelectedSfcDomError> {
    let admitted = observation
        .admitted()
        .ok_or(NativeSelectedSfcDomError::Observation)?;
    let source = admitted.observation().descriptor().source();
    let analysis = build_native_dom_file_decisions(admitted.into_template_view())
        .map_err(NativeSelectedSfcDomError::Analysis)?;
    let mut parts =
        ModuleParts::for_runtime(Runtime::VueDom, options.runtime_version, COMPONENT_BINDING)
            .map_err(NativeSelectedSfcDomError::Assembly)?;
    parts.render = Some(emit_template::<L>(&analysis).map_err(NativeSelectedSfcDomError::Dom)?);
    parts.placement = RenderPlacement::Function {
        binding: "render",
        property: RenderProperty::Render,
    };
    let document = assemble(parts)
        .map_err(NativeSelectedSfcDomError::Assembly)?
        .into_document();
    let source_map = options
        .source_map
        .then(|| document.source_map(options.filename, source));
    Ok(NativeSelectedSfcDomOutput {
        document,
        source_map,
    })
}

#[cfg(test)]
mod tests;
