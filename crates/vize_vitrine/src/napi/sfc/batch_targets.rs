//! One native call that compiles each SFC of a batch for several targets.
//!
//! A build whose client and SSR environments both compile a file used to make
//! one batch call per environment. Each call is its own type-resolution batch,
//! so the TypeScript project was selected and every imported type module was
//! read and resolved once per environment. Here each file is parsed once, its
//! project is selected once and its hashes and blocks are taken once, and every
//! target compiles inside one type-resolution batch, so the targets walk one
//! imported-type graph.
//!
//! The targets differ only in `ssr`, `vapor` and `inlineTemplate`; every other
//! option holds for the whole call. The compiler analyzes each file once for
//! all of its targets and emits per target. Each target's result is the result
//! a separate `compileSfcBatchWithResults` call with that target's options
//! returns, field for field, with one exception: a later target leaves out
//! `dependencies` that are the first target's (see `dependenciesShared`).

use super::codegen_options::sfc_codegen_options;
use napi::{Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::time::Instant;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileTarget, SfcDescriptor, SfcParseOptions,
    SfcScriptOutputMode, StyleCompileOptions, TemplateCompileOptions,
    compile_sfc_for_adapter_targets_with_project, module_shape::finalize_module_output_with_map,
    parse_sfc,
};
use vize_l0::cstr;

use super::batch_helpers::{SfcOutputShape, sfc_output_shape};
use super::types::ModuleShapeNapi;
use super::vue_version::resolve_sfc_vue_version;
use super::{
    experimentals::ExperimentalTemplateOptions,
    thread_pool::BatchThreadPool,
    types::{
        BatchCompileOptionsNapi, BatchCompileResultWithFilesNapi, BatchFileInputNapi,
        BatchFileResultNapi, custom_blocks_to_napi, dependencies_to_napi, macro_artifacts_to_napi,
        style_blocks_to_napi, type_project_to_napi,
    },
};
use crate::template_syntax::resolve_template_syntax;
use crate::whitespace::{ResolvedWhitespace, resolve_whitespace};

/// What one target of a multi-target batch sets. An omitted field keeps the
/// value of the call's options.
#[napi(object)]
#[derive(Default)]
pub struct BatchCompileTargetNapi {
    pub ssr: Option<bool>,
    pub vapor: Option<bool>,
    /// Inline the render function into `setup()` and keep the ES module.
    pub inline_template: Option<bool>,
}

#[napi(object)]
pub struct BatchCompileTargetsResultNapi {
    /// One result per target, in target order. Each lists the files in input
    /// order, and its `timeMs` is the time of the whole call.
    pub targets: Vec<BatchCompileResultWithFilesNapi>,
    /// Per file, in input order: whether every later target compiled the
    /// first target's `dependencies` and so leaves them out. A component typed
    /// through a barrel depends on every module of it, so the list would
    /// otherwise cross the boundary once per target.
    pub dependencies_shared: Vec<bool>,
    pub time_ms: f64,
}

#[napi(js_name = "compileSfcBatchForTargets")]
pub fn compile_sfc_batch_for_targets(
    files: Vec<BatchFileInputNapi>,
    options: Option<BatchCompileOptionsNapi>,
    targets: Vec<BatchCompileTargetNapi>,
) -> Result<BatchCompileTargetsResultNapi> {
    let opts = options.unwrap_or_default();
    if targets.is_empty() {
        return Err(napi::Error::new(
            Status::InvalidArg,
            "compileSfcBatchForTargets needs at least one target",
        ));
    }
    let vue_version = resolve_sfc_vue_version(opts.vue_version.as_deref())?;
    BatchThreadPool::new(opts.threads)?
        .install(|| compile_for_targets(files, &opts, &targets, vue_version))
}

/// The options every file and target of the call share, resolved once.
struct Shared<'a> {
    opts: &'a BatchCompileOptionsNapi,
    vue_version: vize_l0::config::VueVersion,
    type_projects: vize::type_projects::ImportedTypeProjects,
    directive_contracts: vize_l0::config::DirectiveContracts,
    custom_elements: vize_atelier_core::options::CustomElementMatcher,
    experimentals: ExperimentalTemplateOptions,
    template_syntax: vize_atelier_core::TemplateSyntaxMode,
    whitespace: ResolvedWhitespace,
    is_ts: bool,
    /// The call's output shape; a target chooses only its script output.
    shape: SfcOutputShape,
}

