//! Explicit whole original primitive setup SFC output, with retained custody.

use crate::{NativeSelectedSfcDomOptions, NativeSelectedSfcDomOutput};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::{
    NativeSelectedSetupSfcObservation, lower_selected_setup_sfc_native,
};
use vize_l3::decision::{DecisionBuildError, native::build_native_selected_setup_dom_decisions};
use vize_l4::{
    module::{
        AssemblyError, ModuleParts, RenderFunctionName, ScriptPart, assemble,
        setup::{COMPONENT_BINDING, SetupEmitError, emit_selected_setup},
    },
    runtime::Runtime,
    targets::dom::{DomError, DomMode, emit_selected_setup_template_with_mode},
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
/// Actual emitted For facts cannot survive destruction of their original owner:
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_l2::op::Op;
/// use vize_l3::decision::native::build_native_selected_setup_dom_decisions;
/// use vize_atelier_sfc::{compile_native_selected_setup_sfc_dom, NativeSelectedSfcDomOptions};
/// fn discard_for() {
///     let arena = Allocator::default();
///     let compiled = compile_native_selected_setup_sfc_dom(&arena,
///         "<script setup>let count=2</script><template><i v-for='item in count'>fixed</i></template>",
///         NativeSelectedSfcDomOptions::default());
///     let view = compiled.observation().admitted().unwrap();
///     let analysis = build_native_selected_setup_dom_decisions(view.setup()).unwrap();
///     let Op::OriginalFor(original) = &view.setup().file().artifact().root().ops[0] else { return };
///     let row = analysis.dom().unwrap().file_for_head(original.id().node()).unwrap();
///     drop(compiled);
///     let _ = row.resolution();
/// }
/// ```
/// Correctly typed independent owners cannot construct a paired completion:
/// ```compile_fail
/// use vize_l1_to_l2::native_file::{NativeSelectedSetupSfc, NativeSelectedSetupSfcObservation};
/// use vize_l2::lang::js::{NativeSelectedSetup, NativeTemplateView};
/// fn pair<'o, 'a>(observation: &'o NativeSelectedSetupSfcObservation<'a>,
///     template: NativeTemplateView<'o, 'a>, setup: NativeSelectedSetup<'o, 'a>) {
///     let _ = NativeSelectedSetupSfc { observation, template, setup };
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
/// reads. One root primitive original For may repeat an attribute-free native
/// Element with empty/static text or one bare current value-alias interpolation.
/// A two-alias head requires one bare current key-alias interpolation and emits
/// both original callback parameters in source order.
/// One root If may read a direct setup BooleanLiteral binding and own one
/// attribute-free native Element with empty/static text or one bare primitive
/// setup interpolation in the same File scope. An adjacent original valueless
/// Else may own another Element with the same bounded body shape. Either paired
/// branch may also combine one original literal Text and one bare primitive
/// setup interpolation in their authored order.
/// One original root input may bind a bare Model Identifier to a genuine direct
/// mutable setup declaration with primitive initial value. Its same-File Model
/// record grants the distinct read/write receipt; ordinary reads remain read-only.
/// Imports, wider TS, compound reads, outer handler accesses and
/// nested/mixed-body For families remain
/// precise lower/target refusals with no partial module.
#[must_use]
pub fn compile_native_selected_setup_sfc_dom<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSelectedSfcDomOptions<'_>,
) -> NativeSelectedSetupSfcDomCompilation<'a> {
    compile_native_selected_setup_sfc_dom_with_mode(
        allocator,
        source,
        options,
        DomMode::Development,
    )
}

/// Select genuine development or production runtime comment behavior while
/// keeping the original setup/template owner and all existing source policies.
/// The default entry retains its development spelling and options layout.
#[must_use]
pub fn compile_native_selected_setup_sfc_dom_with_mode<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSelectedSfcDomOptions<'_>,
    mode: DomMode,
) -> NativeSelectedSetupSfcDomCompilation<'a> {
    let observation = lower_selected_setup_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options, mode)
    } else {
        emit::<NoLinks>(&observation, options, mode)
    };
    NativeSelectedSetupSfcDomCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedSetupSfcObservation<'_>,
    options: NativeSelectedSfcDomOptions<'_>,
    mode: DomMode,
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
        emit_selected_setup_template_with_mode::<L>(&analysis, mode)
            .map_err(NativeSelectedSetupSfcDomError::Dom)?,
    );
    parts.placement = RenderFunctionName::Render.placement();
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
