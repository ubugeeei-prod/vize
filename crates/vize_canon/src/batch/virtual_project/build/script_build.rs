use std::borrow::Cow;
use std::path::Path;

use oxc_span::SourceType;
use vize_carton::profile;

use crate::batch::error::CorsaResult;
use crate::batch::import_rewriter::VirtualProjectRewriteOptions;
use crate::batch::source_map::CompositeSourceMap;

use super::super::VirtualFile;
use super::super::esm_declaration_spelling::should_preserve_esm_declaration_spelling;
use super::super::passthrough::collect_passthrough_modules;
use super::{RegisteredFile, ScriptBuildContext, retain_source_content};

pub(in crate::batch::virtual_project) fn build_script_registered_file(
    path: &Path,
    source: Cow<'_, str>,
    source_type: SourceType,
    context: ScriptBuildContext<'_>,
) -> CorsaResult<RegisteredFile> {
    let content = source.as_ref();
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
    let authored_identity = rewritten.source_map.is_empty() && rewritten.code.as_str() == content;
    if authored_identity {
        vize_carton::micro_count!("canon.source.identity_shared.files", 1);
        vize_carton::micro_count!("canon.source.identity_shared.bytes", content.len());
    }

    Ok(RegisteredFile {
        diagnostic_route: Default::default(),
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
        pre_rewrite_code: false,
        authored_vue_ts_specifiers: Default::default(),
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
        original_content: (!authored_identity).then(|| retain_source_content(source)),
    })
}
