//! Keep the actual constructor result correlated with one template usage.
//!
//! Slot resolution may replace an unusable slots surface with a fallback. Its
//! result must therefore never become the authority for that usage's props.

use crate::virtual_ts::helpers::shared_types;
use vize_carton::{String, append, cstr};
use vize_croquis::{Scope, ScopeChain, ScopeData, ScopeKind, analysis::ComponentUsage};

/// The first slice uses only slot witnesses already emitted in the root render
/// block. A closure-local witness cannot be named by the root props aliases.
pub(crate) fn root_slot_scope<'a>(
    scopes: &'a ScopeChain,
    usage: &ComponentUsage,
) -> Option<&'a Scope> {
    if usage.vif_guard.is_some() {
        return None;
    }
    let mut parent = Some(usage.scope_id);
    let mut remaining = scopes.len();
    while let Some(id) = parent {
        if remaining == 0 {
            return None;
        }
        remaining -= 1;
        let scope = scopes.get_scope(id)?;
        if matches!(
            scope.kind,
            ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
        ) {
            return None;
        }
        parent = scope.parent();
    }
    scopes.iter().find(|scope| {
        matches!(scope.data(), ScopeData::VSlot(data)
            if data.component.as_deref() == Some(usage.name.as_str())
                && usage.slots.iter().any(|slot| slot.start == scope.span.start))
    })
}

pub(crate) fn instance_pair_name(binding: &str, scopes: &ScopeChain) -> String {
    let mut name = cstr!("{binding}_instances");
    let mut suffix = 0usize;
    while scopes.iter().any(|scope| scope.has_binding(name.as_str())) {
        suffix += 1;
        name = cstr!("{binding}_instances_{suffix}");
    }
    name
}

/// The values come from the unchanged native `new` calls. Return inference has
/// no component or props constraint which could select another overload.
pub(crate) fn append_capture_open(ts: &mut String, pair: &str, indent: &str) {
    append!(
        *ts,
        "{indent}const {pair} = (undefined as unknown as <__Z, __P>(withoutArguments: __Z, withArguments: __P) => {{ readonly withoutArguments: __Z; readonly withArguments: __P }})(",
    );
}

pub(crate) fn append_slot_selection(
    ts: &mut String,
    pair: &str,
    binding: &str,
    component: &str,
    indent: &str,
    hoisted: bool,
) {
    let selector = shared_types::SLOT_HOST_SELECTOR.select(hoisted);
    let constructor = shared_types::SLOT_CONSTRUCTOR.select(hoisted);
    let instance = shared_types::SLOT_CONSTRUCTOR_INSTANCE.select(hoisted);
    let factory = shared_types::SLOTS_FACTORY.select(hoisted);
    append!(
        *ts,
        ");\n{indent}const {binding} = (undefined as unknown as {selector}<typeof {component}>)((undefined as unknown as {instance}<{constructor}<typeof {component}>, typeof {pair}.withoutArguments, typeof {pair}.withArguments>), (undefined as unknown as {factory}<typeof {component}>)({component})(",
    );
}

/// Generated raw metadata remains authoritative. Only a native instance's
/// public `$props`, never its constructor parameters or slot fallback, supplies
/// the external component contract.
pub(crate) fn usage_raw_props(component: &str, pair: Option<&str>, hoisted: bool) -> String {
    let raw = shared_types::COMPONENT_RAW_PROPS.select(hoisted);
    let Some(pair) = pair else {
        return cstr!("{raw}<typeof {component}>");
    };
    let constructor = shared_types::SLOT_CONSTRUCTOR.select(hoisted);
    let is_any = shared_types::IS_ANY.select(hoisted);
    let lists = if hoisted {
        "import(\"vize:virtual-ts\").$SlotConstructorHasArguments"
    } else {
        "$SlotConstructorHasArguments"
    };
    let selected = cstr!(
        "([{lists}<{constructor}<typeof {component}>>] extends [never] ? typeof {pair}.withoutArguments : typeof {pair}.withArguments)"
    );
    cstr!(
        "typeof {component} extends {{ readonly __vizeComponentMarker: true }} ? {raw}<typeof {component}> : {is_any}<{selected}> extends true ? {raw}<typeof {component}> : {selected} extends {{ $props: infer __P }} ? {raw}<typeof {component}, __P> : {raw}<typeof {component}>"
    )
}