fn compile_for_targets(
    files: Vec<BatchFileInputNapi>,
    opts: &BatchCompileOptionsNapi,
    targets: &[BatchCompileTargetNapi],
    vue_version: vize_l0::config::VueVersion,
) -> Result<BatchCompileTargetsResultNapi> {
    let shared = Shared {
        opts,
        vue_version,
        type_projects: vize::type_projects::ImportedTypeProjects::new(
            opts.type_resolution_fallback_tsconfig.as_deref(),
        ),
        directive_contracts: crate::napi::plugin_plan::compile_directives(
            opts.plugin_plan.as_deref(),
        )?,
        custom_elements: vize_atelier_core::options::CustomElementMatcher::from_patterns(
            crate::types::custom_element_patterns(opts.custom_elements.as_deref()),
        ),
        experimentals: ExperimentalTemplateOptions::from_batch(opts),
        template_syntax: resolve_template_syntax(opts.template_syntax.as_deref())
            .map_err(|message| napi::Error::new(Status::InvalidArg, message))?,
        whitespace: resolve_whitespace(opts.whitespace.as_deref())
            .map_err(|message| napi::Error::new(Status::InvalidArg, message))?,
        is_ts: opts.is_ts.unwrap_or(false),
        shape: sfc_output_shape(opts.mode.as_deref(), opts.inline_template, opts.is_prod),
    };
    let targets: Vec<SfcCompileTarget> = targets
        .iter()
        .map(|target| SfcCompileTarget {
            ssr: target.ssr.or(opts.ssr).unwrap_or(false),
            vapor: target.vapor.or(opts.vapor).unwrap_or(false),
            script_output: if sfc_output_shape(
                opts.mode.as_deref(),
                target.inline_template.or(opts.inline_template),
                opts.is_prod,
            )
            .inline_render
            {
                SfcScriptOutputMode::InlineTemplate
            } else {
                SfcScriptOutputMode::SeparateTemplate
            },
        })
        .collect();
    let total_count = files.len();
    let start = Instant::now();
    // One stability window for every target: they share one imported-type
    // graph. The named guard must stay alive until the parallel map has joined.
    #[deny(let_underscore_drop)]
    let _type_resolution_batch = vize_atelier_sfc::join_type_resolution_batch();
    let rows: Vec<(Vec<(BatchFileResultNapi, bool)>, bool)> = files
        .into_par_iter()
        .map(|file| compile_file_for_targets(file, &shared, &targets))
        .collect();
    let time_ms = start.elapsed().as_secs_f64() * 1000.0;

    let mut per_target: Vec<BatchCompileResultWithFilesNapi> = targets
        .iter()
        .map(|_| BatchCompileResultWithFilesNapi {
            results: Vec::with_capacity(total_count),
            success_count: 0,
            failed_count: 0,
            time_ms,
        })
        .collect();
    let mut dependencies_shared = Vec::with_capacity(total_count);
    for (row, shared) in rows {
        dependencies_shared.push(shared);
        for (target, (result, compiled)) in per_target.iter_mut().zip(row) {
            if compiled {
                target.success_count += 1;
            } else {
                target.failed_count += 1;
            }
            target.results.push(result);
        }
    }
    Ok(BatchCompileTargetsResultNapi {
        targets: per_target,
        dependencies_shared,
        time_ms,
    })
}

/// Drop each later target's `dependencies` when they are the first target's,
/// but only when every later target's are, and say whether it did. A later
/// result without a list (a failed compile) is not one that shares it.
pub(super) fn leave_out_shared_dependencies(row: &mut [(BatchFileResultNapi, bool)]) -> bool {
    let Some(((first, _), later)) = row.split_first_mut() else {
        return false;
    };
    if later.is_empty()
        || first.dependencies.is_none()
        || later
            .iter()
            .any(|(result, _)| result.dependencies != first.dependencies)
    {
        return false;
    }
    for (result, _) in later {
        result.dependencies = None;
    }
    true
}

