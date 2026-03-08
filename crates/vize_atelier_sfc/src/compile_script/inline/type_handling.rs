//! Type resolution for inline script compilation.
//!
//! Delegates to the shared script type resolver so inline compilation and
//! template binding analysis stay in sync.

use vize_carton::{FxHashMap, String};

pub(crate) fn resolve_type_args(
    type_args: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> String {
    crate::script::resolve_type_args(type_args, interfaces, type_aliases)
}

#[cfg(test)]
pub(crate) fn resolve_single_type_ref(
    name: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> Option<String> {
    crate::script::type_resolution::resolve_single_type_ref(name, interfaces, type_aliases)
}

#[cfg(test)]
mod tests {
    use super::{resolve_single_type_ref, resolve_type_args};
    use vize_carton::{FxHashMap, ToCompactString};

    #[test]
    fn resolves_intersection_types() {
        let mut interfaces = FxHashMap::default();
        interfaces.insert(
            "BaseProps".to_compact_string(),
            "{ disabled?: boolean }".to_compact_string(),
        );
        let mut type_aliases = FxHashMap::default();
        type_aliases.insert(
            "ExtendedProps".to_compact_string(),
            "{ size?: 'sm' | 'md' }".to_compact_string(),
        );

        let resolved = resolve_type_args(
            "BaseProps & ExtendedProps & { label: string }",
            &interfaces,
            &type_aliases,
        );

        assert_eq!(
            resolved,
            "{ disabled?: boolean; size?: 'sm' | 'md'; label: string }"
        );
    }

    #[test]
    fn resolves_nested_interface_extends_chain() {
        let mut interfaces = FxHashMap::default();
        interfaces.insert(
            "BaseProps".to_compact_string(),
            "{ disabled?: boolean }".to_compact_string(),
        );
        interfaces.insert(
            "FieldProps".to_compact_string(),
            "BaseProps & { size?: 'sm' | 'md' }".to_compact_string(),
        );

        let resolved = resolve_type_args("FieldProps", &interfaces, &FxHashMap::default());

        assert_eq!(resolved, "{ disabled?: boolean; size?: 'sm' | 'md' }");
    }

    #[test]
    fn resolves_single_type_reference() {
        let mut type_aliases = FxHashMap::default();
        type_aliases.insert(
            "Props".to_compact_string(),
            "{ msg: string }".to_compact_string(),
        );

        let resolved = resolve_single_type_ref("Props", &FxHashMap::default(), &type_aliases);
        assert_eq!(resolved.as_deref(), Some("{ msg: string }"));
    }
}
