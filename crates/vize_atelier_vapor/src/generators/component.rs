//! Component code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::CreateComponentIRNode;
use vize_carton::{cstr, String, ToCompactString};

/// Generate CreateComponent code
pub fn generate_create_component(ctx: &mut GenerateContext, component: &CreateComponentIRNode<'_>) {
    let temp = ctx.next_temp();
    let tag = &component.tag;

    // Generate props object
    let props: String = if component.props.is_empty() {
        String::from("{}")
    } else {
        let prop_strs: Vec<String> = component
            .props
            .iter()
            .map(|p| {
                let key = &p.key.content;
                let value: String = if let Some(first) = p.values.first() {
                    if first.is_static {
                        cstr!("\"{}\"", first.content)
                    } else {
                        first.content.to_compact_string()
                    }
                } else {
                    String::from("undefined")
                };
                cstr!("{key}: {value}")
            })
            .collect();
        cstr!("{{ {} }}", prop_strs.join(", "))
    };

    // Generate slots if present
    let slots_code = if component.slots.is_empty() {
        None
    } else {
        Some(generate_slots_object(component))
    };

    if let Some(slots) = slots_code {
        ctx.push_line_fmt(format_args!(
            "const {temp} = _createComponent({tag}, {props}, {slots})"
        ));
    } else {
        ctx.push_line_fmt(format_args!(
            "const {temp} = _createComponent({tag}, {props})"
        ));
    }
}

/// Generate slots object for component
fn generate_slots_object(component: &CreateComponentIRNode<'_>) -> String {
    let slot_strs: Vec<String> = component
        .slots
        .iter()
        .map(|slot| {
            let name: String = if slot.name.is_static {
                slot.name.content.to_compact_string()
            } else {
                cstr!("[{}]", slot.name.content)
            };

            let params = slot
                .fn_exp
                .as_ref()
                .map(|p| p.content.to_compact_string())
                .unwrap_or_default();

            cstr!("{name}: ({params}) => {{ /* slot content */ }}")
        })
        .collect();

    cstr!("{{ {} }}", slot_strs.join(", "))
}

/// Generate component resolution
pub fn generate_resolve_component(name: &str) -> String {
    cstr!("_resolveComponent(\"{name}\")")
}

/// Generate dynamic component
pub fn generate_dynamic_component(
    component_expr: &str,
    props: &str,
    slots: Option<&str>,
) -> String {
    if let Some(slots_code) = slots {
        cstr!("_createComponent({component_expr}, {props}, {slots_code})")
    } else {
        cstr!("_createComponent({component_expr}, {props})")
    }
}

/// Generate async component wrapper
pub fn generate_async_component(component_expr: &str) -> String {
    cstr!("_defineAsyncComponent(() => {component_expr})")
}

/// Generate suspense boundary
pub fn generate_suspense(fallback: Option<&str>) -> (String, String) {
    if let Some(fb) = fallback {
        (
            cstr!("_createSuspense({{ fallback: () => {fb} }})"),
            String::from("})"),
        )
    } else {
        (String::from("_createSuspense({"), String::from("})"))
    }
}

/// Generate keep-alive wrapper
pub fn generate_keep_alive(
    include: Option<&str>,
    exclude: Option<&str>,
    max: Option<usize>,
) -> String {
    let mut options: Vec<vize_carton::CompactString> = Vec::new();

    if let Some(inc) = include {
        options.push(cstr!("include: {inc}"));
    }
    if let Some(exc) = exclude {
        options.push(cstr!("exclude: {exc}"));
    }
    if let Some(m) = max {
        options.push(cstr!("max: {m}"));
    }

    if options.is_empty() {
        String::from("_createKeepAlive({})")
    } else {
        cstr!("_createKeepAlive({{ {} }})", options.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_keep_alive, generate_resolve_component};

    #[test]
    fn test_generate_resolve_component() {
        let result = generate_resolve_component("MyComponent");
        assert_eq!(result, "_resolveComponent(\"MyComponent\")");
    }

    #[test]
    fn test_generate_keep_alive_empty() {
        let result = generate_keep_alive(None, None, None);
        assert_eq!(result, "_createKeepAlive({})");
    }

    #[test]
    fn test_generate_keep_alive_with_options() {
        let result = generate_keep_alive(Some("\"Component1\""), None, Some(10));
        insta::assert_snapshot!(result.as_str());
    }
}
