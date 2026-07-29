//! Resolution of relative module specifiers for `TS2307` false positives.
//!
//! Type-checking a partial file subset can surface a spurious `Cannot find
//! module` diagnostic for a sibling that exists on disk but was not registered
//! in the virtual project. These helpers detect that case so the diagnostic can
//! be suppressed.

use std::path::Path;

use vize_carton::cstr;

/// Extract the specifier from the first `Cannot find module "<spec>"` in
/// `message`, handling straight and smart quotes.
fn module_not_found_specifier(message: &str) -> Option<&str> {
    let rest = message.split_once("Cannot find module ")?.1;
    let open = rest.chars().next()?;
    let close = match open {
        '\'' => '\'',
        '"' => '"',
        '\u{2018}' => '\u{2019}',
        _ => return None,
    };
    let after_open = &rest[open.len_utf8()..];
    let end = after_open.find(close)?;
    Some(&after_open[..end])
}

/// Whether the unresolved module in a `TS2307` `message` is a relative
/// specifier that resolves to a real source file next to `importer`. Such a
/// diagnostic is a false positive from checking a partial file subset, since
/// the sibling exists but was not registered in the virtual project.
pub(crate) fn relative_module_resolves_on_disk(message: &str, importer: &Path) -> bool {
    let Some(specifier) = module_not_found_specifier(message) else {
        return false;
    };
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return false;
    }
    let Some(dir) = importer.parent() else {
        return false;
    };
    relative_specifier_resolves(dir, specifier)
}

/// Source extensions a relative specifier may resolve to when appended to the
/// stem. `vue` is deliberately absent (#3329): the import rewriter redirects an
/// extensionless specifier whose target is a `.vue` file onto that file's mirror
/// module, so a surviving `Cannot find module "./svg"` means the redirect did
/// not fire and the module is genuinely unresolved — excusing it would convert a
/// loud module error into a silent `any` at every usage site. The subset run
/// this suppression exists for still reports the redirected `"./svg.vue.ts"`
/// spelling, which [`relative_specifier_resolves`] keeps suppressing below.
const SOURCE_EXTENSIONS: &[&str] = &["ts", "tsx", "d.ts", "d.mts", "d.cts", "mts", "cts"];

/// Extensions an `index` directory module may use. `vue` stays listed: the
/// rewriter only redirects the direct `<stem>.vue` spelling, so a directory
/// whose entry point is `index.vue` still needs the subset-run suppression.
const INDEX_EXTENSIONS: &[&str] = &["ts", "tsx", "d.ts", "d.mts", "d.cts", "mts", "cts", "vue"];

/// Resolve `specifier` against `dir` the way TypeScript would for a Vue/TS
/// project: an explicit supported extension, an extension appended to the
/// stem, a `.js`->`.ts` rewrite, or an `index` directory module.
fn relative_specifier_resolves(dir: &Path, specifier: &str) -> bool {
    if specifier_has_source_extension(specifier) && dir.join(specifier).is_file() {
        return true;
    }
    if let Some(stem) = specifier
        .strip_suffix(".vue.ts")
        .or_else(|| specifier.strip_suffix(".vue.tsx"))
        && dir.join(cstr!("{stem}.vue").as_str()).is_file()
    {
        return true;
    }
    for (js, ts) in [(".js", ".ts"), (".mjs", ".mts"), (".cjs", ".cts")] {
        if let Some(stem) = specifier.strip_suffix(js)
            && dir.join(cstr!("{stem}{ts}").as_str()).is_file()
        {
            return true;
        }
    }
    for extension in SOURCE_EXTENSIONS {
        if dir
            .join(cstr!("{specifier}.{extension}").as_str())
            .is_file()
        {
            return true;
        }
    }
    let base = dir.join(specifier);
    for extension in INDEX_EXTENSIONS {
        if base.join(cstr!("index.{extension}").as_str()).is_file() {
            return true;
        }
    }
    false
}

fn specifier_has_source_extension(specifier: &str) -> bool {
    specifier.ends_with(".vue")
        || specifier.ends_with(".d.ts")
        || specifier.ends_with(".d.mts")
        || specifier.ends_with(".d.cts")
        || specifier.ends_with(".ts")
        || specifier.ends_with(".tsx")
        || specifier.ends_with(".mts")
        || specifier.ends_with(".cts")
}

