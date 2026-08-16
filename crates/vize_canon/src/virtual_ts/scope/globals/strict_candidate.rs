use crate::virtual_ts::helpers::is_reserved_identifier;

pub(super) fn is_strict_template_context_candidate(name: &str) -> bool {
    !is_reserved_identifier(name) && !matches!(name, "undefined" | "NaN" | "Infinity")
}
