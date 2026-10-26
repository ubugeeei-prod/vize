//! Generating virtual TypeScript for `.vue` SFCs: parsing the template, running
//! Croquis analysis, augmenting type-based props, and emitting the `.vue.ts`
//! source consumed by Corsa. Incomplete scripts retain their authored code and
//! mappings; batch or unrecoverable SFC structure uses a typed fallback module.

use std::path::Path;
use vize_carton::config::VueVersion;
use vize_carton::{Allocator, profile};

use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{
    SfcDescriptor,
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_module_exports},
};

use crate::batch::SfcBlockType;
use crate::batch::error::CorsaResult;
use crate::virtual_ts::{
    AuthoredScriptMetadata, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};

use super::diagnostics::{
    collect_script_parse_fallbacks, collect_sfc_compile_diagnostic, diagnostic_for_offset,
    invalid_sfc_fallback_virtual_ts,
};
use super::{
    art_usage::{art_variant_check_source, collect_art_template_referenced_names},
    css_var_usage::collect_css_var_referenced_names,
    setup_props::{RuntimePropResolveCache, augment_type_based_props_from_script_context},
};

pub(super) mod generic_attribute;
mod generic_syntax;
mod style_modules;
mod template;
mod types;
pub(super) use types::{GeneratedVueFile, VueCodegenOptions};
#[cfg(test)]
mod editor_script_tests;
#[cfg(test)]
mod generic_component_tests;
#[cfg(test)]
mod malformed_setup_tests;

