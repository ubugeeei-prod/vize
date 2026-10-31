//! Keep the actual constructor result correlated with one template usage.
//!
//! Slot resolution may replace an unusable slots surface with a fallback. Its
//! result must therefore never become the authority for that usage's props.

use super::component_props::is_checkable_prop;
use crate::virtual_ts::helpers::shared_types;
use vize_carton::{String, append, cstr};
use vize_croquis::{Scope, ScopeChain, ScopeData, ScopeId, ScopeKind, analysis::ComponentUsage};

/// The first slice uses project-shared helpers and slot witnesses already
/// emitted in the root render block. The qualified helper names cannot capture
/// an authored setup type alias; standalone helpers retain their existing path.
/// A closure-local witness cannot be named by the root props aliases.
pub(crate) fn root_slot_scope<'a>(
    scopes: &'a ScopeChain,
    usage: &ComponentUsage,
    hoisted: bool,
) -> Option<&'a Scope> {
    if !hoisted
        || usage.vif_guard.is_some()
        || (!usage.props.iter().any(is_checkable_prop) && usage.spread_props.is_empty())
        || !root_ancestry(scopes, Some(usage.scope_id))
    {
        return None;
    }
    scopes.iter().find(|scope| {
        matches!(scope.data(), ScopeData::VSlot(data)
            if data.component.as_deref() == Some(usage.name.as_str())
                && usage.slots.iter().any(|slot| slot.start == scope.span.start))
            && root_ancestry(scopes, scope.parent())
    })
}

fn root_ancestry(scopes: &ScopeChain, mut parent: Option<ScopeId>) -> bool {
    let mut remaining = scopes.len();
    while let Some(id) = parent {
        if remaining == 0 {
            return false;
        }
        remaining -= 1;
        let Some(scope) = scopes.get_scope(id) else {
            return false;
        };
        if matches!(
            scope.kind,
            ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
        ) {
            return false;
        }
        parent = scope.parent();
    }
    true
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
fn usage_raw_props(component: &str, pair: &str, hoisted: bool) -> String {
    let raw = shared_types::COMPONENT_RAW_PROPS.select(hoisted);
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

pub(crate) fn append_usage_props(
    ts: &mut String,
    component: &str,
    pair: Option<&str>,
    hoisted: bool,
) {
    let is_any = shared_types::IS_ANY.select(hoisted);
    if let Some(pair) = pair {
        let props = usage_raw_props(component, pair, hoisted);
        append!(
            *ts,
            "typeof {component} extends {{ __vizeCheck: infer __F }} ? ({is_any}<__F> extends true ? {props} : Record<string, unknown>) : {props};\n"
        );
    } else {
        // Preserve the existing allocation-free writer for unrelated usages.
        let raw = shared_types::COMPONENT_RAW_PROPS.select(hoisted);
        append!(
            *ts,
            "typeof {component} extends {{ __vizeCheck: infer __F }} ? ({is_any}<__F> extends true ? {raw}<typeof {component}> : Record<string, unknown>) : {raw}<typeof {component}>;\n"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{instance_pair_name, root_slot_scope};
    use vize_carton::CompactString;
    use vize_croquis::{
        BindingType, ScopeBinding, ScopeChain, ScopeId, ScopeKind, VSlotScopeData,
        analysis::{ComponentUsage, PassedProp, SlotUsage},
    };

    #[test]
    fn constructor_capture_requires_props_and_a_root_slot_candidate() {
        for closure in [
            None,
            Some(ScopeKind::VFor),
            Some(ScopeKind::VSlot),
            Some(ScopeKind::VMatch),
            Some(ScopeKind::VWhen),
        ] {
            let mut scopes = ScopeChain::new();
            if let Some(kind) = closure {
                scopes.enter_scope(kind);
            }
            let candidate = scopes.enter_v_slot_scope(
                VSlotScopeData {
                    name: "default".into(),
                    props_pattern: Some("{ item }".into()),
                    prop_names: Default::default(),
                    component: Some("Child".into()),
                },
                10,
                30,
            );
            let mut usage = ComponentUsage {
                name: "Child".into(),
                start: 0,
                end: 40,
                props: [PassedProp {
                    name: "value".into(),
                    name_is_dynamic: false,
                    value: Some("1".into()),
                    start: 1,
                    end: 9,
                    is_dynamic: true,
                }]
                .into_iter()
                .collect(),
                events: Default::default(),
                slots: [SlotUsage {
                    name: "default".into(),
                    name_is_dynamic: false,
                    scope_vars: Default::default(),
                    start: 10,
                    end: 30,
                    has_scope: true,
                }]
                .into_iter()
                .collect(),
                has_spread_attrs: false,
                spread_props: Default::default(),
                scope_id: ScopeId::ROOT,
                vif_guard: None,
            };
            assert_eq!(
                root_slot_scope(&scopes, &usage, true).map(|scope| scope.id),
                closure.is_none().then_some(candidate),
                "the candidate slot itself must be outside every closure: {closure:?}",
            );
            assert_eq!(
                root_slot_scope(&scopes, &usage, false).map(|scope| scope.id),
                None
            );
            usage.props.clear();
            assert_eq!(
                root_slot_scope(&scopes, &usage, true).map(|scope| scope.id),
                None
            );
        }
    }

    #[test]
    fn constructor_pair_names_skip_all_authored_binding_collisions() {
        let mut scopes = ScopeChain::new();
        for name in [
            "__vize_slot_host_9_instances",
            "__vize_slot_host_9_instances_1",
        ] {
            scopes.add_binding(
                CompactString::new(name),
                ScopeBinding::new(BindingType::SetupConst, 0),
            );
        }
        assert_eq!(
            instance_pair_name("__vize_slot_host_9", &scopes),
            "__vize_slot_host_9_instances_2",
        );
    }
}
