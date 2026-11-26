//! One SFC compiled for several targets at once.
//!
//! The targets of one call differ only in `ssr`, `vapor` and the script
//! output mode. The styles, the setup's parse and facts, the macro type
//! information (imported type resolution and the declaration merge) and the
//! macro analysis do not depend on those, so they run once; each target then
//! emits from them. Each target's result equals the result of compiling it on
//! its own.
//!
//! The analysis depends on whether the target renders through Vapor. When
//! the targets of a `<script setup>` disagree on that, each compiles on its
//! own.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_l0::String;
use vize_l1_to_l2::lower::pug::PugSourceMap;

use crate::types::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
};

use super::prelude::{self, Outputs, Prelude, Target};
use super::setup_component::{self, SetupInputs, SetupSources, SetupView};
use super::{
    ComponentCase, SfcCompileTarget, SfcTargetsCompileResult, component_case, empty_component,
    extract_component_name, extract_descriptor_macro_artifacts, script_component, template_only,
    vapor_requested,
};

/// The call's inputs besides the descriptor, its options and the targets.
pub(super) struct TargetsCall<'a> {
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) custom_elements: &'a CustomElementMatcher,
    pub(super) codegen_options: &'a CodegenOptions,
    pub(super) experimental_options: &'a SfcCompileExperimentalOptions,
    pub(super) nuxt_page_meta: bool,
    pub(super) type_project: Option<&'a crate::script::SfcTypeProject>,
    pub(super) pug_source_map: Option<&'a PugSourceMap>,
}

/// `options` with `target`'s choices: what a separate compile receives.
pub(super) fn target_options(
    options: &SfcCompileOptions,
    target: &SfcCompileTarget,
) -> SfcCompileOptions {
    let mut options = options.clone();
    options.template.ssr = target.ssr;
    options.vapor = target.vapor;
    options
}

pub(super) fn compile_sfc_targets_inner(
    descriptor: &SfcDescriptor,
    options: &SfcCompileOptions,
    targets: &[SfcCompileTarget],
    call: &TargetsCall<'_>,
) -> SfcTargetsCompileResult {
    let refused = |error: SfcError| SfcTargetsCompileResult::refused(targets.len(), &error);
    if let Err(error) = empty_component::validate_component(descriptor) {
        return refused(error);
    }
    let mut target_options: Vec<SfcCompileOptions> = targets
        .iter()
        .map(|target| self::target_options(options, target))
        .collect();
    let renders_vapor = |(options, target): (&SfcCompileOptions, &SfcCompileTarget)| {
        !target.ssr && vapor_requested(descriptor, options)
    };
    let mut vapor = target_options.iter().zip(targets).map(renders_vapor);
    let first_vapor = vapor.next().unwrap_or_default();
    if descriptor.script_setup.is_some() && vapor.any(|is_vapor| is_vapor != first_vapor) {
        return apart(descriptor, target_options, targets, call);
    }

    let mut warnings = Vec::new();
    let macro_artifacts = extract_descriptor_macro_artifacts(descriptor, call.nuxt_page_meta);
    let filenames = prelude::filenames(&options.parse.filename, options.script.id.as_deref());
    let styles = prelude::Styles::compile(descriptor, options, filenames.0, &mut warnings);
    let auto_ssr_css_vars = options.template.ssr_css_vars.is_none();
    let choices: Vec<_> = target_options
        .iter_mut()
        .zip(targets)
        .map(|(options, target)| {
            let requested = vapor_requested(descriptor, options);
            prelude::choose_target(
                descriptor,
                &mut options.template,
                requested,
                target.script_output,
                auto_ssr_css_vars,
                &styles.scope_id,
            )
        })
        .collect();
    let component_name = extract_component_name(filenames.0);
    let prelude = Prelude::new(
        descriptor,
        filenames,
        options,
        auto_ssr_css_vars,
        &styles,
        &component_name,
        call.template_syntax,
        call.custom_elements,
        call.codegen_options,
        call.experimental_options.self_component,
        call.nuxt_page_meta,
        call.type_project,
        call.pug_source_map,
    );
    let views = target_options
        .iter()
        .zip(choices)
        .map(|(options, (is_vapor, script_output))| Target {
            options,
            is_vapor,
            script_output,
        });
    let last_index = target_options.len().saturating_sub(1);
    let mut outputs = Outputs {
        css: styles.css(),
        warnings,
        macro_artifacts,
    };
    let case = match component_case(descriptor) {
        Ok(case) => case,
        Err(error) => return refused(error),
    };
    let (results, dependencies) = match case {
        ComponentCase::TemplateOnly(template) => (
            views
                .enumerate()
                .map(|(index, target)| {
                    let outputs = outputs.for_target(index == last_index);
                    template_only::compile(&prelude, &target, template, outputs, None)
                })
                .collect(),
            Vec::new(),
        ),
        ComponentCase::Script(script) => (
            views
                .enumerate()
                .map(|(index, target)| {
                    let outputs = outputs.for_target(index == last_index);
                    script_component::compile(&prelude, &target, script, outputs, None)
                })
                .collect(),
            Vec::new(),
        ),
        ComponentCase::Setup(script_setup) => {
            let (sources, mut runs) = SetupSources::new(&prelude, script_setup);
            let setup_arena = vize_l0::Allocator::default();
            // The setup facts read only call-level template options.
            let analyzed = setup_component::analyze(
                &setup_arena,
                &prelude,
                script_setup,
                &sources.content,
                &options.template,
                first_vapor,
            );
            let mut analyzed = match analyzed {
                Ok(analyzed) => analyzed,
                Err(error) => return refused(error),
            };
            let mut results = Vec::with_capacity(last_index + 1);
            for (index, target) in views.enumerate() {
                let last = index == last_index;
                let inputs = SetupInputs {
                    outputs: outputs.for_target(last),
                    runs: take_or_clone(&mut runs, last),
                    script_bindings: take_or_clone(&mut analyzed.script_bindings, last),
                    script_tables: take_or_clone(&mut analyzed.script_tables, last),
                    // Every target reads the same imported types; the list is
                    // placed once every target has compiled.
                    dependencies: Vec::new(),
                };
                let view = SetupView {
                    script_setup,
                    sources: &sources,
                    setup: &analyzed.setup,
                    source_is_ts: analyzed.source_is_ts,
                };
                let ctx = &mut analyzed.ctx;
                results.push(if last {
                    setup_component::emit(&prelude, &target, view, ctx, inputs, None)
                } else {
                    // A target changes only the macros and bindings of the
                    // context; the next one starts from the analyzed state.
                    let saved = (ctx.macros.clone(), ctx.bindings.clone());
                    let result = setup_component::emit(&prelude, &target, view, ctx, inputs, None);
                    (ctx.macros, ctx.bindings) = saved;
                    result
                });
            }
            (results, std::mem::take(&mut analyzed.dependencies))
        }
    };
    SfcTargetsCompileResult::with_dependencies(results, dependencies)
}

