//! Template compilation for Vue SFCs.
//!
//! This module handles compilation of `<template>` blocks,
//! supporting both DOM mode and Vapor mode.

use vize_carton::{String, ToCompactString, profile};
mod extraction;
mod string_tracking;
mod vapor;

#[cfg(test)]
mod map_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use extraction::extract_template_parts;
pub(crate) use vapor::compile_template_block_vapor;

use vize_atelier_core::{
    TemplateSyntaxMode,
    rendu::RenduRange,
    source_atlas::SourceAtlasFallback,
    source_map::{SourceMapRegistration, SourceMapRegistrationState},
};
use vize_carton::Bump;

use crate::compile::output_module::{
    AtelierModuleSections, AtelierOutputMaps, AtelierOutputSections, OutputModule,
};
use crate::types::{BindingMetadata, SfcError, SfcTemplateBlock, TemplateCompileOptions};

/// Structured template output returned from one Atelier lane.
///
/// `code` is still the flattened JavaScript module because the public SFC
/// compiler surfaces byte-equivalent JS today. The important canary contract is
/// that the flattened string is no longer the only source of truth: DOM, SSR,
/// and Vapor producers attach section marks that inline SFC assembly can slice
/// directly. If a future lane cannot provide those marks, the caller must treat
/// string recovery as `SourceAtlasFallback::LegacyLineScanner`.
pub(crate) struct TemplateBlockCompileResult {
    pub(crate) code: String,
    pub(crate) warnings: std::vec::Vec<SfcError>,
    /// Section boundaries of `code`, recorded while the render module was
    /// emitted.
    ///
    /// These are fine-grained DOM render-body sections: helper imports,
    /// hoisted declarations, asset-resolution statements, and the returned
    /// expression. Script-setup inline mode uses them to recover the render body
    /// without scanning generated JavaScript line by line.
    ///
    /// `None` is expected for SSR and Vapor, which use coarse module sections
    /// instead, and for error paths where no trustworthy output plate exists.
    pub(crate) sections: Option<AtelierOutputSections>,
    /// Coarse module chunk boundaries in `code`, recorded by SFC output
    /// assembly.
    ///
    /// These ranges describe imports, hoists, render functions, and exports in
    /// the final module string. SSR and Vapor inline modes use them to slice the
    /// complete render function from `AtelierOutput`, preserving the exact
    /// output bytes while dropping the legacy line scanner from the normal path.
    pub(crate) module_sections: Option<AtelierModuleSections>,
    /// Template source-map fragments carried by the Atelier output boundary.
    ///
    /// SFC compilation does not expose these as the final public source map yet:
    /// script/template/style assembly still needs a composed map. Keeping the
    /// fragment here prevents the DOM Atelier map from being discarded before
    /// that composition stage exists.
    pub(crate) maps: AtelierOutputMaps,
}

/// Full render-function pieces used by SSR and Vapor inline SFC assembly.
///
/// This shape mirrors the old scanner tuple, but carries the extra fallback
/// reason. The tuple compatibility keeps existing assembly code small while the
/// source atlas learns whether a lane was served by registered output sections
/// or by the legacy recovery scanner.
pub(crate) struct TemplateFullParts {
    /// Module imports that must remain at the top of the final SFC module.
    pub(crate) imports: String,
    /// Module-level helper/template declarations emitted before the render
    /// function.
    pub(crate) hoisted: String,
    /// The complete render function body, including its signature and closing
    /// brace.
    pub(crate) render_fn: String,
    /// Name exported by the target Atelier, normally `render` or `ssrRender`.
    pub(crate) render_fn_name: &'static str,
    /// Legacy recovery reason when the output did not carry module sections.
    pub(crate) fallback: Option<SourceAtlasFallback>,
}

/// Inline client-render pieces used by `<script setup>` assembly.
///
/// DOM inline mode inserts template imports, hoists, asset preamble statements,
/// and the returned render expression into the generated setup function. These
/// fields are sliced from `AtelierOutputSections` whenever possible so the
/// compiler does not rediscover known structure from a flattened JS string.
pub(crate) struct TemplateBodyParts {
    /// Module imports required by the template render body.
    pub(crate) imports: String,
    /// Static vnode declarations and other hoisted render artifacts.
    pub(crate) hoisted: String,
    /// Component/directive resolution statements that must run inside setup.
    pub(crate) preamble: String,
    /// The expression returned by the render function.
    pub(crate) render_body: String,
    /// Name emitted by the target Atelier. Client inline mode expects `render`.
    pub(crate) render_fn_name: &'static str,
    /// Legacy recovery reason when the output did not carry fine sections.
    pub(crate) fallback: Option<SourceAtlasFallback>,
}

