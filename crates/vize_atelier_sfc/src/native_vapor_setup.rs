//! Explicit primitive setup Vue-source SFC compilation to the pinned Vapor runtime.
//! The once-selected original SFC/File observation survives every target refusal.

use vize_l0::{Allocator, String};
use vize_l1_to_l2::native_file::{
    NativeSelectedSetupSfcObservation, NativeSelectedSfcIssue, lower_selected_setup_sfc_native,
};
use vize_l3::decision::{DecisionBuildError, vapor::build_native_selected_setup_vapor_decisions};
use vize_l4::{
    targets::vapor::{VaporError, emit_selected_setup_component},
    write::{EmitDocument, LinkSink, NoLinks, Recorded},
};

use crate::native_vapor::NativeVaporSfcCompileOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVaporSetupSfcCompileError {
    Source(NativeSelectedSfcIssue),
    Orchestration,
    Analysis(DecisionBuildError),
    Vapor(VaporError),
}

/// A complete default component module; unsupported inputs return no output.
#[derive(Debug)]
pub struct NativeVaporSetupSfcOutput {
    document: EmitDocument,
    source_map: Option<String>,
}
impl NativeVaporSetupSfcOutput {
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

/// Sealed original observation and its associated whole output or refusal.
/// ```compile_fail
/// use vize_atelier_sfc::{NativeVaporSetupSfcCompilation, NativeVaporSetupSfcCompileError};
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;
/// fn forge(observation: NativeSelectedSetupSfcObservation<'_>) {
///     let _ = NativeVaporSetupSfcCompilation { observation, result: Err(NativeVaporSetupSfcCompileError::Orchestration) };
/// }
/// ```
#[derive(Debug)]
pub struct NativeVaporSetupSfcCompilation<'a> {
    observation: NativeSelectedSetupSfcObservation<'a>,
    result: Result<NativeVaporSetupSfcOutput, NativeVaporSetupSfcCompileError>,
}
impl<'a> NativeVaporSetupSfcCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSelectedSetupSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeVaporSetupSfcOutput, NativeVaporSetupSfcCompileError> {
        self.result.as_ref().map_err(|error| *error)
    }
    pub fn into_parts(
        self,
    ) -> (
        NativeSelectedSetupSfcObservation<'a>,
        Result<NativeVaporSetupSfcOutput, NativeVaporSetupSfcCompileError>,
    ) {
        (self.observation, self.result)
    }
}

/// Select the Descriptor once, exhaust its genuine template once, run the
/// existing sole L3 decision walk, and assemble the whole Vapor component.
/// Original pure declarations remain inside actual inline Vapor setup. No
/// public setup state, expose or separate render is synthesized. Unsupported
/// envelopes and target semantics retain their genuine typed refusal stage.
#[must_use]
pub fn compile_native_vapor_setup_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeVaporSfcCompileOptions<'_>,
) -> NativeVaporSetupSfcCompilation<'a> {
    let observation = lower_selected_setup_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options)
    } else {
        emit::<NoLinks>(&observation, options)
    };
    NativeVaporSetupSfcCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedSetupSfcObservation<'_>,
    options: NativeVaporSfcCompileOptions<'_>,
) -> Result<NativeVaporSetupSfcOutput, NativeVaporSetupSfcCompileError> {
    if let Some(issue) = observation.original().issues().first() {
        return Err(NativeVaporSetupSfcCompileError::Source(*issue));
    }
    let selected = observation
        .admitted()
        .ok_or(NativeVaporSetupSfcCompileError::Orchestration)?;
    let analysis = build_native_selected_setup_vapor_decisions(selected.setup())
        .map_err(NativeVaporSetupSfcCompileError::Analysis)?;
    let document = emit_selected_setup_component::<L>(&analysis, options.runtime_version)
        .map_err(NativeVaporSetupSfcCompileError::Vapor)?
        .into_document();
    let source_map = options.source_map.then(|| {
        String::new(document.source_map(
            options.filename,
            observation.original().descriptor().container().source,
        ))
    });
    Ok(NativeVaporSetupSfcOutput {
        document,
        source_map,
    })
}
