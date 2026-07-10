//! Graph-native virtual TypeScript generated from owned Croquis semantics.
//!
//! The provider never parses source text. Frontends own parsing and publish a
//! [`CroquisSemanticProduct`]; Canon turns that cached product into deterministic
//! declarations and template-expression guards.

mod flow_context;
use crate::Span;
use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError,
};
use vize_carton::{String, cstr, source_anchor::SourceAnchor};
use vize_croquis::{CroquisSemanticProduct, CroquisSemanticSnapshot};
use vize_flow::{BlockId, FlowGraph, FlowProduct};

/// Why a generated span exists in semantic virtual TypeScript.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SemanticVirtualTsMappingKind {
    BindingDeclaration,
    TemplateExpression,
}

/// Exact generated-to-semantic-source mapping.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SemanticVirtualTsMapping {
    pub generated: Span,
    pub source: Span,
    /// Stable source revision whose coordinate space contains `source`.
    pub source_anchor: Option<SourceAnchor>,
    pub kind: SemanticVirtualTsMappingKind,
    /// Flow block selected by source-provenance matching, when available.
    pub flow_block: Option<BlockId>,
    /// Immediate dominator of `flow_block`, when the block is reachable.
    pub immediate_dominator: Option<BlockId>,
}

/// Owned virtual TypeScript artifact suitable for type-checker/LSP consumers.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SemanticVirtualTsOutput {
    pub code: String,
    pub mappings: Vec<SemanticVirtualTsMapping>,
    pub binding_declaration_count: usize,
    pub expression_guard_count: usize,
    /// Reachable Flow blocks inspected while generating this artifact.
    pub reachable_block_count: usize,
    /// Reachable non-entry blocks with an immediate dominator.
    pub dominated_block_count: usize,
    /// Template expressions joined to Flow nodes by stable source provenance.
    pub flow_mapped_expression_count: usize,
    /// Joined expressions retained for diagnostics despite unreachable blocks.
    pub unreachable_expression_count: usize,
}

/// Atlas identity for Canon's semantic virtual TypeScript output.
pub struct CanonSemanticVirtualTsProduct;
impl Product for CanonSemanticVirtualTsProduct {
    type Value = SemanticVirtualTsOutput;

    const NAME: &'static str = "canon.semantic-virtual-ts";
}

/// Provider for parser-free semantic virtual TypeScript generation.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonSemanticVirtualTsProvider;
impl Provider for CanonSemanticVirtualTsProvider {
    type Product = CanonSemanticVirtualTsProduct;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<CroquisSemanticProduct>(),
            ProductId::of::<FlowProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SemanticVirtualTsOutput, ProviderError> {
        let semantics = context.get::<CroquisSemanticProduct>()?;
        let flow = context.get::<FlowProduct>()?;
        Ok(generate_semantic_virtual_ts_with_flow(&semantics, &flow))
    }
}

/// Registration handle for Canon's graph-native semantic path.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonSemanticVirtualTsRecipe;

impl CanonSemanticVirtualTsRecipe {
    /// Register the output provider in an Atlas compilation.
    pub fn register(self, compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
        compilation.register_provider(CanonSemanticVirtualTsProvider)
    }

    /// Root product requested by this recipe.
    pub fn product(self) -> ProductId {
        ProductId::of::<CanonSemanticVirtualTsProduct>()
    }
}

/// Register Canon's graph-native semantic virtual TypeScript recipe.
pub fn register_semantic_virtual_ts_recipe(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    CanonSemanticVirtualTsRecipe.register(compilation)
}

#[derive(Clone, Copy)]
struct BindingAnchor<'a> {
    name: &'a str,
    range: Option<Span>,
}

/// Generate deterministic virtual TypeScript without reading parser syntax.
pub fn generate_semantic_virtual_ts(
    semantics: &CroquisSemanticSnapshot,
) -> SemanticVirtualTsOutput {
    generate_semantic_virtual_ts_inner(semantics, None)
}

/// Generate Virtual TypeScript while consuming shared control/data/effect flow.
pub fn generate_semantic_virtual_ts_with_flow(
    semantics: &CroquisSemanticSnapshot,
    flow: &FlowGraph,
) -> SemanticVirtualTsOutput {
    generate_semantic_virtual_ts_inner(semantics, Some(flow))
}

