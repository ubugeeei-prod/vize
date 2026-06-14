//! Contract tests for section-first inline template extraction.
//!
//! The SFC compiler has several template lanes, and each lane exposes a
//! slightly different usable boundary:
//!
//! - DOM records fine render-body sections for `<script setup>` inlining.
//! - SSR records coarse module sections because inline mode keeps the whole
//!   `ssrRender` function.
//! - Vapor records the same coarse module sections after its adapter normalizes
//!   runtime imports and render signatures.
//!
//! These tests keep those lane contracts explicit so the legacy line scanner can
//! remain a compatibility fallback instead of a hidden hot-path dependency.

use std::borrow::Cow;

use vize_atelier_core::{TemplateSyntaxMode, source_atlas::SourceAtlasFallback};
use vize_carton::String;

use super::{
    TemplateBlockCompileContext, TemplateBlockCompileResult, compile_template_block,
    compile_template_block_vapor,
};
use crate::{
    compile::output_module::AtelierOutputMaps,
    types::{BindingMetadata, BlockLocation, SfcTemplateBlock, TemplateCompileOptions},
};

fn template(content: &'static str) -> SfcTemplateBlock<'static> {
    SfcTemplateBlock {
        content: Cow::Borrowed(content),
        loc: BlockLocation {
            start: 0,
            end: 0,
            tag_start: 0,
            tag_end: 0,
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        },
        lang: None,
        src: None,
        attrs: Default::default(),
    }
}

#[test]
fn dom_compile_records_fine_sections_for_inline_body_extraction() {
    let template = template("<MyWidget v-focus>{{ count + 1 }}</MyWidget>");

    let result = compile_template_block(
        &template,
        &TemplateCompileOptions::default(),
        TemplateBlockCompileContext {
            scope_id: "abc123",
            apply_scope_id: false,
            has_scoped: false,
            is_ts: false,
            inline: true,
            component_name: Some("DomComp"),
            bindings: None,
            croquis: None,
        },
        TemplateSyntaxMode::Standard,
    )
    .expect("DOM template should compile");

    assert!(
        result.sections.is_some(),
        "DOM output must expose fine sections for render-body inlining"
    );
    let parts = result.body_parts_for_inline();
    assert_eq!(parts.fallback, None);
    assert!(parts.imports.contains("from 'vue'") || parts.imports.contains("from \"vue\""));
    assert!(
        parts.preamble.contains("_resolveComponent")
            || parts.preamble.contains("_resolveDirective"),
        "component/directive resolution should stay inside the setup preamble"
    );
    assert!(
        parts.render_body.contains("_createVNode") || parts.render_body.contains("_withDirectives")
    );
    assert_eq!(parts.render_fn_name, "render");
}

#[test]
fn vapor_compile_records_module_sections_for_inline_full_function_extraction() {
    let template = template("<button>{{ label }}</button>");
    let bindings = BindingMetadata::default();

    let result = compile_template_block_vapor(
        &template,
        "abc123",
        true,
        Some(&bindings),
        false,
        TemplateSyntaxMode::Standard,
    )
    .expect("Vapor template should compile");

    assert!(
        result.module_sections.is_some(),
        "Vapor output must expose module sections for full-function inlining"
    );
    let parts = result.full_parts_for_inline("render");
    assert_eq!(parts.fallback, None);
    assert!(parts.imports.contains("from 'vue'") || parts.imports.contains("from \"vue\""));
    assert!(parts.hoisted.contains("_template("));
    assert!(parts.render_fn.contains("function render("));
    assert_eq!(parts.render_fn_name, "render");
}

#[test]
fn ssr_compile_records_module_sections_for_inline_full_function_extraction() {
    let template = template("<MyWidget :count=\"count\">{{ label }}</MyWidget>");
    let options = TemplateCompileOptions {
        ssr: true,
        ..Default::default()
    };

    let result = compile_template_block(
        &template,
        &options,
        TemplateBlockCompileContext {
            scope_id: "abc123",
            apply_scope_id: false,
            has_scoped: false,
            is_ts: false,
            inline: true,
            component_name: Some("SsrComp"),
            bindings: None,
            croquis: None,
        },
        TemplateSyntaxMode::Standard,
    )
    .expect("SSR template should compile");

    assert!(
        result.module_sections.is_some(),
        "SSR output must expose module sections for full-function inlining"
    );
    let parts = result.full_parts_for_inline("ssrRender");
    assert_eq!(parts.fallback, None);
    assert!(parts.imports.contains("vue/server-renderer"));
    assert!(
        parts.render_fn.contains("function ssrRender(")
            || parts.render_fn.contains("export function ssrRender(")
    );
    assert_eq!(parts.render_fn_name, "ssrRender");
}

#[test]
fn inline_extraction_reports_legacy_scanner_only_without_sections() {
    let output = TemplateBlockCompileResult {
        code: String::from(
            r#"import { createVNode as _createVNode } from 'vue'

const _hoisted_1 = { class: "test" }

export function render(_ctx, _cache) {
  return _createVNode("div", _hoisted_1, "Hello")
}"#,
        ),
        warnings: std::vec::Vec::new(),
        sections: None,
        module_sections: None,
        maps: AtelierOutputMaps::default(),
    };

    let body_parts = output.body_parts_for_inline();
    assert_eq!(
        body_parts.fallback,
        Some(SourceAtlasFallback::LegacyLineScanner)
    );
    assert_eq!(body_parts.render_fn_name, "render");

    let full_parts = output.full_parts_for_inline("render");
    assert_eq!(
        full_parts.fallback,
        Some(SourceAtlasFallback::LegacyLineScanner)
    );
    assert_eq!(full_parts.render_fn_name, "render");
}
