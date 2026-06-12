//! `<style scoped>` JSX/TSX blocks (#1495).
//!
//! A `<style scoped>` element inside a component is extracted at compile time:
//! it is *not* rendered as a runtime `<style>` vnode, its CSS is scoped-rewritten
//! (reusing the SFC scoped-CSS transform), a `data-v-<hash>` scope id is
//! generated, and that scope attribute is injected into the component's other
//! rendered elements — in both the VDOM and Vapor backends. The rewritten CSS +
//! scope id are exposed on the compiled component for a bundler to emit later.

use vize_atelier_jsx::{
    DomCompileOptions, JsxLang, VaporCompileOptions, compile_to_dom, compile_to_vapor,
};
use vize_carton::{Bump, cstr};

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

fn dom(src: &str) -> vize_atelier_jsx::DomComponent {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, JsxLang::Jsx, DomCompileOptions::default());
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

// --- VDOM ---------------------------------------------------------------------

#[test]
fn vdom_does_not_emit_a_style_element_vnode() {
    let component = dom(SCOPED);
    // The `<style scoped>` block must be extracted, never rendered.
    assert!(
        !component.code.contains("_createElementBlock(\"style\""),
        "style element leaked into VDOM output:\n{}",
        component.code
    );
    assert!(
        !component.code.contains("color: red"),
        "raw CSS leaked into the render code:\n{}",
        component.code
    );
}

#[test]
fn vdom_injects_scope_attr_onto_sibling_element() {
    let component = dom(SCOPED);
    let scope_id = component
        .scoped_style
        .as_ref()
        .expect("scoped style")
        .scope_id
        .clone();
    assert!(scope_id.starts_with("data-v-"), "scope id: {scope_id}");
    // The `<div class="box">` sibling carries the scope attribute as a prop.
    let expected = cstr!("\"{scope_id}\": \"\"");
    assert!(
        component.code.contains(expected.as_str()),
        "expected scope attr {expected:?} on the div:\n{}",
        component.code
    );
}

#[test]
fn vdom_exposes_rewritten_scoped_css() {
    let component = dom(SCOPED);
    let style = component.scoped_style.as_ref().expect("scoped style");
    // Matches the SFC `apply_scoped_css` output shape: the scope attribute is
    // injected before the rule block.
    let expected = cstr!(".box[{}]", style.scope_id);
    assert!(
        style.css.contains(expected.as_str()),
        "rewritten CSS {:?} should contain {expected:?}",
        style.css
    );
    assert!(style.css.contains("color: red"), "css: {:?}", style.css);
}

#[test]
fn vdom_without_scoped_style_is_unchanged() {
    let component = dom(PLAIN);
    assert!(component.scoped_style.is_none());
    assert!(
        !component.code.contains("data-v-"),
        "no scope attr expected without <style scoped>:\n{}",
        component.code
    );
    assert!(
        component.code.contains("class: \"box\""),
        "{}",
        component.code
    );
}

// --- Vapor --------------------------------------------------------------------

#[test]
fn vapor_does_not_emit_a_style_template() {
    let component = vapor(SCOPED);
    assert!(
        !component.code.contains("<style"),
        "style element leaked into Vapor output:\n{}",
        component.code
    );
    assert!(
        !component.code.contains("color: red"),
        "raw CSS leaked into the Vapor code:\n{}",
        component.code
    );
}

#[test]
fn vapor_injects_scope_attr_into_template() {
    let component = vapor(SCOPED);
    let scope_id = component
        .scoped_style
        .as_ref()
        .expect("scoped style")
        .scope_id
        .clone();
    assert!(scope_id.starts_with("data-v-"), "scope id: {scope_id}");
    // The scope attribute is baked into the static template string for the div.
    let expected = cstr!("<div {scope_id}");
    assert!(
        component.code.contains(expected.as_str()),
        "expected {expected:?} in Vapor template:\n{}",
        component.code
    );
    // And into the separately-collected templates vector.
    assert!(
        component
            .templates
            .iter()
            .any(|t| t.contains(expected.as_str())),
        "expected {expected:?} in templates: {:?}",
        component.templates
    );
}

#[test]
fn vapor_exposes_rewritten_scoped_css() {
    let component = vapor(SCOPED);
    let style = component.scoped_style.as_ref().expect("scoped style");
    let expected = cstr!(".box[{}]", style.scope_id);
    assert!(
        style.css.contains(expected.as_str()),
        "rewritten CSS {:?} should contain {expected:?}",
        style.css
    );
}

#[test]
fn vapor_without_scoped_style_is_unchanged() {
    let component = vapor(PLAIN);
    assert!(component.scoped_style.is_none());
    assert!(
        !component.code.contains("data-v-"),
        "no scope attr expected without <style scoped>:\n{}",
        component.code
    );
}

// --- Cross-cutting ------------------------------------------------------------

#[test]
fn dom_and_vapor_agree_on_scope_id() {
    let d = dom(SCOPED);
    let v = vapor(SCOPED);
    assert_eq!(
        d.scoped_style.unwrap().scope_id,
        v.scoped_style.unwrap().scope_id,
        "VDOM and Vapor should derive the same scope id for the same component"
    );
}

#[test]
fn non_scoped_style_element_still_renders() {
    // A `<style>` *without* `scoped` is a normal element, not extracted.
    let src = r#"const C = () => <div><style>{`.x{}`}</style></div>;"#;
    let component = dom(src);
    assert!(
        component.scoped_style.is_none(),
        "non-scoped style must not be extracted"
    );
    assert!(
        component.code.contains("\"style\""),
        "non-scoped <style> should render as an element:\n{}",
        component.code
    );
}

#[test]
fn tsx_components_support_scoped_styles() {
    let bump = Bump::new();
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
    let out = compile_to_dom(&bump, src, JsxLang::Tsx, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    let component = &out.components[0];
    let style = component.scoped_style.as_ref().expect("scoped style");
    assert!(
        style
            .css
            .contains(cstr!(".box[{}]", style.scope_id).as_str())
    );
    assert!(
        component
            .code
            .contains(cstr!("\"{}\": \"\"", style.scope_id).as_str())
    );
}