pub(crate) struct TemplateBlockCompileContext<'a> {
    pub(crate) scope_id: &'a str,
    pub(crate) apply_scope_id: bool,
    /// Whether the component has any `<style scoped>` block. When true, hoisted
    /// module-level static vnodes must carry the `data-v-*` attribute so scoped
    /// CSS selectors continue to match them in client builds.
    pub(crate) has_scoped: bool,
    pub(crate) is_ts: bool,
    pub(crate) inline: bool,
    pub(crate) component_name: Option<&'a str>,
    pub(crate) bindings: Option<&'a BindingMetadata>,
    pub(crate) croquis: Option<vize_croquis::analysis::Croquis>,
}

impl TemplateBlockCompileResult {
    /// Return full render-function parts for lanes that cannot inline only the
    /// returned expression.
    ///
    /// SSR and Vapor need the whole render function in script-setup mode. The
    /// preferred path slices `module_sections` recorded by `OutputModule` or the
    /// Vapor adapter. Only hand-built or legacy outputs without those ranges
    /// fall back to the scanner, and that fallback is surfaced to the caller so
    /// the Source Atlas profile can record it.
    pub(crate) fn full_parts_for_inline(&self, render_fn_name: &'static str) -> TemplateFullParts {
        let template_code = &self.code;
        let (imports, hoisted, render_fn, render_fn_name, fallback) = match &self.module_sections {
            Some(sections) => {
                let (imports, hoisted, render_fn, render_fn_name) =
                    extraction::slice_template_parts_full(template_code, sections, render_fn_name);
                (imports, hoisted, render_fn, render_fn_name, None)
            }
            None => {
                let (imports, hoisted, render_fn, render_fn_name) =
                    extraction::extract_template_parts_full(template_code);
                (
                    imports,
                    hoisted,
                    render_fn,
                    render_fn_name,
                    Some(SourceAtlasFallback::LegacyLineScanner),
                )
            }
        };

        TemplateFullParts {
            imports,
            hoisted,
            render_fn,
            render_fn_name,
            fallback,
        }
    }

    /// Return render-body parts for client `<script setup>` inline assembly.
    ///
    /// This is the section-first replacement for the old direct
    /// `extract_template_parts` call. DOM codegen records precise byte ranges
    /// while emitting, so the normal path is a set of string slices plus tiny
    /// trimming of asset statements. The old scanner remains only as a
    /// compatibility fallback for outputs that do not yet carry sections.
    pub(crate) fn body_parts_for_inline(&self) -> TemplateBodyParts {
        let template_code = &self.code;
        let (imports, hoisted, preamble, render_body, render_fn_name, fallback) =
            match &self.sections {
                Some(sections) => {
                    let (imports, hoisted, preamble, render_body, render_fn_name) =
                        extraction::slice_template_parts(template_code, sections);
                    (
                        imports,
                        hoisted,
                        preamble,
                        render_body,
                        render_fn_name,
                        None,
                    )
                }
                None => {
                    let (imports, hoisted, preamble, render_body, render_fn_name) =
                        extraction::extract_template_parts(template_code);
                    (
                        imports,
                        hoisted,
                        preamble,
                        render_body,
                        render_fn_name,
                        Some(SourceAtlasFallback::LegacyLineScanner),
                    )
                }
            };

        TemplateBodyParts {
            imports,
            hoisted,
            preamble,
            render_body,
            render_fn_name,
            fallback,
        }
    }

    pub(crate) fn source_map_fragment(&self) -> Option<&str> {
        self.maps.source_map()
    }

    pub(crate) fn source_map_registration(
        &self,
        state: SourceMapRegistrationState,
    ) -> Option<SourceMapRegistration<'_>> {
        let fragment = self.source_map_fragment()?;
        Some(
            SourceMapRegistration::for_template_fragment(
                self.source_map_generated_range(),
                fragment,
                state,
            )
            .with_source_name("template.vue"),
        )
    }

    pub(crate) fn source_map_json(&self) -> Option<serde_json::Value> {
        self.source_map_fragment()
            .and_then(|fragment| serde_json::from_str(fragment).ok())
    }

    fn source_map_generated_range(&self) -> RenduRange {
        self.module_sections
            .map(|sections| sections.functions)
            .unwrap_or_else(|| RenduRange::new(0, self.code.len()))
    }
}