fn generate_semantic_virtual_ts_inner(
    semantics: &CroquisSemanticSnapshot,
    flow: Option<&FlowGraph>,
) -> SemanticVirtualTsOutput {
    let mut output = SemanticVirtualTsOutput::default();
    output
        .code
        .push_str("// Virtual TypeScript from the shared Croquis semantic product\n");
    output
        .code
        .push_str("// This artifact contains no parser-owned references.\n\n");

    let expressions = flow_context::plan_expressions(semantics, flow);
    if flow.is_some() {
        output.reachable_block_count = expressions.reachable_block_count;
        output.dominated_block_count = expressions.dominated_block_count;
        output.flow_mapped_expression_count = expressions.mapped_expression_count;
        output.unreachable_expression_count = expressions.unreachable_expression_count;
        output.code.push_str(&cstr!(
            "// Flow: {} reachable block(s), {} dominated block(s), {} mapped expression(s).\n\n",
            output.reachable_block_count,
            output.dominated_block_count,
            output.flow_mapped_expression_count
        ));
    }

    let mut bindings = Vec::with_capacity(
        semantics.bindings.len()
            + semantics
                .scopes
                .iter()
                .map(|scope| scope.bindings.len())
                .sum::<usize>()
            + semantics.reactive_sources.len(),
    );
    bindings.extend(semantics.bindings.iter().map(|binding| BindingAnchor {
        name: binding.name.as_str(),
        range: binding.range.map(|range| Span::new(range.start, range.end)),
    }));
    for scope in &semantics.scopes {
        bindings.extend(scope.bindings.iter().map(|binding| {
            BindingAnchor {
                name: binding.name.as_str(),
                range: Some(Span::new(
                    binding.declaration_offset,
                    binding
                        .declaration_offset
                        .saturating_add(binding.name.len() as u32),
                )),
            }
        }));
    }
    bindings.extend(semantics.reactive_sources.iter().map(|binding| {
        BindingAnchor {
            name: binding.name.as_str(),
            range: Some(Span::new(
                binding.declaration_offset,
                binding
                    .declaration_offset
                    .saturating_add(binding.name.len() as u32),
            )),
        }
    }));
    bindings.sort_by(|left, right| {
        (left.name, left.range.is_none()).cmp(&(right.name, right.range.is_none()))
    });

    let mut previous_name = None;
    for binding in bindings {
        if previous_name == Some(binding.name) {
            continue;
        }
        previous_name = Some(binding.name);
        if !is_typescript_identifier(binding.name) || is_typescript_keyword(binding.name) {
            continue;
        }

        output.code.push_str("declare const ");
        let generated_start = output.code.len();
        output.code.push_str(binding.name);
        let generated_end = output.code.len();
        output.code.push_str(": any;\n");
        output.binding_declaration_count += 1;

        if let Some(source) = binding.range {
            output.mappings.push(SemanticVirtualTsMapping {
                generated: span(generated_start, generated_end),
                source,
                source_anchor: semantics.source_anchor,
                kind: SemanticVirtualTsMappingKind::BindingDeclaration,
                flow_block: None,
                immediate_dominator: None,
            });
        }
    }

    if output.binding_declaration_count > 0 {
        output.code.push('\n');
    }
    for planned in expressions.expressions {
        let index = planned.original_index;
        let expression = planned.expression;
        if expression.content.trim().is_empty() {
            continue;
        }
        if planned.flow.is_some_and(|flow| !flow.reachable) {
            output.code.push_str(&cstr!(
                "// Flow-unreachable block for expression {index}; retained for diagnostics.\n"
            ));
        }
        let guard_name = cstr!("__vize_semantic_expression_{index}");
        output.code.push_str("const ");
        output.code.push_str(&guard_name);
        output.code.push_str(" = () => {\n");

        let has_guard = expression
            .vif_guard
            .as_deref()
            .is_some_and(|guard| !guard.trim().is_empty());
        if let Some(guard) = expression.vif_guard.as_deref().filter(|_| has_guard) {
            output.code.push_str("  if (");
            output.code.push_str(guard);
            output.code.push_str(") {\n");
        }

        let indent = if has_guard { "    " } else { "  " };
        output.code.push_str(indent);
        if expression.kind != "vOn" {
            output.code.push_str("void (");
        }
        let generated_start = output.code.len();
        output.code.push_str(&expression.content);
        let generated_end = output.code.len();
        if expression.kind != "vOn" {
            output.code.push(')');
        }
        output.code.push_str(";\n");
        if has_guard {
            output.code.push_str("  }\n");
        }
        output.code.push_str("};\n");
        output.expression_guard_count += 1;
        output.mappings.push(SemanticVirtualTsMapping {
            generated: span(generated_start, generated_end),
            source: Span::new(expression.range.start, expression.range.end),
            source_anchor: semantics.source_anchor,
            kind: SemanticVirtualTsMappingKind::TemplateExpression,
            flow_block: planned.flow.map(|flow| flow.block),
            immediate_dominator: planned.flow.and_then(|flow| flow.immediate_dominator),
        });
    }

    output
}

fn span(start: usize, end: usize) -> Span {
    Span::new(
        u32::try_from(start).unwrap_or(u32::MAX),
        u32::try_from(end).unwrap_or(u32::MAX),
    )
}

fn is_typescript_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || matches!(first, b'_' | b'$')) {
        return false;
    }
    bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}

fn is_typescript_keyword(name: &str) -> bool {
    matches!(
        name,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

#[cfg(test)]
mod tests;