/// The parts of a file's result every target reports alike.
struct FileFacts {
    path: String,
    scope_id: vize_l0::String,
    has_scoped: bool,
    template_hash: Option<String>,
    style_hash: Option<String>,
    script_hash: Option<String>,
}

impl FileFacts {
    fn failure(
        &self,
        message: String,
        descriptor: Option<&SfcDescriptor>,
        opts: &BatchCompileOptionsNapi,
    ) -> BatchFileResultNapi {
        BatchFileResultNapi {
            path: self.path.clone(),
            code: String::new(),
            map: None,
            css: None,
            scope_id: self.scope_id.as_str().into(),
            has_scoped: self.has_scoped,
            errors: vec![message],
            warnings: vec![],
            template_hash: self.template_hash.clone(),
            style_hash: self.style_hash.clone(),
            script_hash: self.script_hash.clone(),
            styles: block_payload(descriptor, opts.include_styles, |d| {
                style_blocks_to_napi(&d.styles)
            }),
            custom_blocks: block_payload(descriptor, opts.include_custom_blocks, |d| {
                custom_blocks_to_napi(&d.custom_blocks)
            }),
            macro_artifacts: vec![],
            dependencies: None,
            module_shape: None,
            type_project: None,
        }
    }
}

fn block_payload<T>(
    descriptor: Option<&SfcDescriptor>,
    include: Option<bool>,
    payload: impl FnOnce(&SfcDescriptor) -> Vec<T>,
) -> Vec<T> {
    match descriptor {
        Some(descriptor) if include.unwrap_or(false) => payload(descriptor),
        _ => vec![],
    }
}

