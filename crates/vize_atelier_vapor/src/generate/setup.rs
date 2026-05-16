//! Import generation, delegate event collection, and template escaping.

use super::context::GenerateContext;
use crate::ir::{BlockIRNode, OperationNode};
use vize_carton::{cstr, String};

/// Collect delegate events from block
pub(crate) fn collect_delegate_events(ctx: &mut GenerateContext, block: &BlockIRNode<'_>) {
    for op in block.operation.iter() {
        if let OperationNode::SetEvent(set_event) = op {
            if set_event.delegate {
                ctx.add_delegate_event(&set_event.key.content);
            }
        }
    }
}

/// Generate imports based on used helpers
pub(crate) fn generate_imports(ctx: &GenerateContext) -> String {
    if ctx.used_helpers.is_empty() {
        return String::default();
    }

    // Define priority order for helpers (lower = earlier in import)
    fn helper_priority(name: &str) -> u32 {
        match name {
            "resolveComponent" => 1,
            "createComponentWithFallback" => 2,
            "createComponent" => 3,
            "createDynamicComponent" => 4,
            "VaporTeleport" => 5,
            "VaporKeepAlive" => 6,
            "withVaporCtx" => 7,
            "child" => 10,
            "next" => 11,
            "txt" => 20,
            "toDisplayString" => 21,
            "setText" => 22,
            "createTemplateRefSetter" => 23,
            "setClass" => 30,
            "setProp" => 31,
            "setStyle" => 32,
            "setAttr" => 33,
            "setDOMProp" => 34,
            "setDynamicProps" => 35,
            "applyTextModel" => 36,
            "applyCheckboxModel" => 37,
            "applyRadioModel" => 38,
            "applySelectModel" => 39,
            "applyVShow" => 39,
            "createInvoker" => 40,
            "withModifiers" => 41,
            "withKeys" => 42,
            "on" => 43,
            "delegateEvents" => 44,
            "setDynamicEvents" => 45,
            "setInsertionState" => 78,
            "renderEffect" => 79,
            "createIf" => 80,
            "createFor" => 81,
            "template" => 100,
            _ => 50,
        }
    }

    let mut helpers: Vec<_> = ctx.used_helpers.iter().copied().collect();
    helpers.sort_by_key(|h| helper_priority(h));

    let imports = helpers
        .iter()
        .map(|h| cstr!("{h} as _{h}"))
        .collect::<std::vec::Vec<_>>()
        .join(", ");

    cstr!("import {{ {imports} }} from 'vue';\n")
}

/// Escape template string for JavaScript
pub(crate) fn escape_template(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .into()
}

/// Check if a tag is an SVG element
pub(crate) fn is_svg_tag(tag: &str) -> bool {
    matches!(
        tag,
        "svg"
            | "circle"
            | "ellipse"
            | "line"
            | "path"
            | "polygon"
            | "polyline"
            | "rect"
            | "g"
            | "defs"
            | "symbol"
            | "use"
            | "text"
            | "tspan"
            | "image"
            | "clipPath"
            | "mask"
            | "filter"
            | "linearGradient"
            | "radialGradient"
            | "stop"
            | "foreignObject"
            | "animate"
            | "animateMotion"
            | "animateTransform"
            | "set"
            | "desc"
            | "title"
            | "metadata"
            | "marker"
            | "pattern"
    )
}
