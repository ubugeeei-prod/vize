use std::path::Path;

use oxc_span::SourceType;
use vize_carton::{ToCompactString, profile};

use crate::batch::error::CorsaResult;
use crate::batch::import_rewriter::VirtualProjectRewriteOptions;
use crate::batch::source_map::CompositeSourceMap;

use super::super::VirtualFile;
use super::super::esm_declaration_spelling::should_preserve_esm_declaration_spelling;
use super::super::passthrough::collect_passthrough_modules;
use super::{RegisteredFile, ScriptBuildContext};

pub(in crate::batch::virtual_project) fn build_script_registered_file(
    path: &Path,
    content: &str,
    source_type: SourceType,
    context: ScriptBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    let (rewritten, stars) = profile!("canon.import.rewrite.script", {
        context
            .rewriter
            .rewrite_for_virtual_project_with_captured_stars(
                content,
                source_type,
                context.roots,
                path.parent(),
                VirtualProjectRewriteOptions {
                    preserve_relative_declarations: context.preserve_relative_declarations,
                    mirrorable_project_files: (!context.preserve_relative_declarations)
                        .then_some(context.mirrorable_project_files)
                        .flatten(),
                    alias_rewrite_policy: context.alias_rewrite_policy,
                    module_resolver: None,
                },
            )
    });
    let preserve_declaration_spelling = context.preserve_declaration_spelling
        || should_preserve_esm_declaration_spelling(path, content);
    let virtual_path = super::super::paths::script_virtual_path(
        context.roots,
        path,
        content,
        preserve_declaration_spelling,
    )?;

    Ok(RegisteredFile {
        star_projection: super::super::star_exports::StarProjection::from_rewrite(
            &rewritten, stars,
        ),
        file: VirtualFile {
            content: rewritten.code,
            source_map: CompositeSourceMap::new_script(rewritten.source_map),
            original_path: path.to_path_buf(),
            virtual_path,
        },
        extra_virtual_files: Vec::new(),
        original_content: content.to_compact_string(),
        pre_rewrite_code: None,
        passthrough_files: collect_passthrough_modules(
            path,
            content,
            context.roots.0,
            context.roots.1,
        ),
        diagnostics: Vec::new(),
        unchecked_javascript: false,
        typed_router_import: false,
        empty_module_exports: false,
        module_exports: None,
    })
}
