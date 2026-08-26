use vize_croquis::croquis::{ComponentUsage, EventListener, PassedProp, SlotUsage};

pub(super) fn prop_items(usage: &ComponentUsage) -> Vec<String> {
    usage
        .props
        .iter()
        .filter(|prop| {
            prop.name_is_dynamic || (prop.name.as_str() != "key" && prop.name.as_str() != "ref")
        })
        .map(format_prop)
        .collect()
}

fn format_prop(prop: &PassedProp) -> String {
    if prop.name_is_dynamic {
        return match prop.value.as_ref() {
            Some(value) => format!("`:[{}]=\"{}\"`", prop.name, value),
            None => format!("`:[{}]`", prop.name),
        };
    }
    match (prop.is_dynamic, prop.value.as_ref()) {
        (true, Some(value)) => format!("`:{}=\"{}\"`", prop.name, value),
        (true, None) => format!("`:{}=\"{}\"`", prop.name, prop.name),
        (false, Some(value)) => format!("`{}=\"{}\"`", prop.name, value),
        (false, None) => format!("`{}`", prop.name),
    }
}

pub(super) fn event_items(usage: &ComponentUsage) -> Vec<String> {
    usage.events.iter().map(format_event).collect()
}

fn format_event(event: &EventListener) -> String {
    let modifiers = event
        .modifiers
        .iter()
        .map(|modifier| format!(".{modifier}"))
        .collect::<String>();
    let name = if event.name_is_dynamic {
        format!("[{}]", event.name)
    } else {
        event.name.to_string()
    };
    match event.handler.as_ref() {
        Some(handler) => format!("`@{name}{modifiers}=\"{handler}\"`"),
        None => format!("`@{name}{modifiers}`"),
    }
}

pub(super) fn slot_items(usage: &ComponentUsage) -> Vec<String> {
    usage.slots.iter().map(format_slot).collect()
}

fn format_slot(slot: &SlotUsage) -> String {
    let name = if slot.name_is_dynamic {
        format!("[{}]", slot.name)
    } else {
        slot.name.to_string()
    };
    if slot.scope_vars.is_empty() {
        return format!("`#{name}`");
    }
    let vars = slot
        .scope_vars
        .iter()
        .map(|name| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!("`#{name} {{ {vars} }}`")
}

#[cfg(test)]
mod tests {
    use super::{format_event, format_prop, format_slot, prop_items};
    use vize_croquis::ScopeId;
    use vize_croquis::croquis::{ComponentUsage, EventListener, PassedProp, SlotUsage};
    use vize_s0::{CompactString, smallvec};

    #[test]
    fn formats_runtime_directive_names_without_presenting_them_as_static() {
        let prop = PassedProp {
            name: CompactString::new("propName"),
            name_is_dynamic: true,
            value: Some(CompactString::new("value")),
            start: 0,
            end: 20,
            is_dynamic: true,
        };
        let event = EventListener {
            name: CompactString::new("eventName"),
            name_is_dynamic: true,
            handler: Some(CompactString::new("handler")),
            modifiers: smallvec![CompactString::new("once")],
            start: 21,
            end: 40,
        };
        let slot = SlotUsage {
            name: CompactString::new("slotName"),
            name_is_dynamic: true,
            scope_vars: smallvec![CompactString::new("row")],
            start: 41,
            end: 60,
            has_scope: true,
        };

        assert_eq!(format_prop(&prop), r#"`:[propName]="value"`"#);
        assert_eq!(format_event(&event), r#"`@[eventName].once="handler"`"#);
        assert_eq!(format_slot(&slot), "`#[slotName] { row }`");
    }

    #[test]
    fn runtime_key_and_ref_names_are_not_filtered_as_reserved_props() {
        let prop = |name, name_is_dynamic| PassedProp {
            name: CompactString::new(name),
            name_is_dynamic,
            value: Some(CompactString::new("value")),
            start: 0,
            end: 10,
            is_dynamic: true,
        };
        let usage = ComponentUsage {
            name: CompactString::new("Child"),
            start: 0,
            end: 40,
            props: smallvec![
                prop("key", false),
                prop("key", true),
                prop("ref", false),
                prop("ref", true),
            ],
            events: smallvec![],
            slots: smallvec![],
            has_spread_attrs: false,
            spread_props: smallvec![],
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        };

        assert_eq!(
            prop_items(&usage),
            ["`:[key]=\"value\"`", "`:[ref]=\"value\"`"]
        );
    }
}
