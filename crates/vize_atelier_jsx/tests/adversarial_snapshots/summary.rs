//! Pure structural summaries; snapshot assertions remain in the owning test module.

use std::fmt::Write as _;
use std::ops::Range;

use vize_atelier_jsx::{
    JsxCompileOutput, JsxComponent, JsxDiagnostic, LowerOutput, VaporOutput, VdomOutput,
};
use vize_relief::{
    ExpressionNode, PropNode, RootNode, TemplateChildNode, TextCallContent,
    expressions::CompoundExpressionChild,
};

pub(crate) fn lower_summary<'a>(source: &'a str, output: &'a LowerOutput<'a>) -> LowerSummary<'a> {
    LowerSummary {
        diagnostics: diagnostics_summary(source, &output.diagnostics),
        roots: output
            .roots
            .iter()
            .map(|root| RootSummary {
                component_name: root.component_name.as_ref().map(|name| name.as_str()),
                mode: root.mode.map(|mode| format!("{mode:?}")),
                scoped_css: root.scoped_css.as_ref().map(|css| css.as_str()),
                scoped_style_exprs: root
                    .scoped_style_exprs
                    .iter()
                    .map(|expr| ExprSpanSummary {
                        content: expr.content.as_str(),
                        source: source_slice(source, expr.start..expr.end),
                        range: expr.start..expr.end,
                    })
                    .collect(),
                root: root_summary(&root.root),
            })
            .collect(),
    }
}

fn root_summary(root: &RootNode<'_>) -> Vec<ChildSummary> {
    root.children.iter().map(child_summary).collect()
}

fn child_summary(child: &TemplateChildNode<'_>) -> ChildSummary {
    match child {
        TemplateChildNode::Element(element) => ChildSummary::Element {
            tag: element.tag.to_string(),
            tag_type: format!("{:?}", element.tag_type),
            is_self_closing: element.is_self_closing,
            props: element.props.iter().map(prop_summary).collect(),
            children: element.children.iter().map(child_summary).collect(),
        },
        TemplateChildNode::Text(text) => ChildSummary::Text {
            content: text.content.to_string(),
        },
        TemplateChildNode::Comment(comment) => ChildSummary::Comment {
            content: comment.content.to_string(),
        },
        TemplateChildNode::Interpolation(interpolation) => ChildSummary::Interpolation {
            expression: expression_summary(&interpolation.content),
        },
        TemplateChildNode::If(node) => ChildSummary::If {
            branches: node
                .branches
                .iter()
                .map(|branch| IfBranchSummary {
                    condition: branch.condition.as_ref().map(expression_summary),
                    children: branch.children.iter().map(child_summary).collect(),
                })
                .collect(),
        },
        TemplateChildNode::IfBranch(branch) => ChildSummary::IfBranch {
            condition: branch.condition.as_ref().map(expression_summary),
            children: branch.children.iter().map(child_summary).collect(),
        },
        TemplateChildNode::For(node) => ChildSummary::For {
            source: expression_summary(&node.source),
            value_alias: node.value_alias.as_ref().map(expression_summary),
            key_alias: node.key_alias.as_ref().map(expression_summary),
            object_index_alias: node.object_index_alias.as_ref().map(expression_summary),
            children: node.children.iter().map(child_summary).collect(),
        },
        TemplateChildNode::TextCall(node) => ChildSummary::TextCall {
            content: match &node.content {
                TextCallContent::Text(text) => format!("text:{}", text.content),
                TextCallContent::Interpolation(interpolation) => {
                    format!(
                        "interpolation:{}",
                        expression_summary(&interpolation.content)
                    )
                }
                TextCallContent::Compound(compound) => compound_summary(compound),
            },
        },
        TemplateChildNode::CompoundExpression(compound) => ChildSummary::Compound {
            expression: compound_summary(compound),
        },
        TemplateChildNode::Hoisted(index) => ChildSummary::Hoisted { index: *index },
    }
}

