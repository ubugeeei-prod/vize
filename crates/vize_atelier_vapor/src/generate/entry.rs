//! Public Vapor codegen entrypoints.

use std::fmt::Write;

use crate::ir::{OperationNode, RootIRNode};
use vize_atelier_core::options::BindingMetadata;
use vize_carton::{FxHashSet, String};

use super::context::GenerateContext;
use super::setup::{collect_delegate_events, escape_template, generate_imports};
use super::{
    block_has_template_refs, collect_custom_directives, collect_root_if_templates,
    directive_resolution_ident, generate_block,
};

/// Vapor code generation result
pub struct VaporGenerateResult {
    /// Generated code
    pub code: String,
    /// Static templates
    pub templates: Vec<String>,
}

/// Options for Vapor code generation.
#[derive(Debug, Clone, Copy, Default)]
pub struct VaporGenerateOptions {
    /// JSX closure mode: the render code runs inside the authoring component
    /// function, so free identifiers stay bare instead of `_ctx.`-prefixed.
    pub jsx_closure: bool,
}

/// Experimental Vapor codegen options kept separate from
/// [`VaporGenerateOptions`] so existing Rust option literals remain
/// source-compatible.
#[derive(Debug, Clone, Copy, Default)]
pub struct VaporGenerateExperimentalOptions<'a> {
    /// Current SFC component name for experimental `<Self>` resolution.
    pub component_name: Option<&'a str>,
    /// Treat the reserved `<Self>` tag as a reference to the current SFC.
    pub self_component: bool,
}

/// Generate Vapor code from IR
pub fn generate_vapor(
    ir: &RootIRNode<'_>,
    binding_metadata: Option<&BindingMetadata>,
) -> VaporGenerateResult {
    generate_vapor_with_options(ir, binding_metadata, VaporGenerateOptions::default())
}

/// Generate Vapor code from IR with explicit [`VaporGenerateOptions`].
pub fn generate_vapor_with_options(
    ir: &RootIRNode<'_>,
    binding_metadata: Option<&BindingMetadata>,
    options: VaporGenerateOptions,
) -> VaporGenerateResult {
    generate_vapor_with_options_and_experimentals(
        ir,
        binding_metadata,
        options,
        VaporGenerateExperimentalOptions::default(),
    )
}

/// Generate Vapor code from IR with explicit [`VaporGenerateOptions`] and
/// opt-in experimental context.
#[doc(hidden)]
pub fn generate_vapor_with_options_and_experimentals(
    ir: &RootIRNode<'_>,
    binding_metadata: Option<&BindingMetadata>,
    options: VaporGenerateOptions,
    experimental_options: VaporGenerateExperimentalOptions<'_>,
) -> VaporGenerateResult {
    let mut ctx = GenerateContext::new(
        &ir.element_template_map,
        &ir.standalone_text_elements,
        binding_metadata,
        ir.source,
    );
    ctx.jsx_closure = options.jsx_closure;
    ctx.component_name = experimental_options.component_name;
    ctx.experimental_self_component = experimental_options.self_component;

    if !ir.templates.is_empty() {
        ctx.use_helper("template");
    }

    let mut root_template_indices: FxHashSet<usize> = FxHashSet::default();
    if ir.block.returns.len() == 1 {
        let element_id = ir.block.returns[0];
        if let Some(&template_index) = ir.element_template_map.get(&element_id) {
            root_template_indices.insert(template_index);
        }
    }
    for op in ir.block.operation.iter() {
        if let OperationNode::If(if_node) = op {
            collect_root_if_templates(
                if_node,
                &ir.element_template_map,
                &mut root_template_indices,
            );
        }
    }

    let mut template_code = String::default();
    for (i, template) in ir.templates.iter().enumerate() {
        let is_root = root_template_indices.contains(&i);
        let is_svg = template.starts_with("<svg");
        match (is_root, is_svg) {
            (true, true) => writeln!(
                template_code,
                "const t{} = _template(\"{}\", true, 1)",
                i,
                escape_template(template)
            ),
            (true, false) => writeln!(
                template_code,
                "const t{} = _template(\"{}\", true)",
                i,
                escape_template(template)
            ),
            (false, true) => writeln!(
                template_code,
                "const t{} = _template(\"{}\", false, 1)",
                i,
                escape_template(template)
            ),
            (false, false) => writeln!(
                template_code,
                "const t{} = _template(\"{}\")",
                i,
                escape_template(template)
            ),
        }
        .ok();
    }

    collect_delegate_events(&mut ctx, &ir.block);
    ctx.push_line("export function render(_ctx) {");
    ctx.indent();

    if block_has_template_refs(&ir.block) {
        ctx.use_helper("createTemplateRefSetter");
        ctx.push_line("const _setRef = _ctx.vaporTemplateRefSetter || _createTemplateRefSetter()");
    }

    let custom_directives = collect_custom_directives(&ir.block);
    if !custom_directives.is_empty() {
        ctx.use_helper("resolveDirective");
        for directive in custom_directives {
            ctx.push_line(&vize_carton::cstr!(
                "const _directive_{} = _resolveDirective(\"{}\")",
                directive_resolution_ident(directive.as_str()),
                directive
            ));
        }
    }

    generate_block(&mut ctx, &ir.block, &ir.element_template_map);
    ctx.deindent();
    ctx.push_line("}");

    let mut delegate_code = String::default();
    if !ctx.delegate_events.is_empty() {
        ctx.use_helper("delegateEvents");
        let mut events: Vec<_> = ctx.delegate_events.iter().collect();
        events.sort();
        for event in events {
            writeln!(delegate_code, "_delegateEvents(\"{}\")", event).ok();
        }
    }

    let imports = generate_imports(&ctx);
    let mut final_code = imports;
    if !template_code.is_empty() {
        final_code.push_str(&template_code);
    }
    if !delegate_code.is_empty() {
        final_code.push_str(&delegate_code);
    }
    if !final_code.is_empty() {
        final_code.push('\n');
    }
    final_code.push_str(&ctx.code);

    VaporGenerateResult {
        code: final_code,
        templates: ir.templates.iter().map(|t| String::new(t)).collect(),
    }
}
