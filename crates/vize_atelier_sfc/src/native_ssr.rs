//! Explicit scriptless SSR compilation retaining the once-observed whole SFC.

use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::{
    NativeSelectedSfcIssue, NativeSelectedSfcObservation, lower_selected_sfc_native,
};
use vize_l3::decision::ssr::{NativeSsrBuildError, build_native_ssr_file_decisions};
use vize_l4::{
    module::{
        AssemblyError, ModuleParts, RenderPlacement, RenderProperty, assemble,
        setup::COMPONENT_BINDING,
    },
    runtime::Runtime,
    targets::ssr::{SsrError, emit_template},
    write::{EmitDocument, LinkSink, NoLinks, Recorded},
};

/// Original descriptor policy and checked server runtime vocabulary.
#[derive(Debug, Clone, Copy)]
pub struct NativeSsrSfcCompileOptions<'o> {
    pub descriptor: DescriptorOptions,
    pub filename: &'o str,
    pub runtime_version: &'o str,
    pub source_map: bool,
}

impl Default for NativeSsrSfcCompileOptions<'static> {
    fn default() -> Self {
        Self {
            descriptor: DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
            filename: "anonymous.vue",
            runtime_version: "3.5.35",
            source_map: false,
        }
    }
}

/// A refusal retains the same whole source and any original partial File.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSsrSfcCompileError {
    Lowering(NativeSelectedSfcIssue),
    Custody,
    Analysis(NativeSsrBuildError),
    Ssr(SsrError),
    Assembly(AssemblyError),
}

/// Complete exported component module, with optional original whole-source map.
#[derive(Debug)]
pub struct NativeSsrSfcOutput {
    document: EmitDocument,
    source_map: Option<String>,
}

impl NativeSsrSfcOutput {
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

/// Original descriptor and completed/partial selected File beside their result.
/// Construction is private; maps never accept independently paired source text.
///
/// ```compile_fail
/// use vize_atelier_sfc::{NativeSsrSfcCompilation, NativeSsrSfcCompileError};
/// use vize_l1_to_l2::native_file::NativeSelectedSfcObservation;
/// fn forge(observation: NativeSelectedSfcObservation<'_>) {
///     let _ = NativeSsrSfcCompilation {
///         observation, result: Err(NativeSsrSfcCompileError::Custody),
///     };
/// }
/// ```
#[derive(Debug)]
pub struct NativeSsrSfcCompilation<'a> {
    observation: NativeSelectedSfcObservation<'a>,
    result: Result<NativeSsrSfcOutput, NativeSsrSfcCompileError>,
}

impl<'a> NativeSsrSfcCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSelectedSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeSsrSfcOutput, NativeSsrSfcCompileError> {
        self.result.as_ref().map_err(|error| *error)
    }
    pub fn into_parts(
        self,
    ) -> (
        NativeSelectedSfcObservation<'a>,
        Result<NativeSsrSfcOutput, NativeSsrSfcCompileError>,
    ) {
        (self.observation, self.result)
    }
}

/// Compile the authentic scriptless whole-SFC owner through its selected view.
///
/// Existing native HTML/text/comment and root-fallthrough SSR semantics apply.
/// Admitted original HTML listeners are omitted without evaluating their bodies.
/// Scripts, styles, custom/external blocks, unsupported profiles and unavailable
/// dynamic/component/slot semantics return typed refusals with no partial module.
/// This compiler surface does not add Vite SSR-context registration or HMR, and
/// does not replace the default compiler while its fix-history gate remains open.
#[must_use]
pub fn compile_native_ssr_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSsrSfcCompileOptions<'_>,
) -> NativeSsrSfcCompilation<'a> {
    let observation = lower_selected_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options)
    } else {
        emit::<NoLinks>(&observation, options)
    };
    NativeSsrSfcCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedSfcObservation<'_>,
    options: NativeSsrSfcCompileOptions<'_>,
) -> Result<NativeSsrSfcOutput, NativeSsrSfcCompileError> {
    if let Some(issue) = observation.issues().first() {
        return Err(NativeSsrSfcCompileError::Lowering(*issue));
    }
    let admitted = observation
        .admitted()
        .ok_or(NativeSsrSfcCompileError::Custody)?;
    let receipt = build_native_ssr_file_decisions(admitted.into_template_view())
        .map_err(NativeSsrSfcCompileError::Analysis)?;
    let mut parts = ModuleParts::for_runtime(
        Runtime::VueServerRenderer,
        options.runtime_version,
        COMPONENT_BINDING,
    )
    .map_err(NativeSsrSfcCompileError::Assembly)?;
    parts.render = Some(emit_template::<L>(&receipt).map_err(NativeSsrSfcCompileError::Ssr)?);
    parts.placement = RenderPlacement::Function {
        binding: "ssrRender",
        property: RenderProperty::SsrRender,
    };
    let document = assemble(parts)
        .map_err(NativeSsrSfcCompileError::Assembly)?
        .into_document();
    let source_map = options
        .source_map
        .then(|| document.source_map(options.filename, observation.descriptor().source()));
    Ok(NativeSsrSfcOutput {
        document,
        source_map,
    })
}
