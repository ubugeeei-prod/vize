//! Builds an owned [`RegisteredFile`] so SFC/template parsing and virtual-TS generation can run
//! across rayon workers without shared `&mut` state.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{String as CompactString, ToCompactString, cstr, profile};

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

use crate::batch::Diagnostic;
use crate::batch::declaration_path::is_declaration_file;
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::source_map::{CompositeSourceMap, SfcSourceMap};
use crate::virtual_ts::VizeMapping;

mod context;
mod css_modules;
mod script_build;
pub(super) use super::paths::source_type_for_path;
pub(super) use context::{ScriptBuildContext, VirtualBuildContext};
pub(super) use css_modules::{
    css_module_block_type, style_scoped_class_names, virtual_ts_options_for_descriptor,
};
pub(super) use script_build::build_script_registered_file;

use super::VirtualFile;
use super::diagnostics::collect_sfc_block_ranges;
use super::javascript_sfc::{descriptor_is_javascript, descriptor_is_unchecked_javascript};
pub(super) use super::javascript_sfc::{descriptor_uses_jsx_script, prepare_script_blocks};
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
    pub(super) pre_rewrite_code: Option<CompactString>,
    pub(super) passthrough_files: Vec<(PathBuf, PathBuf)>,
    pub(super) diagnostics: Vec<Diagnostic>,
    /// SFC whose script block is JavaScript: TypeScript diagnostics on it are
    /// reportable only under `checkJs` (see `javascript_sfc`, #3322).
    pub(super) unchecked_javascript: bool,
    pub(super) typed_router_import: bool,
    pub(super) empty_module_exports: bool,
    pub(super) module_exports: Option<Box<[vize_croquis::script_parser::AuthoredModuleExport]>>,
    pub(super) star_projection: Option<super::star_exports::StarProjection>,
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
            context.script_context(),
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
    build_script_registered_file(path, content, source_type, context.script_context())
}

