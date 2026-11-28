//! SFC compilation implementation.
//!
//! SFC assembly delegates template, script and style compilation to their compiler lanes.

mod bindings;
mod diagnostics;
mod empty_component;
mod entry;
mod helpers;
mod module_trace;
mod normal_script;
pub(crate) mod output_module;
mod prelude;
mod props_rest;
mod script_component;
mod script_context;
mod setup_component;
mod setup_exports;
pub(crate) mod setup_facts;
#[cfg(test)]
mod setup_syntax_tests;
mod stage_capture;
#[cfg(test)]
mod stage_capture_tests;
mod styles;
mod targets;
#[cfg(test)]
pub(crate) mod targets_tests;
mod template_context;
mod template_only;
mod template_parts;
#[cfg(test)]
#[expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
mod tests;

use crate::types::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
    SfcScriptBlock, SfcTemplateBlock,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

pub(crate) use self::helpers::is_ts_lang;
use self::helpers::{
    extract_component_name, extract_descriptor_macro_artifacts, output_is_ts, template_is_ts,
    vapor_requested,
};
use self::prelude::{Outputs, Prelude, Target};
use self::setup_component::{SetupInputs, SetupSources, SetupView};

mod public;
pub use public::*;
use vize_l0::dump::capture::StageCapture;
use vize_l1_to_l2::lower::pug::PugSourceMap;

#[expect(clippy::too_many_arguments, reason = "optional stage capture")]
fn compile_sfc_inner(
    descriptor: &SfcDescriptor,
    mut options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
    nuxt_page_meta: bool,
    type_project: Option<&crate::script::SfcTypeProject>,
    pug_source_map: Option<&PugSourceMap>,
    mut capture: Option<&mut StageCapture>,
) -> Result<SfcCompileResult, SfcError> {
    empty_component::validate_component(descriptor)?;
    let mut warnings = Vec::new();
    let macro_artifacts = extract_descriptor_macro_artifacts(descriptor, nuxt_page_meta);
    let filenames = prelude::filenames(&options.parse.filename, options.script.id.as_deref());
    let styles = prelude::Styles::compile(descriptor, &options, filenames.0, &mut warnings);
    let auto_ssr_css_vars = options.template.ssr_css_vars.is_none();
    let vapor_requested = vapor_requested(descriptor, &options);
    let (is_vapor, script_output) = prelude::choose_target(
        descriptor,
        &mut options.template,
        vapor_requested,
        script_output,
        auto_ssr_css_vars,
        &styles.scope_id,
    );
    stage_capture::configure(
        capture.as_deref_mut(),
        is_vapor,
        options.template.ssr,
        codegen_options.source_map,
    );

    let component_name = extract_component_name(filenames.0);
    let prelude = Prelude::new(
        descriptor,
        filenames,
        &options,
        auto_ssr_css_vars,
        &styles,
        &component_name,
        template_syntax,
        &custom_elements,
        &codegen_options,
        experimental_options.self_component,
        nuxt_page_meta,
        type_project,
        pug_source_map,
    );
    let target = Target {
        options: &options,
        is_vapor,
        script_output,
    };
    let outputs = Outputs {
        css: styles.css(),
        warnings,
        macro_artifacts,
    };
    if descriptor.template.is_none() {
        stage_capture::unavailable(capture.as_deref_mut(), "no-template");
    }
    match component_case(descriptor)? {
        ComponentCase::TemplateOnly(template) => {
            template_only::compile(&prelude, &target, template, outputs, capture)
        }
        ComponentCase::Script(script) => {
            script_component::compile(&prelude, &target, script, outputs, capture)
        }
        ComponentCase::Setup(script_setup) => {
            let (sources, runs) = SetupSources::new(&prelude, script_setup);
            let setup_arena = vize_l0::Allocator::default();
            let setup_component::AnalyzedSetup {
                setup,
                source_is_ts,
                mut ctx,
                script_bindings,
                script_tables,
                dependencies,
            } = setup_component::analyze(
                &setup_arena,
                &prelude,
                script_setup,
                &sources.content,
                &options.template,
                is_vapor,
            )?;
            setup_component::emit(
                &prelude,
                &target,
                SetupView {
                    script_setup,
                    sources: &sources,
                    setup: &setup,
                    source_is_ts,
                },
                &mut ctx,
                SetupInputs {
                    outputs,
                    runs,
                    script_bindings,
                    script_tables,
                    dependencies,
                },
                capture,
            )
        }
    }
}

/// Which of the three component shapes an SFC has.
enum ComponentCase<'d> {
    /// A template with no `<script>` of either kind: just a render function.
    TemplateOnly(&'d SfcTemplateBlock<'d>),
    /// A `<script>` without `<script setup>`, with or without a template.
    Script(&'d SfcScriptBlock<'d>),
    /// A `<script setup>`, with an optional `<script>`.
    Setup(&'d SfcScriptBlock<'d>),
}

fn component_case<'d>(descriptor: &'d SfcDescriptor<'d>) -> Result<ComponentCase<'d>, SfcError> {
    match (
        descriptor.template.as_ref(),
        descriptor.script.as_ref(),
        descriptor.script_setup.as_ref(),
    ) {
        (_, _, Some(script_setup)) => Ok(ComponentCase::Setup(script_setup)),
        (_, Some(script), None) => Ok(ComponentCase::Script(script)),
        (Some(template), None, None) => Ok(ComponentCase::TemplateOnly(template)),
        (None, None, None) => Err(empty_component::missing_block()),
    }
}