fn prop_summary(prop: &PropNode<'_>) -> PropSummary {
    match prop {
        PropNode::Attribute(attr) => PropSummary::Attribute {
            name: attr.name.to_string(),
            value: attr.value.as_ref().map(|value| value.content.to_string()),
        },
        PropNode::Directive(directive) => PropSummary::Directive {
            name: directive.name.to_string(),
            raw_name: directive.raw_name.as_ref().map(|name| name.to_string()),
            arg: directive.arg.as_ref().map(expression_summary),
            exp: directive.exp.as_ref().map(expression_summary),
            modifiers: directive
                .modifiers
                .iter()
                .map(|modifier| modifier.content.to_string())
                .collect(),
            shorthand: directive.shorthand,
        },
    }
}

fn expression_summary(expression: &ExpressionNode<'_>) -> String {
    match expression {
        ExpressionNode::Simple(simple) => {
            format!(
                "{}:{}",
                if simple.is_static {
                    "static"
                } else {
                    "dynamic"
                },
                simple.content
            )
        }
        ExpressionNode::Compound(compound) => compound_summary(compound),
    }
}

fn compound_summary(compound: &vize_relief::CompoundExpressionNode<'_>) -> String {
    let mut out = String::new();
    out.push_str("compound:[");
    for (index, child) in compound.children.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        match child {
            CompoundExpressionChild::Simple(simple) => {
                let _ = write!(
                    out,
                    "{}:{}",
                    if simple.is_static {
                        "static"
                    } else {
                        "dynamic"
                    },
                    simple.content
                );
            }
            CompoundExpressionChild::Compound(compound) => {
                out.push_str(&compound_summary(compound))
            }
            CompoundExpressionChild::Interpolation(interpolation) => {
                out.push_str(&expression_summary(&interpolation.content));
            }
            CompoundExpressionChild::Text(text) => {
                let _ = write!(out, "text:{}", text.content);
            }
            CompoundExpressionChild::String(value) => {
                let _ = write!(out, "string:{value}");
            }
            CompoundExpressionChild::Symbol(symbol) => {
                let _ = write!(out, "symbol:{symbol:?}");
            }
        }
    }
    out.push(']');
    out
}

pub(crate) fn vdom_output_summary<'a>(
    source: &'a str,
    output: &'a VdomOutput,
) -> VdomOutputSummary<'a> {
    VdomOutputSummary {
        diagnostics: diagnostics_summary(source, &output.diagnostics),
        components: output
            .components
            .iter()
            .map(|component| RenderComponentSummary {
                name: component.component_name.as_ref().map(|name| name.as_str()),
                mode: format!("{:?}", component.mode),
                preamble: Some(component.preamble.as_str()),
                code: component.code.as_str(),
                templates: Vec::new(),
                scoped_style: component
                    .scoped_style
                    .as_ref()
                    .map(ScopedStyleSummary::from),
            })
            .collect(),
    }
}

pub(crate) fn vapor_output_summary<'a>(
    source: &'a str,
    output: &'a VaporOutput,
) -> VaporOutputSummary<'a> {
    VaporOutputSummary {
        diagnostics: diagnostics_summary(source, &output.diagnostics),
        components: output
            .components
            .iter()
            .map(|component| RenderComponentSummary {
                name: component.component_name.as_ref().map(|name| name.as_str()),
                mode: format!("{:?}", component.mode),
                preamble: None,
                code: component.code.as_str(),
                templates: component
                    .templates
                    .iter()
                    .map(|template| template.as_str())
                    .collect(),
                scoped_style: component
                    .scoped_style
                    .as_ref()
                    .map(ScopedStyleSummary::from),
            })
            .collect(),
    }
}

pub(crate) fn compile_output_summary<'a>(
    source: &'a str,
    output: &'a JsxCompileOutput,
) -> CompileOutputSummary<'a> {
    CompileOutputSummary {
        diagnostics: diagnostics_summary(source, &output.diagnostics),
        components: output
            .components
            .iter()
            .map(|component| {
                let templates = match component {
                    JsxComponent::Vapor(component) => component
                        .templates
                        .iter()
                        .map(|template| template.as_str())
                        .collect(),
                    JsxComponent::Vdom(_) | JsxComponent::Ssr(_) => Vec::new(),
                };
                let mode = match component {
                    JsxComponent::Vdom(_) => "Vdom",
                    JsxComponent::Vapor(_) => "Vapor",
                    JsxComponent::Ssr(_) => "Ssr",
                };
                RenderComponentSummary {
                    name: component.component_name(),
                    mode: mode.to_string(),
                    preamble: matches!(component, JsxComponent::Vdom(_))
                        .then(|| component.preamble()),
                    code: component.code(),
                    templates,
                    scoped_style: component.scoped_style().map(ScopedStyleSummary::from),
                }
            })
            .collect(),
    }
}

