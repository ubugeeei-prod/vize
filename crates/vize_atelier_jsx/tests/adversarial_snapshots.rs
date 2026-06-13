//! Snapshot-heavy adversarial coverage for JSX/TSX lowering and codegen.
//!
//! These cases intentionally combine features that are easy to regress in
//! isolation-oriented tests: destructured props, setup state, nested list aliases,
//! slots, directives, SVG attributes, scoped styles, mixed output modes, and
//! parse/lowering diagnostics.

use std::fmt::Write as _;
use std::ops::Range;

use vize_atelier_jsx::{
    JsxCompileConfig, JsxCompileOutput, JsxComponent, JsxDiagnostic, JsxLang, JsxOutputMode,
    LowerOutput, VaporCompileOptions, VaporOutput, VdomCompileOptions, VdomOutput, compile_jsx,
    compile_to_vapor, compile_to_vdom, lower_source,
};
use vize_carton::Bump;
use vize_relief::{
    ExpressionNode, PropNode, RootNode, TemplateChildNode, TextCallContent,
    expressions::CompoundExpressionChild,
};

const STATEFUL_DESTRUCTURED_TSX: &str = r#"
import { computed, ref } from "vue";

type Row = {
  id: string;
  title: string;
  done: boolean;
};

type PanelProps = {
  rows: Row[];
  initialId?: string;
  dense?: boolean;
};

type PanelEmits = {
  select: [id: string];
};

const Panel = (
  { rows, initialId = rows[0]?.id, dense = false }: PanelProps,
  { emit, slots }: Ctx<PanelEmits, { footer: () => unknown }>,
) => {
  const activeId = ref(initialId);
  const activeRow = computed(() =>
    rows.find((row) => row.id === activeId.value),
  );

  const choose = (id: string) => {
    activeId.value = id;
    emit("select", id);
  };

  return (
    <section class={{ panel: true, dense }}>
      <header>
        <h2>{activeRow.value?.title ?? "Select a row"}</h2>
      </header>
      <ul>
        {rows.map((row, index) => (
          <li
            key={row.id}
            class={{ done: row.done, active: row.id === activeId.value }}
            data-index={index}
          >
            <button type="button" onClick={() => choose(row.id)}>
              <span>{row.title}</span>
              {row.id === activeId.value ? (
                <strong>Active</strong>
              ) : (
                <em>{index + 1}</em>
              )}
            </button>
          </li>
        ))}
      </ul>
      <footer>{slots.footer()}</footer>
    </section>
  );
};
"#;

const DIRECTIVES_SLOTS_SVG_JSX: &str = r#"
const Complex = ({ model, visible, raw, attrs, href, rows, focusOptions }) => (
  <form {...attrs} v-show={visible} v-on:submit={model.submit}>
    <input v-model={model.email} v-focus:lazy={focusOptions} />
    <div v-html={raw} />
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <use xlink:href={href} />
    </svg>
    <List rows={rows}>
      {{
        header: () => <h1>{model.email}</h1>,
        item: ({ row, index }) => (
          <span data-index={index}>{row.label}</span>
        ),
      }}
    </List>
  </form>
);
"#;

const TSX_SYNTAX_EDGES: &str = r#"
type Item<T extends string> = {
  key: T;
  value?: unknown;
};

const GenericList = <T extends string>({
  items,
  inputRef,
}: {
  items: Array<Item<T>>;
  inputRef?: HTMLInputElement | null;
}): JSX.Element => (
  <>
    <List<T> items={items} />
    <input ref={inputRef!} value={(items[0]?.key as string) satisfies string} />
    {items.map(({ key, value }, index) =>
      value ? (
        <output key={key}>{`${index}:${key}`}</output>
      ) : (
        <small key={key}>{key}</small>
      ),
    )}
  </>
);
"#;

const SCOPED_STYLE_EXTRACTION: &str = r#"
const Styled = ({ color, gap }) => (
  <article
    class="box"
    style={{
      "--box-color": color,
      "--box-gap": `${gap}px`,
    }}
  >
    <p>content</p>
    <style scoped>{`
      .box {
        color: var(--box-color);
        gap: var(--box-gap);
      }
    `}</style>
  </article>
);
"#;

const SCOPED_STYLE_INTERPOLATION_ESCAPE_HATCH: &str = r#"
const Styled = ({ color, gap }) => (
  <article class="box">
    <p>content</p>
    <style scoped>{`
      .box {
        color: ${color};
        gap: ${gap}px;
      }
    `}</style>
  </article>
);
"#;

const FRAGMENT_WHITESPACE_TEXT: &str = r#"
const Copy = ({ name, count }) => (
  <>
    Hello{" "}
    <strong>{name}</strong>
    {count > 1 ? (
      <span>{count} items</span>
    ) : (
      <span>one item</span>
    )}
    {/* JSX comments should not become visible children */}
  </>
);
"#;

