//! Builds an owned [`RegisteredFile`] so SFC/template parsing and virtual-TS generation can run
//! across rayon workers without shared `&mut` state.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{String as CompactString, ToCompactString, cstr, profile};

use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};

use crate::batch::Diagnostic;
use crate::batch::declaration_path::is_declaration_file;
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::import_rewriter::ImportRewriter;
use crate::batch::source_map::{CompositeSourceMap, SfcSourceMap};
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions, VizeMapping};

use super::VirtualFile;
use super::diagnostics::collect_sfc_block_ranges;
use super::jsx_build::build_jsx_registered_file;
use super::passthrough::collect_passthrough_modules;
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions, generate_vue_virtual_ts};

pub(super) const VUE_JSX_REFERENCE_DIRECTIVE: &str = "/// <reference types=\"vue/jsx\" />\n";

/// Result of building a virtual file for a registered path, owned and
/// independent of any `&mut VirtualProject` so it can be produced in parallel.
pub(super) struct RegisteredFile {
    pub(super) file: VirtualFile,
    pub(super) extra_virtual_files: Vec<VirtualFile>,
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
    pub(super) experimental_in_tag_comments: bool,
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

    if is_declaration_file(path) {
        return build_script_registered_file(
            path,
            content,
            SourceType::ts(),
            (context.project_root, context.virtual_root),
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
        (context.project_root, context.virtual_root),
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
    let source_type = if use_tsx_virtual {
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
                experimental_in_tag_comments: context.experimental_in_tag_comments,
                hoist_shared_preamble: true,
                omit_vite_client_reference: false,
            },
        )
    )?;
    let GeneratedVueFile {
        mut code,
        mut mappings,
        diagnostics,
    } = generated;
    if use_tsx_virtual {
        prepend_vue_jsx_reference(&mut code, &mut mappings);
    }
    let rewritten = profile!(
        "canon.import.rewrite.vue",
        context.rewriter.rewrite(&code, source_type, path.parent())
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
    let extra_virtual_files = if use_tsx_virtual {
        vec![build_tsx_vue_import_shim(
            context.project_root,
            context.virtual_root,
            path,
            &virtual_path,
        )?]
    } else {
        Vec::new()
    };

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map,
            original_path: path.to_path_buf(),
            virtual_path,
        },
        extra_virtual_files,
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

pub(super) fn build_script_registered_file(
    path: &Path,
    content: &str,
    source_type: SourceType,
    roots: (&Path, &Path),
    rewriter: &ImportRewriter,
) -> CorsaResult<RegisteredFile> {
    let rewritten = profile!(
        "canon.import.rewrite.script",
        rewriter.rewrite_for_virtual_project(content, source_type, roots, path.parent())
    );
    let virtual_path = super::paths::script_virtual_path(roots.0, roots.1, path)?;

    Ok(RegisteredFile {
        file: VirtualFile {
            content: rewritten.code,
            source_map: CompositeSourceMap::new_script(rewritten.source_map),
            original_path: path.to_path_buf(),
            virtual_path,
        },
        extra_virtual_files: Vec::new(),
        original_content: content.to_compact_string(),
        passthrough_files: collect_passthrough_modules(path, content, roots.0, roots.1),
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
        // No `<style module>` blocks: reuse the global css_modules.
        base.css_modules.clone()
    } else {
        css_modules
    };

    VirtualTsOptions {
        template_globals: base.template_globals.clone(),
        css_modules,
        auto_import_stubs: Vec::new(),
        external_template_bindings: base.external_template_bindings.clone(),
        reference_paths: base.reference_paths.clone(),
    }
}

pub(super) fn prepend_vue_jsx_reference(code: &mut CompactString, mappings: &mut [VizeMapping]) {
    if code.starts_with(VUE_JSX_REFERENCE_DIRECTIVE) {
        return;
    }
    let offset = VUE_JSX_REFERENCE_DIRECTIVE.len();
    let mut prefixed = CompactString::with_capacity(offset + code.len());
    prefixed.push_str(VUE_JSX_REFERENCE_DIRECTIVE);
    prefixed.push_str(code.as_str());
    *code = prefixed;
    for mapping in mappings {
        mapping.gen_range.start += offset;
        mapping.gen_range.end += offset;
        for sub_span in &mut mapping.sub_spans {
            sub_span.gen_range.start += offset;
            sub_span.gen_range.end += offset;
        }
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

fn build_tsx_vue_import_shim(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
    target_path: &Path,
) -> CorsaResult<VirtualFile> {
    let shim_path = virtual_vue_path(project_root, virtual_root, path, false)?;
    let target_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
    let content = cstr!(
        "export {{ default }} from \"./{target_name}\";\nexport * from \"./{target_name}\";\n"
    );

    Ok(VirtualFile {
        content,
        source_map: CompositeSourceMap::empty(),
        original_path: shim_path.clone(),
        virtual_path: shim_path,
    })
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