pub(crate) fn diagnostics_summary(
    source: &str,
    diagnostics: &[JsxDiagnostic],
) -> Vec<DiagnosticSummary> {
    diagnostics
        .iter()
        .map(|diagnostic| DiagnosticSummary {
            severity: format!("{:?}", diagnostic.severity),
            message: diagnostic.message.to_string(),
            source: source_slice(source, diagnostic.start..diagnostic.end).to_string(),
            range: diagnostic.start..diagnostic.end,
        })
        .collect()
}

fn source_slice(source: &str, range: Range<u32>) -> &str {
    let clamp = |offset: u32| (offset as usize).min(source.len());
    source
        .get(clamp(range.start)..clamp(range.end))
        .unwrap_or_default()
}

impl<'a> From<&'a vize_atelier_jsx::ScopedStyle> for ScopedStyleSummary<'a> {
    fn from(style: &'a vize_atelier_jsx::ScopedStyle) -> Self {
        Self {
            scope_id: style.scope_id.as_str(),
            css: style.css.as_str(),
        }
    }
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
pub(crate) struct NamedDiagnostics<'a> {
    pub(crate) name: &'a str,
    pub(crate) diagnostics: Vec<DiagnosticSummary>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
pub(crate) struct LowerSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    roots: Vec<RootSummary<'a>>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
struct RootSummary<'a> {
    component_name: Option<&'a str>,
    mode: Option<String>,
    scoped_css: Option<&'a str>,
    scoped_style_exprs: Vec<ExprSpanSummary<'a>>,
    root: Vec<ChildSummary>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
struct ExprSpanSummary<'a> {
    content: &'a str,
    source: &'a str,
    range: Range<u32>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
enum ChildSummary {
    Element {
        tag: String,
        tag_type: String,
        is_self_closing: bool,
        props: Vec<PropSummary>,
        children: Vec<ChildSummary>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
    Interpolation {
        expression: String,
    },
    If {
        branches: Vec<IfBranchSummary>,
    },
    IfBranch {
        condition: Option<String>,
        children: Vec<ChildSummary>,
    },
    For {
        source: String,
        value_alias: Option<String>,
        key_alias: Option<String>,
        object_index_alias: Option<String>,
        children: Vec<ChildSummary>,
    },
    TextCall {
        content: String,
    },
    Compound {
        expression: String,
    },
    Hoisted {
        index: usize,
    },
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
struct IfBranchSummary {
    condition: Option<String>,
    children: Vec<ChildSummary>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
enum PropSummary {
    Attribute {
        name: String,
        value: Option<String>,
    },
    Directive {
        name: String,
        raw_name: Option<String>,
        arg: Option<String>,
        exp: Option<String>,
        modifiers: Vec<String>,
        shorthand: bool,
    },
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
pub(crate) struct DiagnosticSummary {
    severity: String,
    message: String,
    source: String,
    range: Range<u32>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
pub(crate) struct VdomOutputSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    components: Vec<RenderComponentSummary<'a>>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
pub(crate) struct VaporOutputSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    components: Vec<RenderComponentSummary<'a>>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
pub(crate) struct CompileOutputSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    components: Vec<RenderComponentSummary<'a>>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
struct RenderComponentSummary<'a> {
    name: Option<&'a str>,
    mode: String,
    preamble: Option<&'a str>,
    code: &'a str,
    templates: Vec<&'a str>,
    scoped_style: Option<ScopedStyleSummary<'a>>,
}

#[expect(dead_code, reason = "fields are read through Debug snapshots")]
#[derive(Debug)]
struct ScopedStyleSummary<'a> {
    scope_id: &'a str,
    css: &'a str,
}
