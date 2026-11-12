//! Script binding reads outside the template: `<style>` `v-bind()`, the
//! `<script setup generic="...">` attribute, and Musea `<art>` variants.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::{CompactString, cstr};

use super::art_use::{is_art_block, note_art_uses};
use super::template_use::{TemplateUse, note_expression};

pub(super) fn note_sfc_uses(descriptor: &SfcDescriptor<'_>, uses: &mut TemplateUse) {
    // `color: v-bind(tint)` and `v-bind('theme.color')` read script bindings.
    for expression in &descriptor.css_vars {
        note_expression(expression, &[], uses);
    }
    if let Some(generic) = descriptor
        .script_setup
        .as_ref()
        .and_then(|block| block.attrs.get("generic"))
    {
        note_generic(generic, uses);
    }
    for block in &descriptor.custom_blocks {
        if is_art_block(&block.block_type) {
            note_art_uses(&block.content, uses);
        }
    }
}

/// `generic="Row extends RowBase"` reads the type `RowBase`.
fn note_generic(generic: &str, uses: &mut TemplateUse) {
    let generic = generic.trim();
    if generic.is_empty() {
        return;
    }
    let allocator = Allocator::default();
    let source = cstr!("type __VizeGeneric<{generic}> = 0;");
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    for ids in scoping.root_unresolved_references_ids() {
        for id in ids {
            let reference = scoping.get_reference(id);
            uses.reads
                .insert(CompactString::new(semantic.reference_name(reference)));
        }
    }
}
