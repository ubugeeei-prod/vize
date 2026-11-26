//! What every target of one SFC compile reads alike, and the choices one
//! target makes.
//!
//! A target is one `ssr` / `vapor` / script-output combination of the same
//! call. Everything here except [`choose_target`] depends on the descriptor and
//! the call's options only, so a multi-target compile computes it once.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_carton::{String, profile};
use vize_l1_to_l2::lower::pug::PugSourceMap;

use crate::types::{
    SfcCompileOptions, SfcDescriptor, SfcError, SfcMacroArtifact, TemplateCompileOptions,
};

use super::SfcScriptOutputMode;
use super::styles::{self, CompiledStyles, compile_styles};
use super::template_context::SfcTemplateContext;

/// The name diagnostics and generated code use for the component, and the
/// path imported types resolve from.
pub(super) fn filenames<'o>(
    parse_filename: &'o str,
    script_id: Option<&'o str>,
) -> (&'o str, &'o str) {
    let filename = if parse_filename.is_empty() {
        script_id.unwrap_or("anonymous.vue")
    } else {
        parse_filename
    };
    (filename, script_id.unwrap_or(filename))
}

/// The scope id and the compiled `<style>` blocks.
pub(super) struct Styles {
    pub(super) has_scoped: bool,
    pub(super) slotted: bool,
    pub(super) scope_id: String,
    pub(super) compiled: CompiledStyles,
}

impl Styles {
    /// Style warnings go to `warnings`; they come first in every target's.
    pub(super) fn compile(
        descriptor: &SfcDescriptor,
        options: &SfcCompileOptions,
        filename: &str,
        warnings: &mut Vec<SfcError>,
    ) -> Self {
        let (has_styles, has_scoped, slotted) = styles::scope_flags(&descriptor.styles);
        let scope_id = styles::scope_id_for_descriptor(descriptor, options, has_styles, filename);
        let compiled = profile!(
            "atelier.sfc.styles",
            compile_styles(&descriptor.styles, &scope_id, &options.style, warnings)
                .with_css_var_names(&descriptor.css_vars, &scope_id, options.template.is_prod)
        );
        Self {
            has_scoped,
            slotted,
            scope_id,
            compiled,
        }
    }

    pub(super) fn css(&self) -> Option<String> {
        (!self.compiled.css.is_empty()).then(|| self.compiled.css.clone())
    }
}

/// Set one target's template options and return whether it renders through
/// Vapor and how it assembles script and template.
pub(super) fn choose_target(
    descriptor: &SfcDescriptor,
    template: &mut TemplateCompileOptions,
    vapor_requested: bool,
    script_output: SfcScriptOutputMode,
    auto_ssr_css_vars: bool,
    scope_id: &str,
) -> (bool, SfcScriptOutputMode) {
    if auto_ssr_css_vars {
        crate::css::transform::vars::inject_ssr_values(
            template,
            &descriptor.css_vars,
            scope_id,
            None,
        );
    }
    // A Vapor component renders on the server through the SSR compiler and is
    // marked `__vapor` (keeping Vapor's hydration anchors), as Vue's does.
    template.vapor = template.ssr && vapor_requested;
    let is_vapor = !template.ssr && vapor_requested;
    // The production Vapor runtime does not expose script-setup bindings
    // through the component proxy. Every adapter must inline that template,
    // including native API callers that retain the development default.
    let script_output = if is_vapor && template.is_prod && descriptor.script_setup.is_some() {
        SfcScriptOutputMode::InlineTemplate
    } else {
        script_output
    };
    (is_vapor, script_output)
}

/// The call's inputs and the target-neutral results every target reads.
pub(super) struct Prelude<'a> {
    pub(super) descriptor: &'a SfcDescriptor<'a>,
    pub(super) filename: &'a str,
    pub(super) source_filename: &'a str,
    pub(super) styles: &'a Styles,
    pub(super) component_name: &'a str,
    pub(super) is_ts: bool,
    pub(super) template_is_ts: bool,
    pub(super) auto_ssr_css_vars: bool,
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) custom_elements: &'a CustomElementMatcher,
    pub(super) codegen_options: &'a CodegenOptions,
    pub(super) experimental_self_component: bool,
    pub(super) nuxt_page_meta: bool,
    pub(super) type_project: Option<&'a crate::script::SfcTypeProject>,
    pub(super) pug_source_map: Option<&'a PugSourceMap>,
    pub(super) template_context: SfcTemplateContext<'a>,
}

impl<'a> Prelude<'a> {
    #[expect(clippy::too_many_arguments, reason = "the call's compile inputs")]
    pub(super) fn new(
        descriptor: &'a SfcDescriptor<'a>,
        (filename, source_filename): (&'a str, &'a str),
        options: &SfcCompileOptions,
        auto_ssr_css_vars: bool,
        styles: &'a Styles,
        component_name: &'a str,
        template_syntax: TemplateSyntaxMode,
        custom_elements: &'a CustomElementMatcher,
        codegen_options: &'a CodegenOptions,
        experimental_self_component: bool,
        nuxt_page_meta: bool,
        type_project: Option<&'a crate::script::SfcTypeProject>,
        pug_source_map: Option<&'a PugSourceMap>,
    ) -> Self {
        let template_is_ts = super::template_is_ts(descriptor, options);
        Self {
            descriptor,
            filename,
            source_filename,
            styles,
            component_name,
            is_ts: super::output_is_ts(options),
            template_is_ts,
            auto_ssr_css_vars,
            template_syntax,
            custom_elements,
            codegen_options,
            experimental_self_component,
            nuxt_page_meta,
            type_project,
            pug_source_map,
            template_context: SfcTemplateContext {
                scope_id: &styles.scope_id,
                has_scoped: styles.has_scoped,
                slotted: styles.slotted,
                is_ts: template_is_ts,
                component_name,
                experimental_self_component,
                authored_source: descriptor.source.as_ref(),
                pug_source_map,
            },
        }
    }
}

/// One target of the call: its options and the choices [`choose_target`] made.
pub(super) struct Target<'o> {
    pub(super) options: &'o SfcCompileOptions,
    pub(super) is_vapor: bool,
    pub(super) script_output: SfcScriptOutputMode,
}

/// What one target's result takes ownership of.
pub(super) struct Outputs {
    pub(super) css: Option<String>,
    pub(super) warnings: Vec<SfcError>,
    pub(super) macro_artifacts: Vec<SfcMacroArtifact>,
}

impl Outputs {
    /// This target's copy: a clone, or the original for the last target.
    pub(super) fn for_target(&mut self, last: bool) -> Self {
        if last {
            Self {
                css: self.css.take(),
                warnings: std::mem::take(&mut self.warnings),
                macro_artifacts: std::mem::take(&mut self.macro_artifacts),
            }
        } else {
            Self {
                css: self.css.clone(),
                warnings: self.warnings.clone(),
                macro_artifacts: self.macro_artifacts.clone(),
            }
        }
    }
}
