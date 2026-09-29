use super::super::ReactivityLossQuery;
use vize_croquis::reactivity::{ReactivityLoss, ReactivityLossKind};
use vize_l0::cstr;

pub(in super::super) fn reactivity_loss_diagnostic(loss: &ReactivityLoss) -> ReactivityLossQuery {
    let (message, help) = match &loss.kind {
        ReactivityLossKind::ReactiveDestructure {
            source_name,
            destructured_props,
        } => (
            cstr!(
                "Destructuring reactive value '{}' creates plain snapshots for: {}",
                source_name,
                destructured_props.join(", ")
            ),
            "Use `toRefs(...)`, `toRef(...)`, or access the property through the reactive object.",
        ),
        ReactivityLossKind::RefValueDestructure {
            source_name,
            destructured_props,
        } => (
            cstr!(
                "Destructuring '{}.value' creates plain snapshots for: {}",
                source_name,
                destructured_props.join(", ")
            ),
            "Keep the ref boundary and derive values through `computed(...)` or `toRef(...)`.",
        ),
        ReactivityLossKind::RefValueExtract {
            source_name,
            target_name,
        } => (
            cstr!(
                "Assigning '{}.value' to '{}' stores a plain snapshot",
                source_name,
                target_name
            ),
            "Pass the ref itself, use a getter `() => ref.value`, or wrap the derived value in `computed(...)`.",
        ),
        ReactivityLossKind::ReactivePropertyExtract {
            source_name,
            prop_name,
            target_name,
        } => (
            cstr!(
                "Assigning '{}.{}' to '{}' stores a plain snapshot",
                source_name,
                prop_name,
                target_name
            ),
            "Use `toRef(source, 'key')`, `toRefs(source)`, or access the property on the reactive object.",
        ),
        ReactivityLossKind::PropsDestructure { destructured_props } => (
            cstr!(
                "Destructuring props creates plain snapshots for: {}",
                destructured_props.join(", ")
            ),
            "Use `toRefs(props)`, `toRef(props, 'key')`, or pass a getter `() => prop` across call boundaries.",
        ),
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } => (
            cstr!(
                "Passing '{}' to '{}' cuts the reactive graph from '{}'",
                argument_name,
                callee_name,
                source_name
            ),
            "Pass `Ref<T>` or `ComputedRef<T>` instead, for example `toRef(source, 'key')` or `computed(() => value)`.",
        ),
        ReactivityLossKind::GetterCallExtract {
            context_name,
            getter_name,
            target_name,
            callee_name,
            source_name,
        } => (
            cstr!(
                "Assigning '{}.{}()' to '{}' stores a plain snapshot from '{}' returned by '{}'",
                context_name,
                getter_name,
                target_name,
                source_name,
                callee_name
            ),
            "Keep the getter lazy, wrap it in `computed(...)`, or have the composable return a ref-like value.",
        ),
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } if alias_name == "<mutation>" => (
            cstr!(
                "Mutating '{}' writes through a plain snapshot from '{}'",
                target_name,
                source_name
            ),
            "Mutate the reactive source directly, or keep the value as a ref/computed.",
        ),
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } => (
            cstr!(
                "Assigning plain snapshot '{}' to '{}' keeps reactivity lost from '{}'",
                alias_name,
                target_name,
                source_name
            ),
            "Pass the reactive source itself, a getter, `toRef(...)`, or `computed(...)` instead of aliasing the snapshot.",
        ),
        ReactivityLossKind::ReactiveSpread { source_name } => (
            cstr!("Spreading '{}' creates a non-reactive copy", source_name),
            "Keep the reactive object intact, or copy through refs with `toRefs(...)` when destructuring is intentional.",
        ),
        ReactivityLossKind::ReactiveReassign { source_name } => (
            cstr!(
                "Reassigning reactive binding '{}' breaks tracked identity",
                source_name
            ),
            "Mutate the reactive object in place or store replaceable state in a ref.",
        ),
    };

    ReactivityLossQuery {
        generated_offset: 0,
        source_start: loss.start,
        source_end: loss.end.max(loss.start.saturating_add(1)),
        message,
        help,
    }
}

#[inline]
pub(in super::super) fn diagnostic_key(start: u32, end: u32) -> u64 {
    ((start as u64) << 32) | end as u64
}
