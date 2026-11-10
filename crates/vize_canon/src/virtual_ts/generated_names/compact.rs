//! Selects a fresh namespace for compact template-scope contract names.

use super::GenericParameters;
use vize_carton::{String, cstr};
use vize_croquis::Croquis;

const ROLES: &[&str] = &[
    "Comp",
    "Props",
    "ValueProps",
    "CheckProps",
    "CheckTail",
    "Check",
    "ResolvedProps",
    "SelectedProps",
];

fn authored(parameters: Option<&GenericParameters>, summary: &Croquis, name: &str) -> bool {
    parameters.is_some_and(|parameters| parameters.parameter_names().any(|bound| bound == name))
        || crate::virtual_ts::helpers::hygiene::authored_binding(summary, name)
}

pub(super) fn template_prefix(parameters: Option<&GenericParameters>, summary: &Croquis) -> String {
    let available = |prefix: &str| {
        summary
            .component_usages
            .iter()
            .enumerate()
            .all(|(idx, usage)| {
                ROLES.iter().all(|role| {
                    !authored(parameters, summary, cstr!("{prefix}{role}{idx}").as_str())
                }) && usage.props.iter().all(|prop| {
                    let prop =
                        crate::virtual_ts::helpers::to_safe_identifier_fragment(prop.name.as_str());
                    ["Prop", "Model"].into_iter().all(|role| {
                        !authored(
                            parameters,
                            summary,
                            cstr!("{prefix}{role}{idx}${prop}").as_str(),
                        )
                    })
                })
            })
    };
    if available("__V") {
        return String::const_new("__V");
    }
    let mut suffix = 1usize;
    loop {
        let prefix = cstr!("__V${suffix}");
        if available(&prefix) {
            return prefix;
        }
        suffix += 1;
    }
}
