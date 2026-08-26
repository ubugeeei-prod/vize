//! Virtual TypeScript generation for standalone `.art.vue` files.
//!
//! Every authored `<variant>` gets its own typed document over the same
//! authored script context. One document per file would type-check whichever
//! variant happened to be default and leave every other variant's expressions
//! outside the checker entirely (#4015).

use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Drawer, DrawerOptions};
use vize_s0::cstr;

use super::super::{DiagnosticService, VirtualTsResult};
use super::virtual_ts::rewrite_vue_imports;
use super::virtual_ts_art_bindings::add_art_target_component_bindings;
use super::virtual_ts_art_imports::collect_art_vue_dependency_paths;

/// One art variant projected into its own typed virtual TypeScript document.
pub(in crate::ide::diagnostics) struct ArtVariantVirtualTs {
    /// Position of the authored `<variant>` inside the `<art>` block.
    pub(in crate::ide::diagnostics) variant_index: usize,
    pub(in crate::ide::diagnostics) virtual_result: VirtualTsResult,
}

pub(in crate::ide::diagnostics) struct ArtVirtualTsResult {
    pub(in crate::ide::diagnostics) variants: Vec<ArtVariantVirtualTs>,
    pub(in crate::ide::diagnostics) vue_dependencies: Vec<PathBuf>,
}

/// Virtual document identity for one art variant.
///
/// The first variant keeps the plain `<file>.ts` identity the single-document
/// generation used, so an already-open session is updated rather than joined by
/// a second copy of the same file. Every identity stays in the authored
/// directory, so relative specifiers resolve exactly as before.
pub(in crate::ide::diagnostics) fn art_variant_virtual_name(
    uri: &Url,
    variant_index: usize,
) -> String {
    if variant_index == 0 {
        cstr!("{}.ts", uri.path()).to_string()
    } else {
        cstr!("{}.art_variant_{variant_index}.ts", uri.path()).to_string()
    }
}

/// The authored script context shared by every variant of one art file.
struct ArtScriptContext {
    script: String,
    script_offset: u32,
    sfc_script_start_line: u32,
}

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn generate_virtual_ts_for_art_with_dependencies(
        uri: &Url,
        content: &str,
        base_options: &VirtualTsOptions,
    ) -> Option<ArtVirtualTsResult> {
        let art_allocator = vize_s0::Allocator::new();
        let art_desc = vize_musea::parse_art(
            &art_allocator,
            content,
            vize_musea::ArtParseOptions::default(),
        )
        .ok()?;

        let descriptor = vize_atelier_sfc::parse_sfc(
            content,
            vize_atelier_sfc::SfcParseOptions {
                filename: uri.path().to_string().into(),
                ..Default::default()
            },
        )
        .ok()?;

        let target_component = descriptor
            .script_setup
            .as_ref()
            .and_then(|script_setup| {
                crate::virtual_code::find_define_art_target_component(script_setup.content.as_ref())
            })
            .or_else(|| {
                art_desc
                    .metadata
                    .component
                    .and_then(crate::virtual_code::art_target_component_from_source)
            });
        let script = art_script_context(&descriptor);

        let mut variants = Vec::new();
        let mut vue_dependencies: Vec<PathBuf> = Vec::new();
        for (variant_index, variant) in art_desc.variants.iter().enumerate() {
            let Some(template_offset) = borrowed_offset(content, variant.template) else {
                continue;
            };
            if variant.template.trim().is_empty() {
                continue;
            }

            let generated = Self::generate_art_variant_virtual_ts(
                uri,
                &script,
                variant.template,
                template_offset,
                target_component.as_ref(),
                base_options,
            );
            for dependency in generated.vue_dependencies {
                if !vue_dependencies.contains(&dependency) {
                    vue_dependencies.push(dependency);
                }
            }
            variants.push(ArtVariantVirtualTs {
                variant_index,
                virtual_result: generated.virtual_result,
            });
        }

        if variants.is_empty() {
            return None;
        }

        Some(ArtVirtualTsResult {
            variants,
            vue_dependencies,
        })
    }

    fn generate_art_variant_virtual_ts(
        uri: &Url,
        script: &ArtScriptContext,
        template_content: &str,
        template_offset: u32,
        target_component: Option<&crate::virtual_code::ArtTargetComponent>,
        base_options: &VirtualTsOptions,
    ) -> ArtVariantGeneration {
        let script_content = script.script.as_str();
        let template_allocator = vize_s0::Allocator::new();
        let (template_ast, _) = vize_armature::parse(&template_allocator, template_content);

        let mut analyzer = Drawer::with_options(DrawerOptions::full());
        analyzer.analyze_script(script_content);
        analyzer.analyze_template(&template_ast);

        let summary = analyzer.finish();
        let mut virtual_ts_options = base_options.clone();
        if let Some(target) = target_component {
            add_art_target_component_bindings(&mut virtual_ts_options, &summary, target);
        }

        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script_content),
            Some(&template_ast),
            script.script_offset,
            template_offset,
            &virtual_ts_options,
        );
        let code = output.code;
        let semantic_links = output.semantic_links;
        let line_mappings = Self::parse_vize_map_comments(&code);
        let vue_dependencies = collect_art_vue_dependency_paths(uri, &code);
        let (rewritten_code, import_source_map) = rewrite_vue_imports(&code);

        ArtVariantGeneration {
            vue_dependencies,
            virtual_result: VirtualTsResult {
                code: rewritten_code,
                source_mappings: output.mappings,
                semantic_links: super::semantic_links_after_import_rewrite(
                    semantic_links,
                    &import_source_map,
                ),
                import_source_map,
                user_code_start_line: code
                    .lines()
                    .enumerate()
                    .find(|(_, line)| line.contains("// User setup code"))
                    .map(|(i, _)| i as u32 + 1)
                    .unwrap_or(0),
                sfc_script_start_line: script.sfc_script_start_line,
                template_scope_start_line: code
                    .lines()
                    .enumerate()
                    .find(|(_, line)| line.contains("Template Scope"))
                    .map(|(i, _)| i as u32)
                    .unwrap_or(u32::MAX),
                line_mappings,
                skipped_import_lines: Self::count_import_lines(script_content),
            },
        }
    }
}

