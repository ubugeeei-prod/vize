//! Nuxt auto-import names for `vue/no-undefined-refs`, as `vize lint` applies them.

use tower_lsp::lsp_types::Url;
use vize_l0::String;
use vize_patina::Linter;

use crate::server::{NuxtTemplateNames, ServerState};

const NO_UNDEFINED_REFS: &str = "vue/no-undefined-refs";

/// Teach `linter` the names the document's Nuxt project auto-imports into its
/// templates, next to the `configured` ones.
///
/// Until `nuxi prepare` has run nothing names them, and every auto-import
/// would read as undefined. The rule is turned off for that project instead;
/// `disabled` is the set the linter was built with.
pub(super) fn apply(
    state: &ServerState,
    uri: &Url,
    linter: Linter,
    configured: Option<&[String]>,
    mut disabled: Vec<String>,
) -> Linter {
    // A linter that never runs the rule needs no generated file read.
    if !linter.is_rule_enabled(NO_UNDEFINED_REFS) || !linter.registry().has_rule(NO_UNDEFINED_REFS)
    {
        return linter;
    }
    let Ok(file) = uri.to_file_path() else {
        return linter;
    };
    let Some(names) = file
        .parent()
        .and_then(|directory| state.nuxt_template_names(directory))
    else {
        return linter;
    };
    match names {
        NuxtTemplateNames::Generated(names) => linter.with_undefined_ref_globals(
            configured
                .unwrap_or_default()
                .iter()
                .chain(names.iter())
                .cloned()
                .collect(),
        ),
        NuxtTemplateNames::NotGenerated => {
            disabled.push(NO_UNDEFINED_REFS.into());
            linter.with_disabled_rules(disabled)
        }
    }
}
