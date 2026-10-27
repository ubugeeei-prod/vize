//! Object-wide certainty from the existing Nuxt config parse.
//!
//! A key-local literal after a spread cannot establish render exclusivity.
//! This intentionally closed grammar also rejects runtime setup and overrides.

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};

use super::{ConfigObject, Lookup, optional_path, static_key};

pub(super) fn admitted<'a>(config: &ConfigObject<'a>) -> bool {
    config.root
        && config.closed_program
        && properties(config.object, |key, value| {
            let (bit, valid) = match key {
                "ssr" => (1, matches!(value, Expression::BooleanLiteral(_))),
                "srcDir" => (2, literal_path(value)),
                "pages" => (4, matches!(value, Expression::BooleanLiteral(_))),
                "dir" => (
                    8,
                    nested(value, |key, value| match key {
                        "pages" => literal_path(value).then_some(1),
                        "layouts" => literal_path(value).then_some(2),
                        _ => None,
                    }),
                ),
                "app" => (
                    16,
                    nested(value, |key, value| match key {
                        "pageTransition" => literal_false(value).then_some(1),
                        "layoutTransition" => literal_false(value).then_some(2),
                        _ => None,
                    }),
                ),
                _ => return None,
            };
            valid.then_some(bit)
        })
}

fn properties<'a>(
    object: &'a ObjectExpression<'a>,
    mut property: impl FnMut(&'a str, &'a Expression<'a>) -> Option<u8>,
) -> bool {
    let mut seen = 0_u8;
    for entry in &object.properties {
        let ObjectPropertyKind::ObjectProperty(entry) = entry else {
            return false;
        };
        if entry.computed || entry.method {
            return false;
        }
        let Some(key) = static_key(&entry.key, entry.computed) else {
            return false;
        };
        let Some(bit) = property(key, &entry.value) else {
            return false;
        };
        if seen & bit != 0 {
            return false;
        }
        seen |= bit;
    }
    true
}

fn nested<'a>(
    expression: &'a Expression<'a>,
    property: impl FnMut(&'a str, &'a Expression<'a>) -> Option<u8>,
) -> bool {
    match expression {
        Expression::ObjectExpression(object) => properties(object, property),
        _ => false,
    }
}

fn literal_path<'a>(expression: &'a Expression<'a>) -> bool {
    optional_path(Lookup::Value(expression)).is_some_and(|path| path.is_some())
}

fn literal_false(expression: &Expression<'_>) -> bool {
    matches!(expression, Expression::BooleanLiteral(value) if !value.value)
}

#[cfg(test)]
mod tests {
    use super::admitted;
    use crate::commands::project_nuxt::read;

    fn known(source: &str) -> bool {
        read(source, |config| Some(admitted(config))).unwrap_or(false)
    }

    #[test]
    fn only_closed_original_programs_and_objects_establish_certainty() {
        for source in [
            "export default defineNuxtConfig({})",
            r#"export default defineNuxtConfig({ ssr: false, srcDir: "src" })"#,
            r#"export default { dir: { pages: "routes", layouts: "shells" }, app: { pageTransition: false, layoutTransition: false } }"#,
        ] {
            assert!(known(source), "{source}");
        }
    }

    #[test]
    fn opaque_or_custom_configuration_never_becomes_known_after_literal_overrides() {
        for source in [
            r#"export default defineNuxtConfig({ ...preset, srcDir: "src", pages: true, dir: { pages: "pages", layouts: "layouts" } })"#,
            "export default defineNuxtConfig({ app: { ...defaults, pageTransition: false, layoutTransition: false } })",
            "export default defineNuxtConfig({ app: { pageTransition: { mode: 'in-out' } } })",
            "export default defineNuxtConfig({ hooks: {} })",
            "export default defineNuxtConfig({ modules: [] })",
            "export default defineNuxtConfig({ extends: [] })",
            "export default defineNuxtConfig({ router: {} })",
            "export default defineNuxtConfig({ pages: {} })",
            "export default defineNuxtConfig({ $production: {} })",
            "export default defineNuxtConfig({ $env: {} })",
            "export default defineNuxtConfig({ ssr: false, ssr: false })",
            "export default defineNuxtConfig({ ['ssr']: false })",
            "export default defineNuxtConfig({ get ssr() { return false } })",
            "export default defineNuxtConfig?.({})",
            "export default defineNuxtConfig({}, preset)",
            "const preset = {}; export default defineNuxtConfig({})",
            "installHooks(); export default defineNuxtConfig({})",
            "export default defineNuxtConfig({}); value.",
        ] {
            assert!(!known(source), "{source}");
        }
    }
}
