//! Vue 2-only public-instance members shared by the template-context emitter
//! and the instance-global emitter.

/// Vue 2-only public-instance members that are absent from Vue 3's
/// `ComponentPublicInstance`.
///
/// In a Vue 2 / 2.7 dialect, template (and `this`) references such as
/// `$listeners`, `$children`, `$scopedSlots`, the `$on`/`$off`/`$once` event
/// emitter, `$set`/`$delete`, and `$createElement`/`_c` are valid but resolve
/// to nothing on the Vue 3 instance type, so Corsa would false-error on them.
/// They are emitted as permissive `any` bindings so v2 templates type-check.
/// Vue 3 output never emits these, so it stays byte-identical.
pub(super) const VUE2_INSTANCE_MEMBERS: &[&str] = &[
    "$listeners",
    "$children",
    "$scopedSlots",
    "$on",
    "$off",
    "$once",
    "$set",
    "$delete",
    "$createElement",
    "_c",
];

/// Whether `generate_template_context` declares this name itself in a Vue 2
/// dialect. The instance-global emitter shares the list so it never emits a
/// second declaration for the same name in the same template closure.
pub(crate) fn is_vue2_instance_member(name: &str) -> bool {
    VUE2_INSTANCE_MEMBERS.contains(&name)
}