/// Compile template block
pub(crate) fn compile_template_block(
    template: &SfcTemplateBlock,
    options: &TemplateCompileOptions,
    ctx: TemplateBlockCompileContext<'_>,
    template_syntax: TemplateSyntaxMode,
) -> Result<TemplateBlockCompileResult, SfcError> {
    let TemplateBlockCompileContext {
        scope_id,
        apply_scope_id,
        has_scoped,
        is_ts,
        inline,
        component_name,
        bindings,
        croquis,
    } = ctx;
    let allocator = Bump::new();
    let scope_attr = if apply_scope_id {
        let mut attr = String::with_capacity(scope_id.len() + 7);
        attr.push_str("data-v-");
        attr.push_str(scope_id);
        Some(attr)
    } else {
        None
    };

    if options.ssr {
        let ssr_opts = vize_atelier_ssr::SsrCompilerOptions {
            scope_id: scope_attr,
            component_name: component_name.map(|name| name.to_compact_string()),
            comments: options
                .compiler_options
                .as_ref()
                .is_some_and(|opts| opts.comments),
            inline: false,
            is_ts,
            custom_renderer: options.custom_renderer,
            ssr_css_vars: options.ssr_css_vars.clone(),
            dialect: options.dialect,
            binding_metadata: bindings.cloned(),
            croquis: croquis.map(Box::new),
        };

        let (_, errors, result) = profile!(
            "atelier.sfc.template.ssr",
            vize_atelier_ssr::compile_ssr_with_template_syntax(
                &allocator,
                &template.content,
                ssr_opts,
                template_syntax,
            )
        );

        // Recoverable parser diagnostics (e.g. duplicate attribute) must
        // not gate SFC compilation, or a single `<div id=a id=b>` produces
        // a 0-byte module marked as success. (#958)
        let fatal: Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
        if !fatal.is_empty() {
            let mut message = String::from("Template compilation errors: ");
            use std::fmt::Write as _;
            let _ = write!(&mut message, "{:?}", fatal);
            return Err(SfcError {
                message,
                code: Some("TEMPLATE_ERROR".to_compact_string()),
                loc: Some(template.loc.clone()),
            });
        }

        let output_module = OutputModule::from_ssr_codegen(result);
        let module_sections = output_module.module_sections();
        let output = output_module.into_code();
        return Ok(TemplateBlockCompileResult {
            code: output,
            warnings: recoverable_template_warnings(&errors),
            sections: None,
            module_sections: Some(module_sections),
            maps: AtelierOutputMaps::default(),
        });
    }

    // Build DOM compiler options
    let mut dom_opts = options.compiler_options.clone().unwrap_or_default();
    dom_opts.mode = vize_atelier_core::options::CodegenMode::Module;
    dom_opts.prefix_identifiers = true;
    // Vue applies SFC scope IDs at runtime. Only module-level hoisted VNodes
    // need an explicit scope attr baked into their props.
    dom_opts.scope_id = None;
    // Hoisted module-level static vnodes are created at import time, when the
    // runtime's `currentScopeId` is null, so the runtime cannot stamp the
    // scoped-CSS attribute on them. Bake `data-v-*` directly into their props
    // here whenever the component owns a scoped style block.
    let hoisted_scope_attr = if has_scoped {
        let mut attr = String::with_capacity(scope_id.len() + 7);
        attr.push_str("data-v-");
        attr.push_str(scope_id);
        Some(attr)
    } else {
        None
    };
    dom_opts.ssr = options.ssr;
    dom_opts.is_ts = is_ts;
    dom_opts.custom_renderer = options.custom_renderer;
    dom_opts.dialect = options.dialect;
    dom_opts.component_name = component_name.map(|name| name.to_compact_string());

    // For script setup, use inline mode to match Vue's actual compiler behavior
    // Inline mode generates direct closure references (e.g., msg instead of $setup.msg)
    // which are captured in the setup() function scope
    if inline && bindings.is_some() {
        dom_opts.inline = true;
        dom_opts.hoist_static = true;
        dom_opts.cache_handlers = true;
    }

    // Pass binding metadata directly (no string conversion needed)
    dom_opts.binding_metadata = bindings.cloned();

    // Pass Croquis to DOM compiler for enhanced transforms
    if let Some(c) = croquis {
        dom_opts.croquis = Some(Box::new(c));
    }

    // Compile template
    let (_, errors, result) = profile!(
        "atelier.sfc.template.dom",
        vize_atelier_dom::compile_template_with_template_syntax_and_hoisted_scope_id_with_sections(
            &allocator,
            &template.content,
            dom_opts,
            template_syntax,
            hoisted_scope_attr,
        )
    );

    // See above — drop recoverable parser diagnostics from the gating
    // check so duplicate-attribute SFCs still produce valid render code. (#958)
    let fatal: Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
    if !fatal.is_empty() {
        let mut message = String::from("Template compilation errors: ");
        use std::fmt::Write as _;
        let _ = write!(&mut message, "{:?}", fatal);
        return Err(SfcError {
            message,
            code: Some("TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    }

    let output_module = OutputModule::from_dom_codegen(result);
    let sections = output_module.sections;
    let module_sections = output_module.module_sections();
    let (output, maps) = output_module.into_code_and_maps();

    Ok(TemplateBlockCompileResult {
        code: output,
        warnings: recoverable_template_warnings(&errors),
        sections,
        module_sections: Some(module_sections),
        maps,
    })
}

pub(crate) fn recoverable_template_warnings(
    errors: &[vize_atelier_core::CompilerError],
) -> std::vec::Vec<SfcError> {
    errors
        .iter()
        .filter(|error| error.is_recoverable())
        .cloned()
        .map(Into::into)
        .collect()
}