/// Each target's result for `file`, and whether the later targets leave out
/// the first target's `dependencies`.
fn compile_file_for_targets(
    file: BatchFileInputNapi,
    shared: &Shared<'_>,
    targets: &[SfcCompileTarget],
) -> (Vec<(BatchFileResultNapi, bool)>, bool) {
    let opts = shared.opts;
    let scope_id: vize_l0::String = file.scope_id.as_deref().map_or_else(
        || vize_atelier_sfc::generate_bundler_scope_id(&file.path, None, false, None),
        Into::into,
    );
    let filename: vize_l0::CompactString = file.path.as_str().into();
    let mut facts = FileFacts {
        path: file.path,
        scope_id,
        has_scoped: false,
        template_hash: None,
        style_hash: None,
        script_hash: None,
    };
    let descriptor = match parse_sfc(
        &file.source,
        SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
    ) {
        Ok(descriptor) => descriptor,
        Err(error) => {
            let message: String = error.message.into();
            let row = targets
                .iter()
                .map(|_| (facts.failure(message.clone(), None, opts), false))
                .collect();
            return (row, false);
        }
    };
    if opts.include_hashes.unwrap_or(false) {
        facts.template_hash = descriptor.template_hash().map(Into::into);
        facts.style_hash = descriptor.style_hash().map(Into::into);
        facts.script_hash = descriptor.script_hash().map(Into::into);
    }
    facts.has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let type_project = match shared
        .type_projects
        .for_descriptor(&descriptor, std::path::Path::new(&facts.path))
    {
        Ok(type_project) => type_project,
        Err(error) => {
            let message: String = error.message.into();
            let row = targets
                .iter()
                .map(|_| {
                    (
                        facts.failure(message.clone(), Some(&descriptor), opts),
                        false,
                    )
                })
                .collect();
            return (row, false);
        }
    };
    let compiled = shared.whitespace.apply(|| {
        compile_sfc_for_adapter_targets_with_project(
            &descriptor,
            &call_compile_options(shared, &facts, &filename),
            targets,
            shared.template_syntax,
            &shared.custom_elements,
            // A requested map is the compiler's structured SFC module map.
            &sfc_codegen_options(
                opts.include_source_map.unwrap_or(false),
                opts.runtime_module_name.as_deref(),
                opts.runtime_global_name.as_deref(),
            ),
            &shared.experimentals.sfc_options(),
            opts.nuxt_page_meta.unwrap_or(false),
            type_project.as_deref(),
        )
    });
    let analyzed_once = compiled.dependencies_shared;
    let mut row: Vec<(BatchFileResultNapi, bool)> = compiled
        .results
        .into_iter()
        .enumerate()
        .map(|(index, compiled)| match compiled {
            Ok(result) => {
                // A later target compiled the first target's list and
                // leaves it out, so the project is recorded in the first.
                let result = if analyzed_once && index > 0 {
                    result
                } else {
                    type_project.track(result)
                };
                let (code, shape, map) =
                    finalize_module_output_with_map(result.code, shared.is_ts, result.map);
                let file_result = BatchFileResultNapi {
                    path: facts.path.clone(),
                    module_shape: shape.map(ModuleShapeNapi::from),
                    type_project: type_project_to_napi(&type_project),
                    code: code.into(),
                    map: map.and_then(|map| serde_json::to_string(&map).ok()),
                    css: result.css.map(Into::into),
                    scope_id: facts.scope_id.as_str().into(),
                    has_scoped: facts.has_scoped,
                    errors: result
                        .errors
                        .into_iter()
                        .map(|e| e.message.into())
                        .collect(),
                    warnings: result
                        .warnings
                        .into_iter()
                        .map(|e| e.message.into())
                        .collect(),
                    template_hash: facts.template_hash.clone(),
                    style_hash: facts.style_hash.clone(),
                    script_hash: facts.script_hash.clone(),
                    styles: block_payload(Some(&descriptor), opts.include_styles, |d| {
                        style_blocks_to_napi(&d.styles)
                    }),
                    custom_blocks: block_payload(
                        Some(&descriptor),
                        opts.include_custom_blocks,
                        |d| custom_blocks_to_napi(&d.custom_blocks),
                    ),
                    macro_artifacts: if opts.include_macro_artifacts.unwrap_or(false) {
                        macro_artifacts_to_napi(result.macro_artifacts)
                    } else {
                        vec![]
                    },
                    dependencies: dependencies_to_napi(result.dependencies),
                };
                (file_result, true)
            }
            Err(error) => (
                facts.failure(error.message.into(), Some(&descriptor), opts),
                false,
            ),
        })
        .collect();
    let dependencies_shared = if analyzed_once {
        row.len() > 1
            && row
                .first()
                .is_some_and(|(first, _)| first.dependencies.is_some())
    } else {
        leave_out_shared_dependencies(&mut row)
    };
    (row, dependencies_shared)
}

/// The options `compileSfcBatchWithResults` builds for one file; each target
/// sets its own `ssr` and `vapor`.
fn call_compile_options(
    shared: &Shared<'_>,
    facts: &FileFacts,
    filename: &vize_l0::CompactString,
) -> SfcCompileOptions {
    let opts = shared.opts;
    let mut dom_options = shared.experimentals.dom_options();
    dom_options.directive_contracts = shared.directive_contracts.clone();
    let scope_id = &facts.scope_id;
    dom_options.scope_id = facts.has_scoped.then(|| cstr!("data-v-{scope_id}"));
    if let Some(value) = opts.template_cache_handlers {
        dom_options.cache_handlers = value;
    }
    if let Some(value) = opts.template_comments {
        dom_options.comments = value;
    }
    if let Some(value) = opts.template_hoist_static {
        dom_options.hoist_static = value;
    }
    if let Some(value) = opts.template_prefix_identifiers {
        dom_options.prefix_identifiers = value;
    }
    // `parse.filename` is left empty, as in `compileSfcBatchWithResults`:
    // compile falls back to `script.id`, which carries the same value.
    SfcCompileOptions {
        parse: SfcParseOptions::default(),
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            inline_template: shared.shape.standalone,
            is_ts: shared.is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            scoped: facts.has_scoped,
            is_ts: shared.is_ts,
            is_prod: shared.shape.is_prod,
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            dialect: shared.vue_version,
            compiler_options: Some(dom_options),
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: facts.has_scoped,
            trim: opts.style_trim.unwrap_or(false),
            ..Default::default()
        },
        vapor: false,
        scope_id: Some(facts.scope_id.clone()),
    }
}
