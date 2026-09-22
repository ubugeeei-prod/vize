//! Public Vapor codegen entrypoints.

use std::fmt::Write;

use crate::ir::{OperationNode, RootIRNode};
use vize_atelier_core::{codegen::document::EmitDocument, options::BindingMetadata};
use vize_carton::{FxHashSet, String, cstr};

use super::context::GenerateContext;
use super::setup::generate_imports;
use super::spans::{TEMPLATE_ESCAPES, VaporSourceSpans};
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
    /// Source Map v3 JSON for the generated render code.
    pub map: Option<String>,
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
    /// Generate a Source Map v3 document for Vapor render code.
    pub source_map: bool,
    /// Filename recorded in the Source Map v3 `file` and `sources` fields.
    pub source_map_filename: Option<&'a str>,
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
    generate_vapor_with_spans(ir, binding_metadata, options, experimental_options, None)
}

/// Generate Vapor code, taking the authored anchors a map-requesting compile
/// collected beyond the IR (Davinci P3-9). Without them, a requested map
/// still carries the render entry and every IR expression's anchors.
pub(crate) fn generate_vapor_with_spans(
    ir: &RootIRNode<'_>,
    binding_metadata: Option<&BindingMetadata>,
    options: VaporGenerateOptions,
    experimental_options: VaporGenerateExperimentalOptions<'_>,
    spans: Option<&VaporSourceSpans>,
) -> VaporGenerateResult {
    let ir_only = VaporSourceSpans::default();
    let spans = spans.or_else(|| experimental_options.source_map.then_some(&ir_only));
    let mut ctx = GenerateContext::new(
        &ir.element_template_map,
        &ir.standalone_text_elements,
        binding_metadata,
        ir.source,
    );
    ctx.jsx_closure = options.jsx_closure;
    ctx.component_name = experimental_options.component_name;
    ctx.experimental_self_component = experimental_options.self_component;
    ctx.spans = spans;
    ctx.out = EmitDocument::with_capacity(4096, spans.is_some());

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

    // Template strings keep the links lowering gave them through escaping.
    let mut template_code = EmitDocument::new(spans.is_some());
    for (i, template) in ir.templates.iter().enumerate() {
        let links = spans
            .and_then(|spans| spans.templates.get(&i))
            .cloned()
            .unwrap_or_default();
        template_code.push_str(&cstr!("const t{i} = _template(\""));
        template_code.push_escaped(
            &EmitDocument::from_parts(String::new(template), links, Vec::new()),
            &TEMPLATE_ESCAPES,
        );
        template_code.push_str(
            match (
                root_template_indices.contains(&i),
                template.starts_with("<svg"),
            ) {
                (true, true) => "\", true, 1)\n",
                (true, false) => "\", true)\n",
                (false, true) => "\", false, 1)\n",
                (false, false) => "\")\n",
            },
        );
    }

    let render = ctx.spanned_at("export function render(_ctx) {", spans.map(|s| s.root));
    ctx.push_line_spanned(&render);
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

    let mut document = EmitDocument::new(spans.is_some());
    document.push_str(&generate_imports(&ctx));
    document.push_spanned(&template_code);
    document.push_str(&delegate_code);
    if !document.is_empty() {
        document.push_char('\n');
    }
    document.push_spanned(&ctx.out);
    let map = spans.map(|_| {
        let filename = experimental_options
            .source_map_filename
            .unwrap_or("template.vue");
        document.source_map(filename, ir.source)
    });

    VaporGenerateResult {
        code: document.into_string(),
        templates: ir.templates.iter().map(|t| String::new(t)).collect(),
        map,
    }
}
