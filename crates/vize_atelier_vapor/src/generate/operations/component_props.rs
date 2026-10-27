use crate::ir::{CreateComponentIRNode, IRProp, PropValueKind};
use vize_atelier_core::steps::{
    expression::is_typescript_function_expression_node, is_event_handler_reference_node,
};
use vize_carton::{String, ToCompactString, cstr};

use super::{
    super::{context::GenerateContext, escape_js_string_literal},
    events::resolve_inline_handler,
};

/// Generate props object string for a component
pub(super) fn generate_component_props_str(
    ctx: &mut GenerateContext,
    component: &CreateComponentIRNode<'_>,
) -> String {
    for prop in &component.props {
        if let Some(modifiers) = prop.handler_modifiers {
            if !modifiers.non_keys.is_empty() {
                ctx.use_helper("withModifiers");
            }
            if !modifiers.keys.is_empty() {
                ctx.use_helper("withKeys");
            }
        }
    }
    if component
        .props
        .iter()
        .any(|prop| prop.value_kind == PropValueKind::ForwardedSlots)
    {
        return generate_component_props_without_forwarded(ctx, component);
    }

    if component.props.is_empty() {
        return "null".to_compact_string();
    }
    if (component.props.iter()).any(|p| p.value_kind == PropValueKind::DisplayString) {
        ctx.use_helper("toDisplayString");
    }
    let has_dynamic_sources = component
        .props
        .iter()
        .any(|p| !p.key.is_static || p.key.content == "$");
    if has_dynamic_sources {
        if component
            .props
            .iter()
            .any(|p| !p.key.is_static && p.key.is_handler_key)
        {
            ctx.use_helper("toHandlerKey");
        }
        // A `v-on` object source normalizes its keys to `onX` handlers.
        if (component.props.iter()).any(|p| p.key.content == "$" && p.key.is_handler_key) {
            ctx.use_helper("toHandlers");
        }
        return generate_component_spread_props_str(ctx, &component.props);
    }

    let prop_refs = component.props.iter().collect::<std::vec::Vec<_>>();
    let prop_strs = generate_component_prop_entries(ctx, prop_refs.as_slice());
    format_component_props(prop_strs.as_slice())
}

fn generate_component_props_without_forwarded(
    ctx: &mut GenerateContext,
    component: &CreateComponentIRNode<'_>,
) -> String {
    let props = component
        .props
        .iter()
        .filter(|prop| prop.value_kind != PropValueKind::ForwardedSlots)
        .collect::<std::vec::Vec<_>>();
    if props.is_empty() {
        return "null".to_compact_string();
    }
    if props
        .iter()
        .any(|prop| prop.value_kind == PropValueKind::DisplayString)
    {
        ctx.use_helper("toDisplayString");
    }
    let has_dynamic_sources = props
        .iter()
        .any(|p| !p.key.is_static || p.key.content == "$");
    if has_dynamic_sources {
        if props
            .iter()
            .any(|p| !p.key.is_static && p.key.is_handler_key)
        {
            ctx.use_helper("toHandlerKey");
        }
        // A `v-on` object source normalizes its keys to `onX` handlers.
        if props
            .iter()
            .any(|p| p.key.content == "$" && p.key.is_handler_key)
        {
            ctx.use_helper("toHandlers");
        }
        return generate_component_spread_prop_refs_str(ctx, props.as_slice());
    }

    let prop_strs = generate_component_prop_entries(ctx, props.as_slice());
    format_component_props(prop_strs.as_slice())
}

fn format_component_props(prop_strs: &[String]) -> String {
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
    generate_component_spread_props(ctx, props.iter())
}

