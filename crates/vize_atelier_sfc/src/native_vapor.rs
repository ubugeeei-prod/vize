//! Explicit scriptless Vue-source SFC compilation to the pinned Vapor runtime.
//! The once-selected original SFC/File observation survives every target refusal.

use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::{
    NativeSelectedSfcIssue, NativeSelectedSfcObservation, lower_selected_sfc_native,
};
use vize_l3::decision::{DecisionBuildError, vapor::build_native_vapor_file_decisions};
use vize_l4::{
    targets::vapor::{VaporError, emit_component},
    write::{EmitDocument, LinkSink, NoLinks, Recorded},
};

/// Vue source syntax is independent of the explicitly selected Vapor target.
#[derive(Debug, Clone, Copy)]
pub struct NativeVaporSfcCompileOptions<'o> {
    pub descriptor: DescriptorOptions,
    pub filename: &'o str,
    pub runtime_version: &'o str,
    pub source_map: bool,
}
impl Default for NativeVaporSfcCompileOptions<'static> {
    fn default() -> Self {
        Self {
            descriptor: DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
            filename: "anonymous.vue",
            runtime_version: "3.6.0-rc.9",
            source_map: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVaporSfcCompileError {
    Source(NativeSelectedSfcIssue),
    Orchestration,
    Analysis(DecisionBuildError),
    Vapor(VaporError),
}

/// A complete default component module; unsupported inputs return no output.
#[derive(Debug)]
pub struct NativeVaporSfcOutput {
    document: EmitDocument,
    source_map: Option<String>,
}
impl NativeVaporSfcOutput {
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
/// use vize_atelier_sfc::{NativeVaporSfcCompilation, NativeVaporSfcCompileError};
/// use vize_l1_to_l2::native_file::NativeSelectedSfcObservation;
/// fn forge(observation: NativeSelectedSfcObservation<'_>) {
///     let _ = NativeVaporSfcCompilation { observation, result: Err(NativeVaporSfcCompileError::Orchestration) };
/// }
/// ```
#[derive(Debug)]
pub struct NativeVaporSfcCompilation<'a> {
    observation: NativeSelectedSfcObservation<'a>,
    result: Result<NativeVaporSfcOutput, NativeVaporSfcCompileError>,
}
impl<'a> NativeVaporSfcCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSelectedSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeVaporSfcOutput, NativeVaporSfcCompileError> {
        self.result.as_ref().map_err(|error| *error)
    }
    pub fn into_parts(
        self,
    ) -> (
        NativeSelectedSfcObservation<'a>,
        Result<NativeVaporSfcOutput, NativeVaporSfcCompileError>,
    ) {
        (self.observation, self.result)
    }
}

/// Select the Descriptor once, exhaust its genuine template once, run the
/// existing sole L3 decision walk, and assemble the whole Vapor component.
/// Script/style/custom/external/header/profile and unsupported template
/// semantics remain typed refusals; no legacy compiler or default route runs.
#[must_use]
pub fn compile_native_vapor_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeVaporSfcCompileOptions<'_>,
) -> NativeVaporSfcCompilation<'a> {
    let observation = lower_selected_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options)
    } else {
        emit::<NoLinks>(&observation, options)
    };
    NativeVaporSfcCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedSfcObservation<'_>,
    options: NativeVaporSfcCompileOptions<'_>,
) -> Result<NativeVaporSfcOutput, NativeVaporSfcCompileError> {
    if let Some(issue) = observation.issues().first() {
        return Err(NativeVaporSfcCompileError::Source(*issue));
    }
    let selected = observation
        .admitted()
        .ok_or(NativeVaporSfcCompileError::Orchestration)?;
    let analysis = build_native_vapor_file_decisions(selected.into_template_view())
        .map_err(NativeVaporSfcCompileError::Analysis)?;
    let document = emit_component::<L>(&analysis, options.runtime_version)
        .map_err(NativeVaporSfcCompileError::Vapor)?
        .into_document();
    let source_map = options.source_map.then(|| {
        String::new(document.source_map(
            options.filename,
            observation.descriptor().container().source,
        ))
    });
    Ok(NativeVaporSfcOutput {
        document,
        source_map,
    })
}
