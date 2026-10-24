/// Check if an attribute name is a built-in HTML/Vue attribute.
#[inline]
pub(in super::super) fn is_builtin_attr(name: &str) -> bool {
    matches!(
        name,
        "key"
            | "ref"
            | "is"
            | "class"
            | "style"
            | "id"
            | "slot"
            | "slot-scope"
            | "v-slot"
            | "v-if"
            | "v-else"
            | "v-else-if"
            | "v-for"
            | "v-show"
            | "v-bind"
            | "v-on"
            | "v-model"
            | "v-html"
            | "v-text"
            | "v-pre"
            | "v-cloak"
            | "v-once"
            | "v-memo"
    )
}
