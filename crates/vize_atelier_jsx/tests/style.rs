//! `<style scoped>` JSX/TSX blocks (#1495).

use std::fmt::Write as _;
use vize_atelier_jsx::{
    DomCompileOptions, JsxLang, VaporCompileOptions, compile_to_dom, compile_to_vapor, lower_source,
};
use vize_carton::Bump;

const SCOPED: &str = r#"
const Comp = () => (
  <>
    <div class="box">hi</div>
    <style scoped>{`
      .box {
        color: red;
      }
    `}</style>
  </>
);
"#;

const PLAIN: &str = r#"
const Comp = () => (
  <>
    <div class="box">hi</div>
  </>
);
"#;

fn dom(src: &str, lang: JsxLang) -> vize_atelier_jsx::DomComponent {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, lang, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap()
}

fn vapor(src: &str) -> vize_atelier_jsx::VaporComponent {
    let bump = Bump::new();
    let out = compile_to_vapor(&bump, src, JsxLang::Jsx, VaporCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap()
}

fn dom_snapshot(component: &vize_atelier_jsx::DomComponent) -> std::string::String {
    let mut snapshot = std::string::String::new();
    writeln!(snapshot, "## code").unwrap();
    snapshot.push_str(component.code.as_str());
    writeln!(snapshot, "\n## scoped_style").unwrap();
    match &component.scoped_style {
        Some(style) => {
            writeln!(snapshot, "scope_id: {}", style.scope_id).unwrap();
            writeln!(snapshot, "css:").unwrap();
            snapshot.push_str(style.css.as_str());
        }
        None => snapshot.push_str("none"),
    }
    snapshot
}

fn vapor_snapshot(component: &vize_atelier_jsx::VaporComponent) -> std::string::String {
    let mut snapshot = std::string::String::new();
    writeln!(snapshot, "## code").unwrap();
    snapshot.push_str(component.code.as_str());
    writeln!(snapshot, "\n## templates").unwrap();
    for template in &component.templates {
        writeln!(snapshot, "{template}").unwrap();
    }
    writeln!(snapshot, "## scoped_style").unwrap();
    match &component.scoped_style {
        Some(style) => {
            writeln!(snapshot, "scope_id: {}", style.scope_id).unwrap();
            writeln!(snapshot, "css:").unwrap();
            snapshot.push_str(style.css.as_str());
        }
        None => snapshot.push_str("none"),
    }
    snapshot
}

#[test]
fn vdom_scoped_style_snapshot() {
    let component = dom(SCOPED, JsxLang::Jsx);

    insta::assert_snapshot!(dom_snapshot(&component));
}

#[test]
fn vapor_scoped_style_snapshot() {
    let component = vapor(SCOPED);

    insta::assert_snapshot!(vapor_snapshot(&component));
}

#[test]
fn vdom_without_scoped_style_snapshot() {
    let component = dom(PLAIN, JsxLang::Jsx);

    assert!(component.scoped_style.is_none());
    insta::assert_snapshot!(dom_snapshot(&component));
}

#[test]
fn vapor_without_scoped_style_snapshot() {
    let component = vapor(PLAIN);

    assert!(component.scoped_style.is_none());
    insta::assert_snapshot!(vapor_snapshot(&component));
}

#[test]
fn dom_and_vapor_agree_on_scope_id() {
    let d = dom(SCOPED, JsxLang::Jsx);
    let v = vapor(SCOPED);

    assert_eq!(
        d.scoped_style.unwrap().scope_id,
        v.scoped_style.unwrap().scope_id
    );
}

#[test]
fn scoped_style_interpolations_are_recovered_with_source_spans() {
    let bump = Bump::new();
    let src = r#"const Comp = (props: { color: string }) => (
  <>
    <div class="box"/>
    <style scoped>{`.box { color: ${props.color}; border: ${props.color}; }`}</style>
  </>
);
"#;
    let out = lower_source(&bump, src, JsxLang::Tsx);
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    let root = &out.roots[0];

    let exprs: Vec<_> = root
        .scoped_style_exprs
        .iter()
        .map(|expr| {
            (
                expr.content.as_str(),
                &src[expr.start as usize..expr.end as usize],
            )
        })
        .collect();
    assert_eq!(
        exprs,
        vec![
            ("props.color", "props.color"),
            ("props.color", "props.color")
        ]
    );
    insta::assert_snapshot!(root.scoped_css.as_deref().expect("scoped css"));
}

#[test]
fn static_scoped_style_has_no_interpolations() {
    let bump = Bump::new();
    let out = lower_source(&bump, SCOPED, JsxLang::Jsx);

    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert!(out.roots[0].scoped_style_exprs.is_empty());
}

#[test]
fn non_scoped_style_element_still_renders() {
    let src = r#"const C = () => <div><style>{`.x{}`}</style></div>;"#;
    let component = dom(src, JsxLang::Jsx);

    assert!(component.scoped_style.is_none());
    insta::assert_snapshot!(component.code.as_str());
}

#[test]
fn tsx_components_support_scoped_styles() {
    let src = r#"
const Comp = (): any => (
  <>
    <div class="box">hi</div>
    <style scoped>{`
      .box {
        color: red;
      }
    `}</style>
  </>
);
"#;
    let component = dom(src, JsxLang::Tsx);

    insta::assert_snapshot!(dom_snapshot(&component));
}
