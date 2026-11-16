//! Keys of the SFC's own `components` option, looked up as Vue's registry does.

use vize_croquis::Croquis;

/// Vue tries the authored tag, its camelCase, then its PascalCase spelling
/// (`[tag, camel, pascal]`). A later duplicate key overrides an earlier one.
pub(super) fn registered_local<'a>(summary: &'a Croquis, candidates: [&str; 3]) -> Option<&'a str> {
    let registrations = &summary.component_registrations;
    if registrations.is_empty() {
        return None;
    }
    candidates.into_iter().find_map(|candidate| {
        registrations
            .iter()
            .rev()
            .find(|registration| registration.name.as_str() == candidate)
            .map(|registration| registration.local_name.as_str())
    })
}

/// Whether the `components` option registers the tag, including keys whose
/// value names no static reference (`defineAsyncComponent(...)`).
pub(crate) fn is_registered_component(summary: &Croquis, candidates: [&str; 3]) -> bool {
    registered_local(summary, candidates).is_some()
        || candidates.into_iter().any(|candidate| {
            summary
                .template_component_registrations
                .contains(candidate, false)
        })
}
