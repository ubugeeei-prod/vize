use super::super::{json, trace};

/// A child the HTML parser would re-parent out of one template string mounts
/// and updates where it was written, as the VDOM renders it.
#[test]
fn invalid_nesting_mounts_the_authored_tree() {
    for template in [
        "<form><form><button>{{ state.label }}</button></form></form>",
        "<p><ul><li>{{ state.label }}</li></ul></p>",
        "<p>one<p>{{ state.label }}</p></p>",
        "<a href=\"/x\"><a href=\"/y\">{{ state.label }}</a></a>",
        "<table><tr><td>{{ state.label }}</td></tr></table>",
        "<p>a<div>{{ state.label }}</div><span>b</span><div>y</div></p>",
        "<div><p><div>x</div></p><i>{{ state.label }}</i></div>",
    ] {
        let source = format!(
            "<script setup>import {{ state }} from './fixture';</script><template>{template}</template>"
        );
        let extra = json!({"steps": [{"patch": {"label": "updated"}}]});
        let dom = trace(&source, "vdom", extra.clone());
        assert_eq!(trace(&source, "vapor", extra), dom, "{template}");
    }
}
