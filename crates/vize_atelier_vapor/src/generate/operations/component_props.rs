use crate::ir::{CreateComponentIRNode, IRProp};
use vize_carton::{cstr, String, ToCompactString};

use super::{super::context::GenerateContext, events::is_inline_statement};

/// Generate props object string for a component
pub(super) fn generate_component_props_str(
    ctx: &GenerateContext,
    component: &CreateComponentIRNode<'_>,
) -> String {
    if component.props.is_empty() {
        return "null".to_compact_string();
    }
    let has_spreads = component
        .props
        .iter()
        .any(|p| p.key.content.as_str() == "$");
    if has_spreads {
        return generate_component_spread_props_str(ctx, &component.props);
    }

    let prop_refs = component.props.iter().collect::<std::vec::Vec<_>>();
    let prop_strs = generate_component_prop_entries(ctx, prop_refs.as_slice());
    if prop_strs.len() >= 2 {
        let mut result = String::from("{\n");
        for (i, prop_str) in prop_strs.iter().enumerate() {
            result.push_str("    ");
            result.push_str(prop_str);
            if i < prop_strs.len() - 1 {
                result.push(',');
            }
            result.push('\n');
        }
        result.push_str("  }");
        result
    } else {
        ["{ ", &prop_strs.join(", "), " }"].concat().into()
    }
}

fn generate_component_spread_props_str(ctx: &GenerateContext, props: &[IRProp<'_>]) -> String {
    let mut sources: std::vec::Vec<String> = std::vec::Vec::new();
    let mut static_group: std::vec::Vec<&IRProp<'_>> = std::vec::Vec::new();

    for prop in props {
        if prop.key.content.as_str() == "$" {
            push_component_static_prop_group(ctx, &mut sources, &mut static_group);
            if let Some(first) = prop.values.first() {
                let resolved = ctx.resolve_expression(first.content.as_str());
                sources.push(cstr!("() => ({})", resolved));
            }
        } else {
            static_group.push(prop);
        }
    }
    push_component_static_prop_group(ctx, &mut sources, &mut static_group);

    let mut result = String::from("{ $: [");
    if sources.len() >= 2 {
        result.push('\n');
        for (i, source) in sources.iter().enumerate() {
            result.push_str("    ");
            result.push_str(source);
            if i < sources.len() - 1 {
                result.push(',');
            }
            result.push('\n');
        }
        result.push_str("  ] }");
    } else {
        result.push_str(sources.join(", ").as_str());
        result.push_str("] }");
    }
    result
}

fn push_component_static_prop_group(
    ctx: &GenerateContext,
    sources: &mut std::vec::Vec<String>,
    group: &mut std::vec::Vec<&IRProp<'_>>,
) {
    if group.is_empty() {
        return;
    }

    let entries = generate_component_prop_entries(ctx, group.as_slice());
    let source = if entries.len() >= 2 {
        let mut result = String::from("{\n");
        for (i, entry) in entries.iter().enumerate() {
            result.push_str("      ");
            result.push_str(entry);
            if i < entries.len() - 1 {
                result.push(',');
            }
            result.push('\n');
        }
        result.push_str("    }");
        result
    } else {
        cstr!("{{ {} }}", entries.join(", "))
    };
    sources.push(source);
    group.clear();
}

fn generate_component_prop_entries(
    ctx: &GenerateContext,
    props: &[&IRProp<'_>],
) -> std::vec::Vec<String> {
    let class_values = collect_component_prop_values(ctx, props, "class");
    let style_values = collect_component_prop_values(ctx, props, "style");
    let first_class_index = props.iter().position(|p| p.key.content.as_str() == "class");
    let first_style_index = props.iter().position(|p| p.key.content.as_str() == "style");
    let mut entries = std::vec::Vec::new();

    for (i, prop) in props.iter().enumerate() {
        let key = prop.key.content.as_str();
        if key == "class" {
            if Some(i) == first_class_index {
                entries.push(format_component_prop_entry(
                    prop,
                    merge_component_prop_values(class_values.as_slice()),
                ));
            }
            continue;
        }
        if key == "style" {
            if Some(i) == first_style_index {
                entries.push(format_component_prop_entry(
                    prop,
                    merge_component_prop_values(style_values.as_slice()),
                ));
            }
            continue;
        }
        entries.push(format_component_prop_entry(
            prop,
            component_prop_getter_value(ctx, prop),
        ));
    }

    entries
}

fn collect_component_prop_values(
    ctx: &GenerateContext,
    props: &[&IRProp<'_>],
    key: &str,
) -> std::vec::Vec<String> {
    props
        .iter()
        .filter(|p| p.key.content.as_str() == key)
        .map(|p| component_prop_expression_value(ctx, p))
        .collect()
}

fn merge_component_prop_values(values: &[String]) -> String {
    if values.len() == 1 {
        cstr!("() => ({})", values[0])
    } else {
        cstr!("() => ([{}])", values.join(", "))
    }
}

fn format_component_prop_entry(prop: &IRProp<'_>, value: String) -> String {
    let key = prop.key.content.as_str();
    if !prop.key.is_static {
        return cstr!("[{}]: {}", key, value);
    }
    if should_quote_component_prop_key(key) {
        cstr!("\"{}\": {}", key, value)
    } else {
        cstr!("{}: {}", key, value)
    }
}

fn component_prop_getter_value(ctx: &GenerateContext, prop: &IRProp<'_>) -> String {
    let key = prop.key.content.as_str();
    let is_event = key.starts_with("on") && key.len() > 2;
    if let Some(first) = prop.values.first() {
        if first.content.starts_with("__RAW__") {
            return String::from(&first.content.as_str()[7..]);
        }
        if first.is_static {
            return cstr!("() => (\"{}\")", first.content);
        }
        let resolved = ctx.resolve_expression(first.content.as_str());
        if is_event {
            if is_inline_statement(first.content.as_str()) {
                cstr!("() => ($event => ({}))", resolved)
            } else {
                cstr!("() => {}", resolved)
            }
        } else {
            cstr!("() => ({})", resolved)
        }
    } else {
        "undefined".to_compact_string()
    }
}

fn component_prop_expression_value(ctx: &GenerateContext, prop: &IRProp<'_>) -> String {
    if let Some(first) = prop.values.first() {
        if first.content.starts_with("__RAW__") {
            return String::from(&first.content.as_str()[7..]);
        }
        if first.is_static {
            return cstr!("\"{}\"", first.content);
        }
        ctx.resolve_expression(first.content.as_str())
    } else {
        "undefined".to_compact_string()
    }
}

fn should_quote_component_prop_key(key: &str) -> bool {
    if key.contains(':') {
        return true;
    }

    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return true;
    };

    if !first.is_alphabetic() && first != '_' && first != '$' {
        return true;
    }

    chars.any(|ch| !ch.is_alphanumeric() && ch != '_' && ch != '$')
}