struct ArtVariantGeneration {
    virtual_result: VirtualTsResult,
    vue_dependencies: Vec<PathBuf>,
}

/// Project the authored `<script setup>` (or classic `<script>`) into the
/// module-level context every variant document is generated against.
fn art_script_context(descriptor: &vize_atelier_sfc::SfcDescriptor<'_>) -> ArtScriptContext {
    let (script, script_offset, sfc_script_start_line) =
        if let Some(script_setup) = descriptor.script_setup.as_ref() {
            let isolate = !script_setup
                .attrs
                .get("isolate")
                .is_some_and(|value| value.as_ref().eq_ignore_ascii_case("false"));
            let parts = crate::virtual_code::analyze_art_script_setup(
                script_setup.content.as_ref(),
                script_setup.loc.start,
                isolate,
            );
            let kept = parts
                .shared_imports
                .iter()
                .chain(parts.isolated_body.iter())
                .map(|chunk| {
                    (
                        chunk.source_start - script_setup.loc.start,
                        chunk.source_end - script_setup.loc.start,
                    )
                })
                .collect::<Vec<_>>();

            (
                blank_dropped_statements(script_setup.content.as_ref(), &kept),
                script_setup.loc.start as u32,
                script_setup.loc.start_line as u32,
            )
        } else if let Some(classic) = descriptor.script.as_ref() {
            (
                classic.content.as_ref().to_owned(),
                classic.loc.start as u32,
                classic.loc.start_line as u32,
            )
        } else {
            (String::new(), 0, 1)
        };

    ArtScriptContext {
        script,
        script_offset,
        sfc_script_start_line,
    }
}

/// Replace every statement the art decomposition drops (`defineArt(...)`, and
/// the target component's own import under `isolate`) with blanks of the same
/// byte length, keeping line breaks.
///
/// Concatenating only the kept chunks would be shorter and reordered, so the
/// generator's `script_offset + local` arithmetic would place every later
/// script diagnostic and hover on the wrong authored line (#4015).
fn blank_dropped_statements(script: &str, kept: &[(usize, usize)]) -> String {
    let mut projected = String::with_capacity(script.len());
    for (offset, character) in script.char_indices() {
        if kept
            .iter()
            .any(|(start, end)| offset >= *start && offset < *end)
            || character == '\n'
            || character == '\r'
        {
            projected.push(character);
        } else {
            for _ in 0..character.len_utf8() {
                projected.push(' ');
            }
        }
    }
    projected
}

/// Byte offset of a slice borrowed out of `source`, or `None` when the parser
/// handed back a copy instead of a borrow. Pointer arithmetic across unrelated
/// allocations would otherwise fabricate a mapping into the authored file.
fn borrowed_offset(source: &str, slice: &str) -> Option<u32> {
    let base = source.as_ptr() as usize;
    let start = slice.as_ptr() as usize;
    if start < base || start.checked_add(slice.len())? > base + source.len() {
        return None;
    }
    u32::try_from(start - base).ok()
}
