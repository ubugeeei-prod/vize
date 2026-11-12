//! Whether a program declares the global `useTemplateRef` as Vue's own export.
//!
//! Nuxt auto-imports Vue's composable and declares it as
//! `const useTemplateRef: typeof import('…/node_modules/vue').useTemplateRef`.
//! Vue's language tools recognize such a call by name; Vize does so only for
//! this spelling, because a separately authored ambient composable with a
//! Vue-like signature keeps its own contract (#10980).

/// Whether `declaration` (an ambient `.d.ts`) declares the global
/// `useTemplateRef` as the `useTemplateRef` export of the `vue` package.
pub fn declares_vue_template_ref_global(declaration: &str) -> bool {
    let mut rest = declaration;
    while let Some(index) = rest.find("useTemplateRef") {
        let before = rest.get(..index).unwrap_or_default();
        let after = rest
            .get(index + "useTemplateRef".len()..)
            .unwrap_or_default();
        if before.trim_end().ends_with("const") && vue_export_type(after) {
            return true;
        }
        rest = after;
    }
    false
}

/// `: typeof import('<vue>').useTemplateRef` or `['useTemplateRef']`.
fn vue_export_type(after_name: &str) -> bool {
    let Some(rest) = after_name.trim_start().strip_prefix(':') else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix("typeof") else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix("import(") else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(quote) = rest
        .chars()
        .next()
        .filter(|quote| matches!(quote, '\'' | '"'))
    else {
        return false;
    };
    let Some((specifier, rest)) = rest.get(1..).and_then(|rest| rest.split_once(quote)) else {
        return false;
    };
    if !is_vue_package(specifier) {
        return false;
    }
    let Some(member) = rest.trim_start().strip_prefix(')') else {
        return false;
    };
    let member = member.trim_start();
    member
        .strip_prefix(".useTemplateRef")
        .is_some_and(ends_identifier)
        || ["['useTemplateRef']", "[\"useTemplateRef\"]"]
            .iter()
            .any(|spelling| member.starts_with(spelling))
}

/// `vue`, or a resolved path whose last segment is the `vue` package.
fn is_vue_package(specifier: &str) -> bool {
    let specifier = specifier.trim_end_matches('/');
    specifier == "vue" || specifier.ends_with("/node_modules/vue")
}

fn ends_identifier(rest: &str) -> bool {
    rest.chars()
        .next()
        .is_none_or(|next| !(next.is_alphanumeric() || next == '_' || next == '$'))
}

#[cfg(test)]
mod tests {
    use super::declares_vue_template_ref_global;

    #[test]
    fn vue_reexports_are_recognized() {
        for declaration in [
            "declare global {\n  const useTemplateRef: typeof import('vue')['useTemplateRef']\n}",
            "declare global {\n  const useTemplateRef: typeof import('../../node_modules/.pnpm/vue@3.5.41/node_modules/vue').useTemplateRef\n}",
            "declare global { const useTemplateRef: typeof import(\"vue\")[\"useTemplateRef\"] }",
        ] {
            assert!(
                declares_vue_template_ref_global(declaration),
                "{declaration}"
            );
        }
    }

    #[test]
    fn separately_authored_composables_are_not() {
        for declaration in [
            "declare global {\n  const useTemplateRef: <T = unknown, K extends string = string>(key: K) => Readonly<import('vue').ShallowRef<T | null>>;\n}",
            "declare global { const useTemplateRef: typeof import('my-vue')['useTemplateRef'] }",
            "declare global { const useTemplateRefsList: typeof import('@vueuse/core').useTemplateRefsList }",
            "declare global { const useTemplateRef: typeof import('vue').useTemplateRefOther }",
        ] {
            assert!(
                !declares_vue_template_ref_global(declaration),
                "{declaration}"
            );
        }
    }
}
