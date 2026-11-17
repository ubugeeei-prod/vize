//! The `$`-identifiers an SFC authors, bound for generated-name hygiene.

use vize_carton::String;
use vize_croquis::Croquis;

/// Bind the `$`-identifiers the SFC authors, so a generated value name the SFC
/// spells takes a fresh spelling (see [`vize_l0::ts_emit::generated_name::hygiene`]).
pub(super) fn bind_values(summary: &Croquis) -> vize_l0::ts_emit::generated_name::hygiene::Scope {
    use vize_l0::ts_emit::generated_name::hygiene;
    let definitions = summary.types.definitions();
    let scoped = summary
        .scopes
        .iter()
        .flat_map(|scope| scope.bindings().map(|(name, _)| name));
    let script = crate::virtual_ts::script_facts::with_bindings(summary, |bindings| {
        bindings
            .typed()
            .filter(|(name, _)| name.starts_with('$'))
            .map(|(name, _)| String::from(name))
            .collect::<Vec<_>>()
    });
    let types = definitions
        .type_aliases
        .keys()
        .chain(definitions.interfaces.keys())
        .chain(definitions.imported_types.keys())
        .map(|name| name.as_str());
    let template = summary
        .template_expressions
        .iter()
        .flat_map(|expression| hygiene::dollar_identifiers(expression.content.as_str()));
    hygiene::bind(
        scoped
            .chain(script.iter().map(String::as_str))
            .chain(types)
            .chain(template),
    )
}
