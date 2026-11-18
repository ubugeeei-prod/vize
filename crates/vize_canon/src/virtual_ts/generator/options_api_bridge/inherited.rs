//! `mixins:` / `extends:` members incorporated into the Options API `$This`.
//!
//! Without this, `$This` only lists the component's own
//! `props`/`data`/`computed`/`methods`/`inject`/`setup` names, so a
//! `methods`/`computed` body that legitimately calls a member contributed by a
//! mixin reports `TS2339 Property 'x' does not exist on type '$This'` on
//! code `vue-tsc` accepts (#3609).
//!
//! Component constructors contribute their instance. A plain options-object
//! mixin can also contribute its finite, typed `methods` object, including
//! plugin-style dollar names. The method signatures remain intact, so an
//! invalid argument does not acquire an `any` receiver. Unknown, `any`, and
//! open-index method dictionaries contribute no keys: intersecting those into
//! `$This` would silently drop genuine missing-member findings. Finite methods
//! are rebuilt from their keys so a non-strict JavaScript object cannot carry
//! its open property access into the whole receiver (#11780).
//!
//! The neutral element is a named empty interface on purpose. TypeScript drops
//! `{}` / `unknown` while building the intersection and then prints the
//! expanded own-member object in diagnostics. Keeping a named neutral member
//! preserves the `$This` alias and the pinned legacy messages.

pub(super) use crate::virtual_ts::inherited_members::{
    INHERITED_MEMBER_HELPERS, InheritedComponent, collect_inherited_components,
};

#[cfg(test)]
mod tests {
    use super::INHERITED_MEMBER_HELPERS;
    use std::{path::Path, process::Command};

    #[test]
    fn native_mixin_method_laws_preserve_opaque_and_constructor_types() {
        if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
            return;
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let provider = std::env::var_os("CORSA_PATH")
            .map(std::path::PathBuf::from)
            .or_else(|| vize_carton::corsa_resolver::discover_corsa_in_ancestors(root));
        let Some(provider) = provider else {
            assert!(
                std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
                "required native checker unavailable"
            );
            return;
        };
        assert!(provider.is_file(), "native checker must be a regular file");
        let project =
            std::env::temp_dir().join(format!("vize-legacy-method-laws-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&project);
        std::fs::create_dir_all(&project).unwrap();
        let source = format!(
            "{}\n{}\n{}",
            include_str!("../../helpers/declarations/core-types.d.ts.txt"),
            INHERITED_MEMBER_HELPERS,
            include_str!("inherited/guard_laws.ts.txt")
                .replace("__INHERITED_HELPERS__", INHERITED_MEMBER_HELPERS)
        );
        std::fs::write(project.join("laws.ts"), &source).unwrap();
        let output = Command::new(&provider)
            .current_dir(&project)
            .args([
                "--noEmit",
                "--strict",
                "--skipLibCheck",
                "--target",
                "ES2022",
                "laws.ts",
            ])
            .output()
            .expect("native helper law process");
        if let Some(capture) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
            let capture =
                std::path::PathBuf::from(capture).join("legacy-typed-mixin-methods/helper-laws");
            std::fs::create_dir_all(&capture).unwrap();
            std::fs::write(capture.join("laws.ts"), source).unwrap();
            std::fs::write(capture.join("stdout.bin"), &output.stdout).unwrap();
            std::fs::write(capture.join("stderr.bin"), &output.stderr).unwrap();
            std::fs::write(
                capture.join("status.json"),
                serde_json::to_vec_pretty(
                    &serde_json::json!({"status": output.status.code(), "provider": provider}),
                )
                .unwrap(),
            )
            .unwrap();
        }
        let _ = std::fs::remove_dir_all(project);
        assert_eq!(
            output.status.code(),
            Some(0),
            "stdout={:?} stderr={:?}",
            std::str::from_utf8(&output.stdout).unwrap(),
            std::str::from_utf8(&output.stderr).unwrap()
        );
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
}