pub(super) fn generate_vue_virtual_ts(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
    options: &VirtualTsOptions,
    mut codegen_options: VueCodegenOptions<'_>,
) -> CorsaResult<GeneratedVueFile> {
    crate::virtual_ts::validate_reference_paths(&options.reference_paths).map_err(|error| {
        crate::batch::error::CorsaError::Configuration(vize_carton::cstr!(
            "Invalid declaration reference path {:?}: {}",
            error.path,
            error.reason
        ))
    })?;
    codegen_options.check_options =
        super::vue_compiler_comments::apply(source, codegen_options.check_options);
    // All consumers resolve per-SFC metadata after project and top-level options.
    let mut descriptor_options = super::build::virtual_ts_options_for_descriptor(
        options,
        descriptor,
        codegen_options.check_options.strict_css_modules,
        codegen_options.check_options.resolve_style_imports,
    );
    if codegen_options.inline_auto_import_stubs {
        descriptor_options
            .auto_import_stubs
            .clone_from(&options.auto_import_stubs);
    }
    let options = &descriptor_options;
    let allocator = Allocator::new();
    let mut diagnostics = Vec::new();
    let generic_parameters = descriptor
        .script_setup
        .as_ref()
        .and_then(|block| block.attrs.get("generic"))
        .map(|value| value.as_ref())
        .filter(|value: &&str| !value.trim().is_empty())
        .map(crate::virtual_ts::generic_parameters::GenericParameters::parse);
    let diagnostic_route = if generic_parameters
        .as_ref()
        .is_some_and(|plan| !plan.diagnostics.is_empty())
    {
        super::DiagnosticRoute::ProjectSessionRequired
    } else {
        super::DiagnosticRoute::NativeCliAllowed
    };

    if generic_parameters.as_ref().is_some_and(|plan| {
        !plan.diagnostics.is_empty()
            && (plan.escaped_wrapper || !plan.recoverable_at_closing_delimiter)
    }) {
        return Ok(generic_syntax::refuse(path, source, descriptor));
    }

    let view = match vize_atelier_sfc::prepare_root_patterned_template(
        descriptor,
        codegen_options.experimental_patterned_template,
        codegen_options.experimental_in_tag_comments,
        codegen_options.template_syntax,
    ) {
        Ok(view) => view,
        Err(error) => {
            return Ok(GeneratedVueFile {
                diagnostic_route,
                generic_copies: None,
                typed_router_import: false,
                empty_module_exports: false,
                module_exports: None,
                code: invalid_sfc_fallback_virtual_ts(),
                mappings: Vec::new(),
                mapping_meta: Vec::new(),
                semantic_links: Vec::new(),
                prop_default_key_links: Vec::new(),
                diagnostics: vec![diagnostic_for_offset(
                    path,
                    source,
                    error.loc.as_ref().map_or(0, |loc| loc.start as u32),
                    error.message,
                    SfcBlockType::Template,
                )],
            });
        }
    };
    let descriptor = &*view;

    // `<variant>` markup is checked through the same template AST. The pad keeps
    // every checked byte at its file offset, so the template offset stays 0.
    let art_pad = art_variant_check_source(source, descriptor);
    let template_offset = if art_pad.is_some() {
        0
    } else {
        descriptor
            .template
            .as_ref()
            .map(|template| template.loc.start as u32)
            .unwrap_or(0)
    };
    let template_text = match art_pad.as_deref() {
        Some(pad) => Some(pad),
        None => descriptor
            .template
            .as_ref()
            .map(|template| template.content.as_ref()),
    };
    // Keep authored scripts even when OXC cannot recover their analysis AST.
    // The native TypeScript parser recovers incomplete expressions and can
    // still answer editor requests against their exact source mappings.
    // Track whether the template produced any *hard* parse error. Only hard
    // errors omit the template AST; batch projections use the fallback stub.
    // Recovery-level diagnostics keep the real virtual TS:
    //   - `ErrorCode::ExtendPoint`, pushed by the HTML tree-construction
    //     recovery path for self-closing rewrites, fostered elements,
    //     auto-closed `<p>`, etc. (#1065/#1090 regression);
    //   - every code in `RECOVERED_PARSE_CODES`, for which the parser
    //     documents a concrete recovery and still yields a complete tree.
    //     Treating those as hard meant a single missing space between
    //     attributes collapsed the file to the stub and silenced every script
    //     type diagnostic in it (#3323) — the same false-negative shape #3294
    //     fixed in the linter, keyed off the same shared classification.
    let (template_ast, template_hard_error, incomplete_tag) = template_text
        .map(|template_content| {
            template::parse(
                &allocator,
                template_content,
                template_offset,
                path,
                source,
                codegen_options,
                &mut diagnostics,
            )
        })
        .unwrap_or((None, false, false));

    let has_script_projection = !template_hard_error
        || (codegen_options.preserve_script_on_template_error
            && incomplete_tag
            && (descriptor.script.is_some() || descriptor.script_setup.is_some()));
    let script_diagnostics =
        collect_script_parse_fallbacks(path, source, descriptor, has_script_projection);
    diagnostics.extend(script_diagnostics.diagnostics);

    // Batch projections keep the fallback. Editor and Content Mapper projections
    // retain authored scripts only for unfinished tags while the unusable template
    // AST remains absent; native TypeScript owns syntax diagnostics for that script.
    if !has_script_projection {
        return Ok(GeneratedVueFile {
            diagnostic_route,
            generic_copies: None,
            typed_router_import: false,
            empty_module_exports: false,
            module_exports: None,
            code: invalid_sfc_fallback_virtual_ts(),
            mappings: Vec::new(),
            mapping_meta: Vec::new(),
            semantic_links: Vec::new(),
            prop_default_key_links: Vec::new(),
            diagnostics,
        });
    }

    let mut croquis_options = SfcCroquisOptions::full();
    croquis_options
        .analyzer_options
        .experimental_patterned_template = codegen_options.experimental_patterned_template;
    let vue2_compat = codegen_options.legacy_vue2
        || matches!(codegen_options.dialect, VueVersion::V2 | VueVersion::V2_7);

    let (analysis, module_exports) = profile!(
        "canon.croquis.analyze_sfc",
        analyze_sfc_descriptor_with_module_exports(
            descriptor,
            template_ast.as_ref(),
            croquis_options,
            codegen_options.options_api,
            vue2_compat,
        )
    );
    let split_script_setup_offsets = analysis.split_script_setup_offsets(descriptor);
    let mut croquis = analysis.croquis;
    if !codegen_options.experimental_patterned_template {
        croquis
            .pattern_diagnostics
            .extend(crate::sfc_typecheck::patterns::disabled_diagnostics(
                template_ast.as_ref(),
            ));
    }
    for pattern in &croquis.pattern_diagnostics {
        let mut diagnostic = diagnostic_for_offset(
            path,
            source,
            template_offset + pattern.start,
            pattern.message.as_str().into(),
            SfcBlockType::Template,
        );
        diagnostic.severity = if pattern.warning { 2 } else { 1 };
        diagnostics.push(diagnostic);
    }
    let typed_router = super::typed_router::Generation::new(
        codegen_options.typed_router_root,
        path,
        options,
        codegen_options.typed_router_root.is_some()
            && template_ast.as_ref().is_some_and(|root| {
                vize_atelier_sfc::script::resolve_template_used_identifiers(root)
                    .used_ids
                    .contains("$route")
            }),
    );
    let options = typed_router.options(options);
    let script_content = analysis.script_content;
    let script_offset = analysis.script_offset;
    let local_runtime_prop_resolve_cache;
    let runtime_prop_resolve_cache = if let Some(cache) = codegen_options.runtime_prop_resolve_cache
    {
        cache
    } else {
        local_runtime_prop_resolve_cache = RuntimePropResolveCache::default();
        &local_runtime_prop_resolve_cache
    };
    profile!(
        "canon.croquis.augment_type_props",
        augment_type_based_props_from_script_context(
            &mut croquis,
            descriptor,
            path,
            runtime_prop_resolve_cache
        )
    );
    // Names the generated component surface consumes outside `<template>`:
    // `<art>` variant templates and CSS `v-bind()` expressions. Both feed the
    // unused-binding anchors, which are otherwise narrowed to template-referenced
    // names so a genuinely unused binding still reports TS6133.
    let extra_template_referenced_names = codegen_options.preserve_unused_diagnostics.then(|| {
        let mut names = collect_art_template_referenced_names(
            descriptor,
            codegen_options.template_syntax,
            codegen_options.experimental_in_tag_comments,
        );
        names.extend(collect_css_var_referenced_names(descriptor));
        names
    });

    let hoist_shared_preamble = codegen_options.hoist_shared_preamble && !vue2_compat;
    let generation_options = VirtualTsGenerationOptions {
        diagnostic_ownership: codegen_options.diagnostic_ownership,
        generic_parameters: generic_parameters.as_ref(),
        authored_scripts: Some(AuthoredScriptMetadata::from_descriptor(descriptor)),
        check_options: codegen_options.check_options,
        dialect: codegen_options.dialect,
        preserve_unused_diagnostics: codegen_options.preserve_unused_diagnostics,
        extra_template_referenced_names: extra_template_referenced_names.as_ref(),
        options_api: codegen_options.options_api || vue2_compat,
        preserve_authored_component: codegen_options.preserve_authored_component,
        component_name: codegen_options.component_name,
        self_component_name: path.file_stem().and_then(|name| name.to_str()),
        preserve_event_navigation: codegen_options.preserve_event_navigation,
        legacy_vue2: vue2_compat,
        template_syntax_quirks: matches!(
            codegen_options.template_syntax,
            TemplateSyntaxMode::Quirks
        ),
        hoist_shared_preamble,
        external_component_attrs_module: codegen_options.external_component_attrs_module,
        lib_references: None,
        omit_vite_client_reference: codegen_options.omit_vite_client_reference,
        split_script_setup_offsets,
        script_syntax_boundaries: &script_diagnostics.boundaries,
        experimental_strict_slot_children: codegen_options.experimental_strict_slot_children,
    };
    let mut output = if let Some(plan) = generic_parameters
        .as_ref()
        .filter(|plan| !plan.diagnostics.is_empty())
    {
        let Some(setup) = descriptor.script_setup.as_ref().filter(|_| {
            descriptor.script.is_none()
                && !generation_options.legacy_vue2
                && generation_options.dialect == VueVersion::V3
        }) else {
            return Ok(generic_syntax::refuse(path, source, descriptor));
        };
        let Some(attribute) =
            crate::virtual_ts::generic_parameters::authored_generic_range(source, descriptor)
                .filter(|range| source.get(range.clone()) == Some(plan.raw()))
        else {
            return Ok(generic_syntax::refuse(path, source, descriptor));
        };
        let Some(output) = crate::virtual_ts::generic_recovery::generate(
            &croquis,
            script_content.as_deref().unwrap_or_default(),
            (
                script_offset,
                crate::script_parse::script_source_type(setup.lang.as_deref()),
            ),
            attribute,
            plan,
            options,
            generation_options,
        ) else {
            return Ok(generic_syntax::refuse(path, source, descriptor));
        };
        output
    } else {
        profile!(
            "canon.virtual_ts.generate",
            generate_virtual_ts_with_offsets_and_checks(
                &croquis,
                script_content.as_deref(),
                template_ast.as_ref(),
                script_offset,
                template_offset,
                options,
                generation_options,
            )
        )
    };

    let typed_router_import = typed_router.apply(
        &mut output,
        descriptor,
        script_content.as_deref(),
        script_offset,
        split_script_setup_offsets,
        codegen_options.diagnostic_ownership,
    );

    // Surface Vue-specific semantic errors (e.g. DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE)
    // that the SFC compiler catches but TypeScript itself does not. Without this,
    // `vize check` would silently accept SFCs that `vize build` rejects.
    if let Some(diagnostic) = profile!(
        "canon.sfc.compile_validate",
        collect_sfc_compile_diagnostic(path, source, descriptor)
    ) {
        diagnostics.push(diagnostic);
    }

    let mut code = output.code;
    let (mut mappings, mut mapping_meta, semantic_links, prop_default_key_links) =
        output.mapping.into_binding_parts_with_metadata();
    let generic_copies = generic_parameters.as_ref().and_then(|plan| {
        let attribute =
            crate::virtual_ts::generic_parameters::authored_generic_range(source, descriptor)?;
        generic_attribute::GenericCopies::find(&code, &mappings, plan, attribute)
    });
    generic_attribute::map_generic_attribute(
        &code,
        &mut mappings,
        &mut mapping_meta,
        generic_copies.as_ref(),
    );
    style_modules::append_style_scoped_classes(
        &mut code,
        source,
        descriptor,
        codegen_options.check_options,
    );
    style_modules::append_duplicate_style_modules(
        &mut code,
        &mut mappings,
        source,
        descriptor,
        codegen_options.check_options.strict_css_modules,
    );

    Ok(GeneratedVueFile {
        diagnostic_route,
        generic_copies,
        code,
        mappings,
        mapping_meta,
        semantic_links,
        prop_default_key_links,
        diagnostics,
        typed_router_import,
        empty_module_exports: module_exports.as_ref().is_some_and(Vec::is_empty),
        module_exports: module_exports.map(Into::into),
    })
}
