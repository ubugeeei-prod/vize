//! Snapshot-heavy adversarial coverage for JSX/TSX lowering and codegen.
//!
//! These cases intentionally combine features that are easy to regress in
//! isolation-oriented tests: destructured props, setup state, nested list aliases,
//! slots, directives, SVG attributes, scoped styles, mixed output modes, and
//! parse/lowering diagnostics.
#![expect(clippy::disallowed_macros, reason = "insta uses format!")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

use vize_atelier_jsx::{
    JsxCompileConfig, JsxLang, JsxOutputMode, VaporCompileOptions, VdomCompileOptions, compile_jsx,
    compile_to_vapor, compile_to_vdom, lower_source, parse_module,
};
use vize_l0::Allocator;
mod adversarial_snapshots {
    pub(super) mod summary;
}
use adversarial_snapshots::summary::{
    NamedDiagnostics, compile_output_summary, diagnostics_summary, lower_summary,
    vapor_output_summary, vdom_output_summary,
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
        "tsx_syntax_edges_root_fallthrough_8625",
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
        "scoped_style_extraction_shared_scope_10701",
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
    let bump = Allocator::new();
    let output = compile_jsx(
        &bump,
        MIXED_MODES,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: JsxOutputMode::Vdom,
            ..JsxCompileConfig::default()
        },
    );

    capture_snapshot(
        "mode_aware_multi_component",
        MIXED_MODES,
        &compile_output_summary(MIXED_MODES, &output),
        Some(output.module_code().as_str()),
    );
    assert!(
        !parse_module(bump.as_oxc(), output.module_code().as_str(), JsxLang::Jsx).has_errors(),
        "the complete mixed-mode module retains valid helper declarations"
    );
    insta::assert_debug_snapshot!(
        "mode_aware_multi_component_summary_authored_native_10701_child_values_6890_inert_template_vdom_child_values_6890",
        compile_output_summary(MIXED_MODES, &output),
    );
    insta::assert_snapshot!(
        "mode_aware_multi_component_module_authored_native_10701_child_values_6890_inert_template_vdom_child_values_6890",
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
        let bump = Allocator::new();
        let output = lower_source(&bump, bump.as_oxc(), source, lang);
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
    let bump = Allocator::new();
    let output = lower_source(&bump, bump.as_oxc(), source, lang);
    insta::assert_debug_snapshot!(format!("{name}_lower"), lower_summary(source, &output));
}

fn assert_vdom_snapshot(name: &str, source: &str, lang: JsxLang) {
    let bump = Allocator::new();
    let output = compile_to_vdom(&bump, source, lang, VdomCompileOptions::default());
    let active = match name {
        "directives_slots_svg_and_spreads"
        | "fragment_whitespace_text"
        | "stateful_destructured_component"
        | "tsx_syntax_edges" => format!("{name}_vdom_vdom_child_values_6890"),
        _ => format!("{name}_vdom"),
    };
    insta::assert_debug_snapshot!(active, vdom_output_summary(source, &output));
}

fn assert_vapor_snapshot(name: &str, source: &str, lang: JsxLang, options: VaporCompileOptions) {
    let bump = Allocator::new();
    let output = compile_to_vapor(&bump, source, lang, options);
    capture_snapshot(name, source, &vapor_output_summary(source, &output), None);
    let active = match name {
        "stateful_destructured_component"
        | "tsx_syntax_edges_root_fallthrough_8625"
        | "fragment_whitespace_text" => format!("{name}_vapor_child_values_6890_inert_template"),
        _ => format!("{name}_vapor"),
    };
    insta::assert_debug_snapshot!(active, vapor_output_summary(source, &output));
}

/// Retain the complete observation before a snapshot failure can stop the test.
/// Capture never changes the expected snapshots or grants qualification credit.
fn capture_snapshot(
    name: &str,
    source: &str,
    summary: &impl std::fmt::Debug,
    module: Option<&str>,
) {
    let Ok(directory) = std::env::var("VIZE_JSX_ADVERSARIAL_CAPTURE") else {
        return;
    };
    let path = std::path::Path::new(&directory).join(format!("{name}.json"));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "source": source,
            "summary": format!("{summary:#?}"),
            "module": module,
        }))
        .unwrap(),
    )
    .unwrap();
}
