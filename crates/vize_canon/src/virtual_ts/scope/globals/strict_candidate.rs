use crate::virtual_ts::helpers::is_reserved_identifier;
use vize_croquis::builtins::is_js_global;

pub(super) fn is_strict_template_context_candidate(name: &str) -> bool {
    !is_reserved_identifier(name)
        && !is_js_global(name)
        && !matches!(name, "undefined" | "NaN" | "Infinity")
}
