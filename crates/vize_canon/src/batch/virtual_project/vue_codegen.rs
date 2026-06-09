//! Generating virtual TypeScript for `.vue` SFCs: parsing the template, running
//! Croquis analysis, augmenting type-based props, and emitting the `.vue.ts`
//! source consumed by Corsa. Parse/compile errors are surfaced as diagnostics
//! and replaced with a typed fallback module.

use std::path::Path;

use vize_carton::{Bump, FxHashSet, String as CompactString, cstr, profile};

use vize_atelier_core::{
    ParserOptions, TemplateSyntaxMode, parser::parse_with_options_and_template_syntax,
};
use vize_atelier_sfc::{
    SfcDescriptor,
    croquis::{
        SfcCroquisOptions, analyze_sfc_descriptor_with_context,
        analyze_sfc_descriptor_with_context_legacy_vue2,
        analyze_sfc_descriptor_with_context_options_api,
    },
    script::ScriptCompileContext,
};

use crate::batch::error::CorsaResult;
use crate::batch::{Diagnostic, SfcBlockType};
use crate::script_parse::collect_script_parse_diagnostics;
use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions, extract_interface_fields,
    generate_virtual_ts_with_offsets_and_checks,
};

use super::diagnostics::{
    collect_sfc_compile_diagnostic, diagnostic_for_offset, invalid_sfc_fallback_virtual_ts,
};

