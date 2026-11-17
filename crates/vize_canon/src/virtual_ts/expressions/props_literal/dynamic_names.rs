//! `v-bind:[name]` entries of the checked props literal (#13269).
//!
//! A dynamic argument is no static contract key (#2797), but at runtime it
//! is still a prop. Without it, `<Child :[propName]="value" />` reached the
//! whole-props check as `{}` and reported a required prop the binding
//! supplies. Each such binding is spread as `...{ [name]: value }`, so
//! TypeScript decides as vue-tsc does: a literal-typed name supplies and
//! checks its prop, while a `string` name supplies nothing required. As a
//! spread it raises no `TS1117`/`TS2783` against a static key of the same
//! name.
//!
//! The name and value are already checked as template expressions. This
//! copy is unmapped and `@ts-ignore`d, so only the whole-literal check sees
//! it. A multi-line expression would outrun that one-line directive, so it
//! keeps today's behavior.

use crate::virtual_ts::expressions::prop_sources::generated_prop_span;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::append;
use vize_croquis::croquis::{ComponentUsage, PassedProp};

const DYNAMIC_NAME_IGNORE: &str =
    "// @ts-ignore Dynamic prop names and values own their diagnostics as expressions.";

/// Append one spread entry per `v-bind:[name]="value"` of `usage`.
pub(super) fn append_dynamic_name_entries(
    ts: &mut crate::virtual_ts::TextBuf,
    usage: &ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
    expr_indent: &str,
) {
    for prop in &usage.props {
        if !prop.name_is_dynamic || !prop.is_dynamic || prop.value.is_none() {
            continue;
        }
        let name_prop = PassedProp {
            name: prop.name.clone(),
            name_is_dynamic: false,
            value: Some(prop.name.clone()),
            start: prop.start,
            end: prop.end,
            is_dynamic: true,
        };
        let (Some(name), Some(value)) = (
            generated_prop_span(&name_prop, template_binding_access),
            generated_prop_span(prop, template_binding_access),
        ) else {
            continue;
        };
        let (name, value) = (name.text.trim(), value.text.trim());
        if name.is_empty() || value.is_empty() || name.contains('\n') || value.contains('\n') {
            continue;
        }
        append!(
            *ts,
            "{expr_indent}  {DYNAMIC_NAME_IGNORE}\n{expr_indent}  ...{{ [({name})]: ({value}) }},\n",
        );
    }
}