pub(super) fn build_vue_registered_file(
    path: &Path,
    authored_content: &str,
    context: VirtualBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    let authored = crate::authored_sfc::AuthoredSfc::new(authored_content);
    let content = authored.analyzed();
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
    let module_sources_known =
        vize_atelier_sfc::croquis::descriptor_module_sources_are_known(&descriptor);
    let descriptor = prepare_script_blocks(descriptor);

    let use_tsx_virtual = descriptor_uses_jsx_script(&descriptor);
    // Options API codegen gives a normal script a typed component surface,
    // including the normal block in a mixed normal/setup SFC. Match the
    // generator's Vue 2 mode; a setup-only JS block retains allowJs membership.
    let typed_options_projection = descriptor.script.is_some()
        && (context.options_api
            || context.legacy_vue2
            || matches!(
                context.dialect,
                vize_carton::config::VueVersion::V2 | vize_carton::config::VueVersion::V2_7
            ));
    let denied_javascript = context.editor_document_options.is_none()
        && !context.allow_javascript
        && !typed_options_projection
        && descriptor_is_javascript(&descriptor)
        && (context.javascript_explicitly_denied
            || descriptor_is_unchecked_javascript(&descriptor));
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
            context.virtual_ts_options,
            VueCodegenOptions {
                check_options: context.virtual_ts_check_options,
                typed_router_root: context.typed_router_root,
                preserve_unused_diagnostics: context.preserve_unused_diagnostics,
                options_api: context.options_api,
                // Batch check/declaration codegen must keep the authored
                // default export so Options API instance members survive
                // `InstanceType<typeof Component>` (#4010).
                preserve_authored_component: context.editor_document_options.is_none(),
                preserve_script_on_template_error: context.editor_document_options.is_some(),
                component_name: None,
                external_component_attrs_module: None,
                preserve_event_navigation: context
                    .editor_document_options
                    .is_some_and(|options| options.preserve_event_navigation),
                legacy_vue2: context.legacy_vue2,
                dialect: context.dialect,
                template_syntax: context.template_syntax,
                experimental_in_tag_comments: context.experimental_in_tag_comments,
                experimental_patterned_template: context.experimental_patterned_template,
                experimental_strict_slot_children: context.experimental_strict_slot_children,
                hoist_shared_preamble: context.hoist_shared_preamble,
                inline_auto_import_stubs: false,
                omit_vite_client_reference: false,
                runtime_prop_resolve_cache: context.runtime_prop_resolve_cache,
            },
        )
    )?;
    let GeneratedVueFile {
        mut code,
        mut mappings,
        mapping_meta,
        mut semantic_links,
        mut prop_default_key_links,
        mut diagnostics,
        typed_router_import,
        empty_module_exports,
        module_exports,
    } = generated;
    if use_tsx_virtual {
        prepend_vue_jsx_reference(
            &mut code,
            &mut mappings,
            &mut semantic_links,
            &mut prop_default_key_links,
        );
    }
    authored.remap_mappings(&mut mappings);
    for diagnostic in &mut diagnostics {
        (diagnostic.line, diagnostic.column) =
            authored.to_authored_line_column(diagnostic.line, diagnostic.column);
    }
    for error in authored.preprocessing_errors() {
        let (line, column) =
            vize_l0::line_index::offset_to_line_col(authored_content, error.start as usize);
        diagnostics.push(Diagnostic {
            file: path.to_path_buf(),
            line,
            column,
            message: error.message.into(),
            code: None,
            severity: 1,
            block_type: Some(crate::batch::SfcBlockType::Template),
        });
    }
    let (rewritten, stars) = profile!(
        "canon.import.rewrite.vue",
        context
            .rewriter
            .rewrite_generated_for_virtual_project_with_captured_stars(
                &code,
                source_type,
                (context.project_root, context.virtual_root),
                path.parent(),
                crate::batch::import_rewriter::VirtualProjectRewriteOptions {
                    preserve_relative_declarations: context.preserve_relative_declarations,
                    mirrorable_project_files: context.mirrorable_project_files,
                    alias_rewrite_policy: context.alias_rewrite_policy,
                    module_resolver: None,
                },
            )
    );
    let star_projection = super::star_exports::StarProjection::from_rewrite(&rewritten, stars);
    let preserve_module_source = !context.allow_javascript
        && rewritten.source_map != crate::batch::import_rewriter::ImportSourceMap::empty();
    let mut projection = crate::virtual_ts::ProjectionMapping::from_parts(mappings, semantic_links);
    projection.restore_row_metadata(mapping_meta);
    projection.set_prop_default_key_links(prop_default_key_links);
    let source_map = CompositeSourceMap::new_vue(
        SfcSourceMap::from_projection(
            projection,
            authored.remap_block_ranges(collect_sfc_block_ranges(&descriptor)),
        ),
        rewritten.source_map,
    );
    let virtual_path = virtual_vue_path(
        context.project_root,
        context.virtual_root,
        path,
        use_tsx_virtual,
        denied_javascript,
    )?;
    let extra_virtual_files = if use_tsx_virtual && !denied_javascript {
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
        star_projection,
        file: VirtualFile {
            content: rewritten.code,
            source_map,
            original_path: path.to_path_buf(),
            virtual_path,
        },
        extra_virtual_files,
        original_content: authored_content.to_compact_string(),
        pre_rewrite_code: (context.editor_document_options.is_some()
            || context.jsx_typecheck
            || preserve_module_source)
            .then_some(code),
        passthrough_files: collect_passthrough_modules(
            path,
            authored_content,
            context.project_root,
            context.virtual_root,
        ),
        diagnostics,
        unchecked_javascript: denied_javascript || descriptor_is_unchecked_javascript(&descriptor),
        typed_router_import,
        empty_module_exports: empty_module_exports && module_sources_known,
        module_exports: module_exports.filter(|_| module_sources_known),
    })
}

pub(super) fn prepend_vue_jsx_reference(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [crate::virtual_ts::VizeSemanticLink],
    prop_default_key_links: &mut [crate::virtual_ts::VizeSemanticLink],
) {
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
    for link in semantic_links.iter_mut().chain(prop_default_key_links) {
        link.shift_generated_ranges(offset);
    }
}

pub(super) fn mirrored_virtual_path(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
) -> CorsaResult<PathBuf> {
    let canonical = vize_carton::path::canonicalize_non_verbatim(path);
    if let Ok(relative) = canonical.strip_prefix(project_root) {
        return Ok(virtual_root.join(relative));
    }
    // Out-of-root files land in the external escape subtree (#3887).
    super::external_mirror::external_mirror_path(virtual_root, &canonical)
}

pub(super) fn build_tsx_vue_import_shim(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
    target_path: &Path,
) -> CorsaResult<VirtualFile> {
    let shim_path = virtual_vue_path(project_root, virtual_root, path, false, false)?;
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
    denied_javascript: bool,
) -> CorsaResult<PathBuf> {
    let mut virtual_path = mirrored_virtual_path(project_root, virtual_root, path)?;
    let file_name = virtual_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            if denied_javascript {
                let extension = if use_tsx_virtual { "tsx" } else { "ts" };
                cstr!("{name}.__vize_javascript__.{extension}")
            } else if use_tsx_virtual {
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