const MIXED_MODES: &str = r#"
export const DefaultMode = () => <main>default</main>;

export const VaporOnly = () => {
  "use vue:vapor";
  return <section>{message}</section>;
};

export const VdomOnly = () => {
  "use vue:vdom";
  return <aside>{count}</aside>;
};
"#;

#[test]
fn stateful_destructured_component_snapshots() {
    assert_lower_snapshot(
        "stateful_destructured_component",
        STATEFUL_DESTRUCTURED_TSX,
        JsxLang::Tsx,
    );
    assert_vdom_snapshot(
        "stateful_destructured_component",
        STATEFUL_DESTRUCTURED_TSX,
        JsxLang::Tsx,
    );
    assert_vapor_snapshot(
        "stateful_destructured_component",
        STATEFUL_DESTRUCTURED_TSX,
        JsxLang::Tsx,
        VaporCompileOptions::default(),
    );
}

#[test]
fn directives_slots_svg_and_spreads_snapshots() {
    assert_lower_snapshot(
        "directives_slots_svg_and_spreads",
        DIRECTIVES_SLOTS_SVG_JSX,
        JsxLang::Jsx,
    );
    assert_vdom_snapshot(
        "directives_slots_svg_and_spreads",
        DIRECTIVES_SLOTS_SVG_JSX,
        JsxLang::Jsx,
    );
}

#[test]
fn tsx_syntax_edge_snapshots() {
    assert_lower_snapshot("tsx_syntax_edges", TSX_SYNTAX_EDGES, JsxLang::Tsx);
    assert_vdom_snapshot("tsx_syntax_edges", TSX_SYNTAX_EDGES, JsxLang::Tsx);
    assert_vapor_snapshot(
        "tsx_syntax_edges",
        TSX_SYNTAX_EDGES,
        JsxLang::Tsx,
        VaporCompileOptions::default(),
    );
}

#[test]
fn scoped_style_snapshots() {
    assert_lower_snapshot(
        "scoped_style_extraction",
        SCOPED_STYLE_EXTRACTION,
        JsxLang::Jsx,
    );
    assert_vdom_snapshot(
        "scoped_style_extraction",
        SCOPED_STYLE_EXTRACTION,
        JsxLang::Jsx,
    );
    assert_vapor_snapshot(
        "scoped_style_extraction",
        SCOPED_STYLE_EXTRACTION,
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
}

#[test]
fn scoped_style_interpolation_escape_hatch_snapshots() {
    assert_lower_snapshot(
        "scoped_style_interpolation_escape_hatch",
        SCOPED_STYLE_INTERPOLATION_ESCAPE_HATCH,
        JsxLang::Jsx,
    );
    assert_vdom_snapshot(
        "scoped_style_interpolation_escape_hatch",
        SCOPED_STYLE_INTERPOLATION_ESCAPE_HATCH,
        JsxLang::Jsx,
    );
}

#[test]
fn fragment_whitespace_text_snapshots() {
    assert_lower_snapshot(
        "fragment_whitespace_text",
        FRAGMENT_WHITESPACE_TEXT,
        JsxLang::Jsx,
    );
    assert_vdom_snapshot(
        "fragment_whitespace_text",
        FRAGMENT_WHITESPACE_TEXT,
        JsxLang::Jsx,
    );
    assert_vapor_snapshot(
        "fragment_whitespace_text",
        FRAGMENT_WHITESPACE_TEXT,
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
}

#[test]
fn mode_aware_multi_component_snapshot() {
    let bump = Bump::new();
    let output = compile_jsx(
        &bump,
        MIXED_MODES,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: JsxOutputMode::Vdom,
            ..JsxCompileConfig::default()
        },
    );

    insta::assert_debug_snapshot!(
        "mode_aware_multi_component_summary",
        compile_output_summary(MIXED_MODES, &output),
    );
    insta::assert_snapshot!(
        "mode_aware_multi_component_module",
        output.module_code().as_str()
    );
}

#[test]
fn invalid_and_ambiguous_source_diagnostics_snapshot() {
    let cases = [
        (
            "jsx_rejects_ts_type_annotation",
            "const App = ({ id }: { id: string }) => <div>{id}</div>;",
            JsxLang::Jsx,
        ),
        (
            "conflicting_mode_directives",
            "const App = () => {\n  \"use vue:vapor\";\n  \"use vue:vdom\";\n  return <div/>;\n};",
            JsxLang::Jsx,
        ),
        (
            "malformed_jsx_tree",
            "const App = () => <div><span></div>;",
            JsxLang::Jsx,
        ),
    ];

    let mut summaries = Vec::new();
    for (name, source, lang) in cases {
        let bump = Bump::new();
        let output = lower_source(&bump, source, lang);
        summaries.push(NamedDiagnostics {
            name,
            diagnostics: diagnostics_summary(source, &output.diagnostics),
        });
    }

    insta::assert_debug_snapshot!("invalid_and_ambiguous_source_diagnostics", summaries);
}

#[test]
fn vapor_ssr_snapshot() {
    assert_vapor_snapshot(
        "vapor_ssr_nested_control_flow",
        STATEFUL_DESTRUCTURED_TSX,
        JsxLang::Tsx,
        VaporCompileOptions { ssr: true },
    );
}

fn assert_lower_snapshot(name: &str, source: &str, lang: JsxLang) {
    let bump = Bump::new();
    let output = lower_source(&bump, source, lang);
    insta::assert_debug_snapshot!(format!("{name}_lower"), lower_summary(source, &output));
}

fn assert_vdom_snapshot(name: &str, source: &str, lang: JsxLang) {
    let bump = Bump::new();
    let output = compile_to_vdom(&bump, source, lang, VdomCompileOptions::default());
    insta::assert_debug_snapshot!(format!("{name}_vdom"), vdom_output_summary(source, &output));
}

fn assert_vapor_snapshot(name: &str, source: &str, lang: JsxLang, options: VaporCompileOptions) {
    let bump = Bump::new();
    let output = compile_to_vapor(&bump, source, lang, options);
    insta::assert_debug_snapshot!(
        format!("{name}_vapor"),
        vapor_output_summary(source, &output)
    );
}

fn lower_summary<'a>(source: &'a str, output: &'a LowerOutput<'a>) -> LowerSummary<'a> {
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

