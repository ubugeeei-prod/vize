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
use super::passthrough::collect_passthrough_modules;
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
    /// Opt-in type-checking of `.jsx`/`.tsx` Vue components (#1497).
    /// Otherwise JSX/TSX files pass through to TypeScript verbatim.
    pub(super) jsx_typecheck: bool,
    pub(super) dialect: vize_carton::config::VueVersion,
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

    // Opt-in Vize JSX/TSX type-checking (#1497). Only when the user explicitly
    // enabled `typeChecker.jsxTypecheck`: otherwise `.jsx`/`.tsx` is passed to
    // TypeScript verbatim (React passthrough) by the script path below.
    if context.jsx_typecheck
        && let Some(name) = path.file_name().and_then(|name| name.to_str())
        && (name.ends_with(".jsx") || name.ends_with(".tsx"))
    {
        return build_jsx_registered_file(path, content, context);
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
    let use_tsx_virtual = descriptor_uses_jsx_script(&descriptor);
    let virtual_source_type = if use_tsx_virtual {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
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
                dialect: context.dialect,
                template_syntax: context.template_syntax,
                hoist_shared_preamble: true,
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
        context.rewriter.rewrite(&code, virtual_source_type)
    );
    let source_map = CompositeSourceMap::new_vue(
        SfcSourceMap::new(mappings, collect_sfc_block_ranges(&descriptor)),
        rewritten.source_map,
    );
    let virtual_path = virtual_vue_path(
        context.project_root,
        context.virtual_root,
        path,
        use_tsx_virtual,
    )?;

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map,
            original_path: path.to_path_buf(),
            virtual_path,
        },
        original_content: content.to_compact_string(),
        passthrough_files: collect_passthrough_modules(
            path,
            content,
            context.project_root,
            context.virtual_root,
        ),
        diagnostics,
    })
}

/// Build a virtual file for a `.jsx`/`.tsx` Vize component (#1497, opt-in).
///
/// Parallels [`build_vue_registered_file`]: lower the JSX/TSX to plain virtual
/// TypeScript (props from the typed first parameter + setup scope + JSX
/// expressions), rewrite imports, and mirror the file to `<name>.ts` so Corsa
/// type-checks it as plain TypeScript. Reached only when `jsx_typecheck` is on.
pub(super) fn build_jsx_registered_file(
    path: &Path,
    content: &str,
    context: VirtualBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    let lang = jsx_lang_for_path(path);
    let generated = profile!(
        "canon.jsx.virtual_ts",
        super::jsx_codegen::generate_jsx_virtual_ts(path, content, lang)
    )?;
    let super::jsx_codegen::GeneratedJsxFile {
        code,
        mappings,
        diagnostics,
    } = generated;

    let rewritten = profile!(
        "canon.import.rewrite.jsx",
        context.rewriter.rewrite(&code, SourceType::ts())
    );

    // The whole `.jsx`/`.tsx` body is one Script block for block-type recovery.
    let blocks = vec![crate::batch::source_map::SfcBlockRange {
        start: 0,
        end: content.len() as u32,
        block_type: crate::batch::SfcBlockType::Script,
    }];
    let source_map =
        CompositeSourceMap::new_vue(SfcSourceMap::new(mappings, blocks), rewritten.source_map);
    let virtual_path = virtual_jsx_path(context.project_root, context.virtual_root, path)?;

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map,
            original_path: path.to_path_buf(),
            virtual_path,
        },
        original_content: content.to_compact_string(),
        passthrough_files: collect_passthrough_modules(
            path,
            content,
            context.project_root,
            context.virtual_root,
        ),
        diagnostics,
    })
}

fn jsx_lang_for_path(path: &Path) -> vize_atelier_jsx::JsxLang {
    let is_tsx = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".tsx"));
    if is_tsx {
        vize_atelier_jsx::JsxLang::Tsx
    } else {
        vize_atelier_jsx::JsxLang::Jsx
    }
}

fn virtual_jsx_path(project_root: &Path, virtual_root: &Path, path: &Path) -> CorsaResult<PathBuf> {
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
        rewriter.rewrite_for_virtual_project(content, source_type, (project_root, virtual_root))
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
        passthrough_files: collect_passthrough_modules(path, content, project_root, virtual_root),
        diagnostics: Vec::new(),
    })
}

pub(super) fn virtual_ts_options_for_descriptor(
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

fn virtual_vue_path(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
    use_tsx_virtual: bool,
) -> CorsaResult<PathBuf> {
    let mut virtual_path = mirrored_virtual_path(project_root, virtual_root, path)?;
    let file_name = virtual_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            if use_tsx_virtual {
                cstr!("{name}.tsx")
            } else {
                cstr!("{name}.ts")
            }
        })
        .ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
    virtual_path.set_file_name(file_name.as_str());
    Ok(virtual_path)
}

pub(super) fn descriptor_uses_jsx_script(descriptor: &SfcDescriptor) -> bool {
    descriptor
        .script
        .as_ref()
        .and_then(|script| script.lang.as_deref())
        .is_some_and(is_jsx_like_lang)
        || descriptor
            .script_setup
            .as_ref()
            .and_then(|script| script.lang.as_deref())
            .is_some_and(is_jsx_like_lang)
}

fn is_jsx_like_lang(lang: &str) -> bool {
    matches!(lang, "jsx" | "tsx")
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