fn take_or_clone<T: Clone + Default>(value: &mut T, last: bool) -> T {
    if last {
        std::mem::take(value)
    } else {
        value.clone()
    }
}

/// Each target compiled on its own.
fn apart(
    descriptor: &SfcDescriptor,
    target_options: Vec<SfcCompileOptions>,
    targets: &[SfcCompileTarget],
    call: &TargetsCall<'_>,
) -> SfcTargetsCompileResult {
    let results = target_options
        .into_iter()
        .zip(targets)
        .map(|(options, target)| {
            super::compile_sfc_inner(
                descriptor,
                options,
                call.template_syntax,
                call.custom_elements.clone(),
                call.codegen_options.clone(),
                target.script_output,
                call.experimental_options.clone(),
                call.nuxt_page_meta,
                call.type_project,
                call.pug_source_map,
                None,
            )
        })
        .collect();
    SfcTargetsCompileResult {
        results,
        dependencies_shared: false,
    }
}

impl SfcTargetsCompileResult {
    fn refused(count: usize, error: &SfcError) -> Self {
        Self {
            results: (0..count).map(|_| Err(error.clone())).collect(),
            dependencies_shared: false,
        }
    }

    /// Place the dependency list every target compiled: once, on the first
    /// result, when every target compiled; otherwise on each compiled one.
    fn with_dependencies(
        mut results: Vec<Result<SfcCompileResult, SfcError>>,
        dependencies: Vec<String>,
    ) -> Self {
        let dependencies_shared = results.iter().all(Result::is_ok);
        if dependencies_shared {
            if let Some(Ok(first)) = results.first_mut() {
                first.dependencies = dependencies;
            }
        } else {
            for result in results.iter_mut().flatten() {
                result.dependencies.clone_from(&dependencies);
            }
        }
        Self {
            results,
            dependencies_shared,
        }
    }
}
