//! Explicit original primitive setup SFC through its same-owner SSR collector.

use crate::{NativeSsrSfcCompileOptions, NativeSsrSfcOutput};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::{
    NativeSelectedSetupSfcObservation, NativeSelectedSfcIssue, lower_selected_setup_sfc_native,
};
use vize_l3::decision::ssr::{NativeSsrBuildError, build_native_selected_setup_ssr_decisions};
use vize_l4::{
    module::{
        AssemblyError, ModuleParts, RenderPlacement, RenderProperty, ScriptPart, assemble,
        setup::{COMPONENT_BINDING, SetupEmitError, emit_selected_setup},
    },
    runtime::Runtime,
    targets::ssr::{SsrError, emit_selected_setup_template},
    write::{LinkSink, NoLinks, Recorded},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSetupSsrSfcCompileError {
    Lowering(NativeSelectedSfcIssue),
    Custody,
    Analysis(NativeSsrBuildError),
    Setup(SetupEmitError),
    Ssr(SsrError),
    Assembly(AssemblyError),
}

/// The original whole descriptor, Program and completed/partial File stay owned.
/// Maps always derive this original root; no external source pair is accepted.
/// ```compile_fail
/// use vize_atelier_sfc::{NativeSetupSsrSfcCompilation, NativeSetupSsrSfcCompileError};
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;
/// fn forge(observation: NativeSelectedSetupSfcObservation<'_>) {
///     let _ = NativeSetupSsrSfcCompilation {
///         observation, result: Err(NativeSetupSsrSfcCompileError::Custody),
///     };
/// }
/// ```
/// ```compile_fail
/// use vize_atelier_sfc::NativeSetupSsrSfcCompilation;
/// fn discard(compilation: NativeSetupSsrSfcCompilation<'_>) {
///     let view = compilation.observation().admitted().unwrap();
///     drop(compilation);
///     let _ = view.setup().program();
/// }
/// ```
#[derive(Debug)]
pub struct NativeSetupSsrSfcCompilation<'a> {
    observation: NativeSelectedSetupSfcObservation<'a>,
    result: Result<NativeSsrSfcOutput, NativeSetupSsrSfcCompileError>,
}
impl<'a> NativeSetupSsrSfcCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSelectedSetupSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeSsrSfcOutput, NativeSetupSsrSfcCompileError> {
        self.result.as_ref().map_err(|error| *error)
    }
}

/// One original JS/TS setup owner, external SSR function and full component.
/// Bounded primitive declarations and one root primitive/Identifier
/// interpolation beside comments or static markup use Vue's actual sixth setup-state argument. Original HTML
/// listeners stay owned and are omitted without evaluation. Adjacent root text
/// runs, nested reads, For/components/dynamic markup and unsupported original
/// scripts/styles/profiles remain typed refusals with no partial module.
/// This additive entry does not replace default compiler routing or add Vite's
/// SSR-context registration, HMR or upstream raw-map parity.
#[must_use]
pub fn compile_native_setup_ssr_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSsrSfcCompileOptions<'_>,
) -> NativeSetupSsrSfcCompilation<'a> {
    let observation = lower_selected_setup_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options)
    } else {
        emit::<NoLinks>(&observation, options)
    };
    NativeSetupSsrSfcCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedSetupSfcObservation<'_>,
    options: NativeSsrSfcCompileOptions<'_>,
) -> Result<NativeSsrSfcOutput, NativeSetupSsrSfcCompileError> {
    if let Some(issue) = observation.original().issues().first() {
        return Err(NativeSetupSsrSfcCompileError::Lowering(*issue));
    }
    let admitted = observation
        .admitted()
        .ok_or(NativeSetupSsrSfcCompileError::Custody)?;
    let analysis = build_native_selected_setup_ssr_decisions(admitted.setup())
        .map_err(NativeSetupSsrSfcCompileError::Analysis)?;
    let mut parts = ModuleParts::for_runtime(
        Runtime::VueServerRenderer,
        options.runtime_version,
        COMPONENT_BINDING,
    )
    .map_err(NativeSetupSsrSfcCompileError::Assembly)?;
    parts.script = Some(ScriptPart::Body(
        emit_selected_setup::<L>(analysis.setup()).map_err(NativeSetupSsrSfcCompileError::Setup)?,
    ));
    parts.render = Some(
        emit_selected_setup_template::<L>(&analysis).map_err(NativeSetupSsrSfcCompileError::Ssr)?,
    );
    parts.placement = RenderPlacement::Function {
        binding: "ssrRender",
        property: RenderProperty::SsrRender,
    };
    let document = assemble(parts)
        .map_err(NativeSetupSsrSfcCompileError::Assembly)?
        .into_document();
    let map = options.source_map.then(|| {
        document.source_map(
            options.filename,
            observation.original().descriptor().source(),
        )
    });
    Ok(NativeSsrSfcOutput::new(document, map))
}
