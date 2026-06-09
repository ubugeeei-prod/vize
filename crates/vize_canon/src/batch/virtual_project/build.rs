//! Building a [`RegisteredFile`] from a source path. This owns the expensive,
//! `&mut`-free work (SFC/template parse, virtual-TS generation, import
//! rewriting) so it can be fanned out across rayon workers, returning a
//! self-contained result the project absorbs after the join point.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{String as CompactString, ToCompactString, cstr, profile};

use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};

use crate::batch::Diagnostic;
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::import_rewriter::ImportRewriter;
use crate::batch::source_map::{CompositeSourceMap, SfcSourceMap};
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

use super::VirtualFile;
use super::diagnostics::collect_sfc_block_ranges;
use super::passthrough::collect_passthrough_json_modules;
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions, generate_vue_virtual_ts};

/// Result of building a virtual file for a registered path, owned and
/// independent of any `&mut VirtualProject` so it can be produced in parallel.
pub(super) struct RegisteredFile {
    pub(super) file: VirtualFile,
    /// Original source text as registered, retained for offset<->line/col
    /// mapping without a disk re-read. Stored on the project, not the public
    /// `VirtualFile`.
    pub(super) original_content: CompactString,
    pub(super) passthrough_files: Vec<(PathBuf, PathBuf)>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
pub(super) struct VirtualBuildContext<'a> {
    pub(super) project_root: &'a Path,
    pub(super) virtual_root: &'a Path,
    pub(super) virtual_ts_options: &'a VirtualTsOptions,
    pub(super) virtual_ts_check_options: VirtualTsCheckOptions,
    pub(super) preserve_unused_diagnostics: bool,
    pub(super) options_api: bool,
    pub(super) legacy_vue2: bool,
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) rewriter: &'a ImportRewriter,
}

pub(super) fn build_registered_file(
    path: &Path,
    content: &str,
    context: VirtualBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    if path.extension().and_then(|extension| extension.to_str()) == Some("vue") {
        return build_vue_registered_file(path, content, context);
    }

    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
        return build_script_registered_file(
            path,
            content,
            SourceType::ts(),
            context.project_root,
            context.virtual_root,
            context.rewriter,
        );
    }

    let source_type = source_type_for_path(path).ok_or_else(|| CorsaError::PathError {
        path: path.to_path_buf(),
    })?;
    build_script_registered_file(
        path,
        content,
        source_type,
        context.project_root,
        context.virtual_root,
        context.rewriter,
    )
}

pub(super) fn build_vue_registered_file(
    path: &Path,
    content: &str,
    context: VirtualBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    let descriptor = profile!(
        "canon.sfc.parse",
        parse_sfc(
            content,
            SfcParseOptions {
                filename: path.to_string_lossy().to_compact_string(),
                ..Default::default()
            },
        )
        .map_err(|error| CorsaError::SfcParse(error.message.to_compact_string()))
    )?;

    let effective_options =
        virtual_ts_options_for_descriptor(context.virtual_ts_options, &descriptor);
    let generated = profile!(
        "canon.vue.virtual_ts",
        generate_vue_virtual_ts(
            path,
            content,
            &descriptor,
            &effective_options,
            VueCodegenOptions {
                check_options: context.virtual_ts_check_options,
                preserve_unused_diagnostics: context.preserve_unused_diagnostics,
                options_api: context.options_api,
                legacy_vue2: context.legacy_vue2,
                template_syntax: context.template_syntax,
            },
        )
    )?;
    let GeneratedVueFile {
        code,
        mappings,
        diagnostics,
    } = generated;
    let rewritten = profile!(
        "canon.import.rewrite.vue",
        context.rewriter.rewrite(&code, SourceType::ts())
    );
    let source_map = CompositeSourceMap::new_vue(
        SfcSourceMap::new(mappings, collect_sfc_block_ranges(&descriptor)),
        rewritten.source_map,
    );
    let virtual_path = virtual_vue_path(context.project_root, context.virtual_root, path)?;

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map,
            original_path: path.to_path_buf(),
            virtual_path,
        },
        original_content: content.to_compact_string(),
        passthrough_files: collect_passthrough_json_modules(
            path,
            content,
            context.project_root,
            context.virtual_root,
        ),
        diagnostics,
    })
}

pub(super) fn build_script_registered_file(
    path: &Path,
    content: &str,
    source_type: SourceType,
    project_root: &Path,
    virtual_root: &Path,
    rewriter: &ImportRewriter,
) -> CorsaResult<RegisteredFile> {
    let rewritten = profile!(
        "canon.import.rewrite.script",
        rewriter.rewrite(content, source_type)
    );
    let virtual_path = mirrored_virtual_path(project_root, virtual_root, path)?;

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map: CompositeSourceMap::new_script(rewritten.source_map),
            original_path: path.to_path_buf(),
            virtual_path,
        },
        original_content: content.to_compact_string(),
        passthrough_files: collect_passthrough_json_modules(
            path,
            content,
            project_root,
            virtual_root,
        ),
        diagnostics: Vec::new(),
    })
}

fn virtual_ts_options_for_descriptor(
    base: &VirtualTsOptions,
    descriptor: &SfcDescriptor,
) -> VirtualTsOptions {
    // Per-file generation never re-emits the global auto-import stubs inline:
    // they are written once to a shared ambient `.d.ts` (see
    // `write_auto_import_stubs`). Build the per-file options with an empty
    // `auto_import_stubs` instead of deep-cloning the (potentially large,
    // Nuxt/auto-import) global Vec only to clear it again at the call site.
    let css_modules: Vec<CompactString> = descriptor
        .styles
        .iter()
        .filter_map(|style| {
            style
                .module
                .as_ref()
                .map(|module| module.to_compact_string())
        })
        .collect();
    let css_modules = if css_modules.is_empty() {
        // No `<style module>` blocks: reuse the global css_modules (typically
        // also empty) rather than the freshly collected empty Vec.
        base.css_modules.clone()
    } else {
        css_modules
    };

    VirtualTsOptions {
        template_globals: base.template_globals.clone(),
        css_modules,
        auto_import_stubs: Vec::new(),
        external_template_bindings: base.external_template_bindings.clone(),
    }
}

pub(super) fn mirrored_virtual_path(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
) -> CorsaResult<PathBuf> {
    let relative = path.strip_prefix(project_root)?;
    Ok(virtual_root.join(relative))
}

fn virtual_vue_path(project_root: &Path, virtual_root: &Path, path: &Path) -> CorsaResult<PathBuf> {
    let mut virtual_path = mirrored_virtual_path(project_root, virtual_root, path)?;
    let file_name = virtual_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| cstr!("{name}.ts"))
        .ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
    virtual_path.set_file_name(file_name.as_str());
    Ok(virtual_path)
}

pub(super) fn source_type_for_path(path: &Path) -> Option<SourceType> {
    let file_name = path.file_name()?.to_str()?;
    if file_name.ends_with(".tsx") {
        return Some(SourceType::tsx());
    }
    if file_name.ends_with(".ts")
        || file_name.ends_with(".d.ts")
        || file_name.ends_with(".mts")
        || file_name.ends_with(".cts")
    {
        return Some(SourceType::ts());
    }
    None
}
