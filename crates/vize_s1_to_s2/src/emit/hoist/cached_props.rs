//! Cached static-props object layout.

use vize_s0::String;
use vize_s2::op::Attribute;

use super::{compact_props_object, push_attr_pair, push_spaces, unique_attrs};

/// First-occurrence static attrs for cached vnode calls. The shipped transform
/// keeps one attr inline, but prints multi-key static props over vnode-relative
/// lines inside `_cache` initializers.
pub(super) fn push_object<'a>(
    out: &mut String,
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
    line_indent: usize,
    scope_id: Option<&str>,
) {
    let unique = unique_attrs(attributes);
    let scope = scope_id.filter(|scope| !unique.iter().any(|attr| attr.name == *scope));
    if unique.len() + usize::from(scope.is_some()) <= 1 {
        out.push_str(compact_props_object(unique.iter().copied(), scope_id).as_str());
        return;
    }

    out.push('{');
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('\n');
        push_spaces(out, line_indent + 2);
        push_attr_pair(out, attr);
    }
    if let Some(scope_id) = scope {
        if !unique.is_empty() {
            out.push(',');
        }
        out.push('\n');
        push_spaces(out, line_indent + 2);
        super::push_empty_attr_pair(out, scope_id);
    }
    out.push('\n');
    push_spaces(out, line_indent);
    out.push('}');
}