fn vdom_output_summary<'a>(source: &'a str, output: &'a VdomOutput) -> VdomOutputSummary<'a> {
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

fn vapor_output_summary<'a>(source: &'a str, output: &'a VaporOutput) -> VaporOutputSummary<'a> {
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

fn compile_output_summary<'a>(
    source: &'a str,
    output: &'a JsxCompileOutput,
) -> CompileOutputSummary<'a> {
    CompileOutputSummary {
        diagnostics: diagnostics_summary(source, &output.diagnostics),
        components: output
            .components
            .iter()
            .map(|component| match component {
                JsxComponent::Vdom(component) => RenderComponentSummary {
                    name: component.component_name.as_ref().map(|name| name.as_str()),
                    mode: "Vdom".to_string(),
                    preamble: Some(component.preamble.as_str()),
                    code: component.code.as_str(),
                    templates: Vec::new(),
                    scoped_style: component
                        .scoped_style
                        .as_ref()
                        .map(ScopedStyleSummary::from),
                },
                JsxComponent::Vapor(component) => RenderComponentSummary {
                    name: component.component_name.as_ref().map(|name| name.as_str()),
                    mode: "Vapor".to_string(),
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
                },
            })
            .collect(),
    }
}

fn diagnostics_summary(source: &str, diagnostics: &[JsxDiagnostic]) -> Vec<DiagnosticSummary> {
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
    let start = (range.start as usize).min(source.len());
    let end = (range.end as usize).min(source.len());
    if start <= end {
        &source[start..end]
    } else {
        ""
    }
}

impl<'a> From<&'a vize_atelier_jsx::ScopedStyle> for ScopedStyleSummary<'a> {
    fn from(style: &'a vize_atelier_jsx::ScopedStyle) -> Self {
        Self {
            scope_id: style.scope_id.as_str(),
            css: style.css.as_str(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct NamedDiagnostics<'a> {
    name: &'a str,
    diagnostics: Vec<DiagnosticSummary>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct LowerSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    roots: Vec<RootSummary<'a>>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct RootSummary<'a> {
    component_name: Option<&'a str>,
    mode: Option<String>,
    scoped_css: Option<&'a str>,
    scoped_style_exprs: Vec<ExprSpanSummary<'a>>,
    root: Vec<ChildSummary>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct ExprSpanSummary<'a> {
    content: &'a str,
    source: &'a str,
    range: Range<u32>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug)]
struct IfBranchSummary {
    condition: Option<String>,
    children: Vec<ChildSummary>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug)]
struct DiagnosticSummary {
    severity: String,
    message: String,
    source: String,
    range: Range<u32>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct VdomOutputSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    components: Vec<RenderComponentSummary<'a>>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct VaporOutputSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    components: Vec<RenderComponentSummary<'a>>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct CompileOutputSummary<'a> {
    diagnostics: Vec<DiagnosticSummary>,
    components: Vec<RenderComponentSummary<'a>>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct RenderComponentSummary<'a> {
    name: Option<&'a str>,
    mode: String,
    preamble: Option<&'a str>,
    code: &'a str,
    templates: Vec<&'a str>,
    scoped_style: Option<ScopedStyleSummary<'a>>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct ScopedStyleSummary<'a> {
    scope_id: &'a str,
    css: &'a str,
}