#[cfg(test)]
mod tests {
    use super::relative_module_resolves_on_disk;

    use tempfile::TempDir;

    #[test]
    fn ts2307_for_resolvable_relative_siblings_is_suppressed() {
        let dir = TempDir::new().unwrap();
        let importer = dir.path().join("App.vue");
        std::fs::write(&importer, "").unwrap();
        std::fs::write(dir.path().join("types.ts"), "").unwrap();
        std::fs::write(dir.path().join("module-types.d.mts"), "").unwrap();
        std::fs::write(dir.path().join("Panel.vue"), "<template />").unwrap();
        std::fs::create_dir_all(dir.path().join("util")).unwrap();
        std::fs::write(dir.path().join("util").join("index.ts"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("common")).unwrap();
        std::fs::write(dir.path().join("common").join("index.d.cts"), "").unwrap();

        let resolvable_ts = "Cannot find module './types' or its corresponding type declarations.";
        let resolvable_vue_ts =
            "Cannot find module './Panel.vue.ts' or its corresponding type declarations.";
        let resolvable_vue_tsx =
            "Cannot find module './Panel.vue.tsx' or its corresponding type declarations.";
        let resolvable_index =
            "Cannot find module './util' or its corresponding type declarations.";
        let resolvable_dmts =
            "Cannot find module './module-types' or its corresponding type declarations.";
        let resolvable_index_dcts =
            "Cannot find module './common' or its corresponding type declarations.";
        let resolvable_js_to_ts =
            "Cannot find module './types.js' or its corresponding type declarations.";
        let missing_relative =
            "Cannot find module './nope' or its corresponding type declarations.";
        let bare_package = "Cannot find module 'lodash-es' or its corresponding type declarations.";

        assert!(relative_module_resolves_on_disk(resolvable_ts, &importer));
        assert!(relative_module_resolves_on_disk(
            resolvable_vue_ts,
            &importer
        ));
        assert!(relative_module_resolves_on_disk(
            resolvable_vue_tsx,
            &importer
        ));
        assert!(relative_module_resolves_on_disk(
            resolvable_index,
            &importer
        ));
        assert!(relative_module_resolves_on_disk(resolvable_dmts, &importer));
        assert!(relative_module_resolves_on_disk(
            resolvable_index_dcts,
            &importer
        ));
        assert!(relative_module_resolves_on_disk(
            resolvable_js_to_ts,
            &importer
        ));
        assert!(!relative_module_resolves_on_disk(
            missing_relative,
            &importer
        ));
        assert!(!relative_module_resolves_on_disk(bare_package, &importer));
    }

    #[test]
    fn ts2307_for_an_extensionless_sfc_specifier_is_no_longer_suppressed() {
        let dir = TempDir::new().unwrap();
        let importer = dir.path().join("App.vue");
        std::fs::write(&importer, "").unwrap();
        std::fs::write(dir.path().join("Panel.vue"), "<template />").unwrap();
        std::fs::create_dir_all(dir.path().join("widget")).unwrap();
        std::fs::write(dir.path().join("widget").join("index.vue"), "").unwrap();

        // The import rewriter redirects `./Panel` onto `./Panel.vue.ts`, so the
        // extensionless spelling reaching tsgo means the module is genuinely
        // unresolved and must stay loud (#3329)...
        let extensionless = "Cannot find module './Panel' or its corresponding type declarations.";
        assert!(!relative_module_resolves_on_disk(extensionless, &importer));

        // ...while the redirected spelling a partial-subset run reports for a
        // sibling outside the checked set stays suppressed, as does the
        // `index.vue` directory module the rewriter does not redirect.
        let redirected =
            "Cannot find module './Panel.vue.ts' or its corresponding type declarations.";
        let directory_module =
            "Cannot find module './widget' or its corresponding type declarations.";
        assert!(relative_module_resolves_on_disk(redirected, &importer));
        assert!(relative_module_resolves_on_disk(
            directory_module,
            &importer
        ));
    }
}