fn generate_component_spread_prop_refs_str(ctx: &GenerateContext, props: &[&IRProp<'_>]) -> String {
    generate_component_spread_props(ctx, props.iter().copied())
}

fn generate_component_spread_props<'a>(
    ctx: &GenerateContext,
    props: impl IntoIterator<Item = &'a IRProp<'a>>,
) -> String {
    let mut sources: std::vec::Vec<String> = std::vec::Vec::new();
    let mut static_group: std::vec::Vec<&IRProp<'_>> = std::vec::Vec::new();

    for prop in props {
        if !prop.key.is_static {
            push_component_static_prop_group(ctx, &mut sources, &mut static_group);
            let key = ctx.resolve_expression_node(&prop.key);
            let key = match prop.value_kind {
                PropValueKind::ModelUpdate => cstr!("\"update:\" + ({key})"),
                PropValueKind::ModelModifiers => cstr!("({key}) + \"Modifiers\""),
                PropValueKind::Expression
                | PropValueKind::DisplayString
                | PropValueKind::ForwardedSlots => key,
            };
            // Dynamic handler keys retain the event-name expression through
            // lowering; convert after resolving its authored lexical scope.
            let key = if prop.key.is_handler_key {
                cstr!("_toHandlerKey({key})")
            } else {
                key
            };
            let suffix = handler_option_suffix(prop);
            let key = if suffix.is_empty() {
                key
            } else {
                cstr!("{key} + \"{suffix}\"")
            };
            let value = component_prop_expression_value(ctx, prop);
            // Function sources return direct values, unlike static prop
            // groups whose values are getters. Their keys stay reactive.
            sources.push(cstr!("() => ({{ [({})]: {} }})", key, value));
        } else if prop.key.content == "$" {
            push_component_static_prop_group(ctx, &mut sources, &mut static_group);
            if let Some(first) = prop.values.first() {
                let resolved = ctx.resolve_expression_node(first);
                sources.push(if prop.key.is_handler_key {
                    cstr!("() => (_toHandlers({}))", resolved)
                } else {
                    cstr!("() => ({})", resolved)
                });
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
    let first_class_index = props.iter().position(|p| p.key.content == "class");
    let first_style_index = props.iter().position(|p| p.key.content == "style");
    let mut entries = std::vec::Vec::new();

    for (i, prop) in props.iter().enumerate() {
        let key = prop.key.content;
        if crate::ir::is_on(key) {
            if props
                .iter()
                .take(i)
                .any(|earlier| same_component_key(earlier, prop))
            {
                continue;
            }
            let mut duplicates = props
                .iter()
                .skip(i + 1)
                .filter(|other| same_component_key(other, prop));
            if let Some(second) = duplicates.next() {
                let mut handlers = std::vec::Vec::new();
                handlers.push(component_prop_expression_value(ctx, prop));
                handlers.push(component_prop_expression_value(ctx, second));
                handlers
                    .extend(duplicates.map(|other| component_prop_expression_value(ctx, other)));
                entries.push(format_component_prop_entry(
                    prop,
                    cstr!("() => ([{}])", handlers.join(", ")),
                ));
                continue;
            }
        }
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
        .filter(|p| p.key.content == key)
        .map(|p| component_prop_expression_value(ctx, p))
        .collect()
}

fn merge_component_prop_values(values: &[String]) -> String {
    if let [value] = values {
        cstr!("() => ({})", value)
    } else {
        cstr!("() => ([{}])", values.join(", "))
    }
}

fn format_component_prop_entry(prop: &IRProp<'_>, value: String) -> String {
    let suffix = handler_option_suffix(prop);
    if suffix.is_empty() {
        return format_component_prop_named_entry(prop.key.content, prop.key.is_static, value);
    }
    let mut key = String::from(prop.key.content);
    key.push_str(suffix);
    format_component_prop_named_entry(&key, prop.key.is_static, value)
}

fn format_component_prop_named_entry(key: &str, is_static: bool, value: String) -> String {
    if !is_static {
        return cstr!("[{}]: {}", key, value);
    }
    if should_quote_component_prop_key(key) {
        cstr!("\"{}\": {}", escape_js_string_literal(key), value)
    } else {
        cstr!("{}: {}", key, value)
    }
}

fn component_prop_getter_value(ctx: &GenerateContext, prop: &IRProp<'_>) -> String {
    if prop.handler_modifiers.is_none()
        && let Some(first) = prop.values.first()
        && let Some(raw) = first.content.strip_prefix("__RAW__")
    {
        return String::from(raw);
    }
    cstr!("() => ({})", component_prop_expression_value(ctx, prop))
}

fn component_prop_expression_value(ctx: &GenerateContext, prop: &IRProp<'_>) -> String {
    let value = if let Some(first) = prop.values.first() {
        match prop.value_kind {
            PropValueKind::ModelUpdate => {
                return super::super::expression_retained::resolve_model_update_node(ctx, first);
            }
            PropValueKind::ModelModifiers => return first.content.to_compact_string(),
            PropValueKind::DisplayString => {
                return cstr!("_toDisplayString({})", ctx.resolve_expression_node(first));
            }
            PropValueKind::Expression | PropValueKind::ForwardedSlots => {}
        }
        if let Some(raw) = first.content.strip_prefix("__RAW__") {
            cstr!("({})()", raw)
        } else if first.is_static {
            cstr!("\"{}\"", escape_js_string_literal(first.content))
        } else if prop.key.is_handler_key
            && !is_typescript_function_expression_node(first)
            && !is_event_handler_reference_node(first)
        {
            resolve_inline_handler(ctx, first)
        } else {
            ctx.resolve_expression_node(first)
        }
    } else if prop.handler_modifiers.is_some() {
        cstr!("() => {{}}")
    } else {
        cstr!("\"\"")
    };
    let Some(modifiers) = prop.handler_modifiers else {
        return value;
    };
    let mut value = value;
    for (helper, names) in [
        ("withModifiers", &modifiers.non_keys),
        ("withKeys", &modifiers.keys),
    ] {
        if !names.is_empty() {
            let names = names
                .iter()
                .map(|name| cstr!("\"{}\"", escape_js_string_literal(name)))
                .collect::<std::vec::Vec<_>>()
                .join(", ");
            value = cstr!("_{helper}({value}, [{names}])");
        }
    }
    value
}

fn same_component_key(left: &IRProp<'_>, right: &IRProp<'_>) -> bool {
    let (left_suffix, right_suffix) = (handler_option_suffix(left), handler_option_suffix(right));
    if left.key.content == right.key.content {
        return left_suffix == right_suffix;
    }
    if left_suffix.is_empty() && right_suffix.is_empty() {
        return false;
    }
    left.key.content.bytes().chain(left_suffix.bytes()).eq(right
        .key
        .content
        .bytes()
        .chain(right_suffix.bytes()))
}

fn handler_option_suffix(prop: &IRProp<'_>) -> &'static str {
    let Some(modifiers) = prop.handler_modifiers else {
        return "";
    };
    match (
        modifiers.options.once,
        modifiers.options.capture,
        modifiers.options.passive,
    ) {
        (false, false, false) => "",
        (true, false, false) => "Once",
        (false, true, false) => "Capture",
        (false, false, true) => "Passive",
        (true, true, false) => "OnceCapture",
        (true, false, true) => "OncePassive",
        (false, true, true) => "CapturePassive",
        (true, true, true) => "OnceCapturePassive",
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