pub(super) struct GeneratedVueFile {
    pub(super) code: CompactString,
    pub(super) mappings: Vec<crate::virtual_ts::VizeMapping>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
pub(super) struct VueCodegenOptions {
    pub(super) check_options: VirtualTsCheckOptions,
    pub(super) options_api: bool,
    pub(super) legacy_vue2: bool,
    pub(super) template_syntax: TemplateSyntaxMode,
}

pub(super) fn generate_vue_virtual_ts(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
    options: &VirtualTsOptions,
    codegen_options: VueCodegenOptions,
) -> CorsaResult<GeneratedVueFile> {
    let allocator = Bump::new();
    let mut diagnostics = Vec::new();

    if let Some(ref script) = descriptor.script {
        let script_diagnostics =
            collect_script_parse_diagnostics(&script.content, script.loc.start as u32);
        if !script_diagnostics.is_empty() {
            diagnostics.extend(script_diagnostics.into_iter().map(|diagnostic| {
                diagnostic_for_offset(
                    path,
                    source,
                    diagnostic.start,
                    cstr!("Script parse error: {}", diagnostic.message),
                    SfcBlockType::Script,
                )
            }));
        }
    }

    if let Some(ref script_setup) = descriptor.script_setup {
        let script_diagnostics =
            collect_script_parse_diagnostics(&script_setup.content, script_setup.loc.start as u32);
        if !script_diagnostics.is_empty() {
            diagnostics.extend(script_diagnostics.into_iter().map(|diagnostic| {
                diagnostic_for_offset(
                    path,
                    source,
                    diagnostic.start,
                    cstr!("Script parse error: {}", diagnostic.message),
                    SfcBlockType::ScriptSetup,
                )
            }));
        }
    }

    let template_offset = descriptor
        .template
        .as_ref()
        .map(|template| template.loc.start as u32)
        .unwrap_or(0);
    let template_ast = descriptor.template.as_ref().and_then(|template| {
        profile!("canon.template.parse", {
            let (root, errors) = parse_with_options_and_template_syntax(
                &allocator,
                &template.content,
                ParserOptions::default(),
                codegen_options.template_syntax,
            );
            if errors.is_empty() {
                Some(root)
            } else {
                diagnostics.extend(errors.into_iter().map(|error| {
                    let start = error
                        .loc
                        .as_ref()
                        .map(|loc| template_offset + loc.start.offset)
                        .unwrap_or(template_offset);
                    diagnostic_for_offset(
                        path,
                        source,
                        start,
                        cstr!("Template parse error: {}", error.message),
                        SfcBlockType::Template,
                    )
                }));
                None
            }
        })
    });

    if !diagnostics.is_empty() {
        return Ok(GeneratedVueFile {
            code: invalid_sfc_fallback_virtual_ts(),
            mappings: Vec::new(),
            diagnostics,
        });
    }

    let croquis_options = SfcCroquisOptions::full();

    let analysis = profile!(
        "canon.croquis.analyze_sfc",
        if codegen_options.legacy_vue2 {
            analyze_sfc_descriptor_with_context_legacy_vue2(
                descriptor,
                template_ast.as_ref(),
                croquis_options,
            )
        } else if codegen_options.options_api {
            analyze_sfc_descriptor_with_context_options_api(
                descriptor,
                template_ast.as_ref(),
                croquis_options,
            )
        } else {
            analyze_sfc_descriptor_with_context(descriptor, template_ast.as_ref(), croquis_options)
        }
    );
    let vize_atelier_sfc::croquis::SfcCroquisAnalysis {
        mut croquis,
        script_content,
        script_offset,
    } = analysis;
    profile!(
        "canon.croquis.augment_type_props",
        augment_type_based_props_from_script_context(&mut croquis, descriptor, path)
    );

    let output = profile!(
        "canon.virtual_ts.generate",
        generate_virtual_ts_with_offsets_and_checks(
            &croquis,
            script_content.as_deref(),
            template_ast.as_ref(),
            script_offset,
            template_offset,
            options,
            VirtualTsGenerationOptions {
                check_options: codegen_options.check_options,
                options_api: codegen_options.options_api,
                legacy_vue2: codegen_options.legacy_vue2,
            },
        )
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

    Ok(GeneratedVueFile {
        code: output.code,
        mappings: output.mappings,
        diagnostics,
    })
}

fn augment_type_based_props_from_script_context(
    croquis: &mut vize_croquis::Croquis,
    descriptor: &SfcDescriptor<'_>,
    path: &Path,
) {
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return;
    };
    if croquis
        .macros
        .define_props()
        .is_none_or(|call| call.type_args.is_none())
    {
        return;
    }

    let mut ctx = ScriptCompileContext::new(&script_setup.content);
    let path_string = path.to_string_lossy();

    if let Some(script) = descriptor.script.as_ref()
        && !script.content.is_empty()
    {
        ctx.collect_types_from(&script.content);
        ctx.collect_imported_types_from_path(&script.content, path_string.as_ref());
    }
    ctx.collect_imported_types_from_path(&script_setup.content, path_string.as_ref());
    ctx.analyze();

    let known_props = known_type_based_prop_names(croquis, &script_setup.content);
    let mut missing_props: Vec<CompactString> = ctx
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            matches!(binding_type, vize_relief::BindingType::Props)
                .then(|| name)
                .filter(|name| !known_props.contains(*name))
                .cloned()
        })
        .collect();
    if missing_props.is_empty() {
        return;
    }
    missing_props.sort();

    for name in missing_props {
        croquis
            .bindings
            .bindings
            .entry(name.clone())
            .or_insert(vize_relief::BindingType::Props);
        croquis
            .macros
            .add_prop(vize_croquis::macros::PropDefinition {
                name,
                prop_type: None,
                required: false,
                default_value: None,
            });
    }
}

fn known_type_based_prop_names(
    croquis: &vize_croquis::Croquis,
    script_setup: &str,
) -> FxHashSet<CompactString> {
    let mut names: FxHashSet<CompactString> = croquis
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.clone())
        .collect();

    let Some(type_args) = croquis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_ref())
    else {
        return names;
    };

    let type_name = strip_outer_angle_brackets(type_args.trim());
    for prop in croquis.types.extract_properties(type_name) {
        names.insert(prop.name);
    }
    for field in extract_interface_fields(script_setup, type_name) {
        names.insert(CompactString::new(field));
    }

    names
}

fn strip_outer_angle_brackets(value: &str) -> &str {
    value
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(value)
}
