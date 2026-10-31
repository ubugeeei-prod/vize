//! Choose a native parsed member without probing another project's first file.

use std::path::Path;

pub(super) fn select<'a>(config: &Path, files: &'a [String]) -> Option<&'a Path> {
    let existing = || {
        files
            .iter()
            .map(Path::new)
            .filter(|path| path.is_absolute() && path.is_file())
    };
    // Native parseConfigFile may put inherited members before the requested
    // project's own members. Prefer an actual local member before projectInfo
    // creates the wrong configured project and forces immediate owner teardown.
    // Canonical paths rank aliases and `..` correctly, while returning the
    // original native member spelling. projectInfo still attests exact config.
    if let Ok(config) = config.canonicalize()
        && let Some(root) = config.parent()
        && let Some(member) = existing().find(|member| {
            member
                .canonicalize()
                .is_ok_and(|member| member.starts_with(root))
        })
    {
        return Some(member);
    }
    // A config can legitimately include only external files. Keep the existing
    // native-member fallback and its subsequent exact projectInfo validation.
    existing().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "vize-startup-member-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }
        fn file(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "export {};\n").unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn strings(paths: &[&Path]) -> Vec<String> {
        paths
            .iter()
            .map(|path| path.to_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn inherited_foreign_member_cannot_outrank_an_existing_requested_project_member() {
        let fixture = Fixture::new();
        let config = fixture.file("overlay/tsconfig.json");
        let foreign = fixture.file("main.ts");
        let local = fixture.file("overlay/main.ts");
        let files = strings(&[&foreign, &local]);
        assert_eq!(select(&config, &files), Some(local.as_path()));
        assert_eq!(fs::read(&foreign).unwrap(), b"export {};\n");
        assert_eq!(fs::read(&local).unwrap(), b"export {};\n");
    }

    #[test]
    fn component_prefixes_dotdot_and_absent_members_do_not_create_local_ownership() {
        let fixture = Fixture::new();
        let config = fixture.file("overlay/tsconfig.json");
        let sibling = fixture.file("overlay-sibling/main.ts");
        let foreign = fixture.file("main.ts");
        let dotdot = fixture.0.join("overlay/../main.ts");
        let local = fixture.file("overlay/main.ts");
        let absent = fixture.0.join("overlay/absent.ts");
        let files = strings(&[
            Path::new("overlay/main.ts"),
            &absent,
            &sibling,
            &dotdot,
            &local,
        ]);
        assert_eq!(select(&config, &files), Some(local.as_path()));
        let fallback = strings(&[&sibling, &foreign]);
        assert_eq!(select(&config, &fallback), Some(sibling.as_path()));
        assert!(select(&config, &strings(&[Path::new("main.ts"), &absent])).is_none());
    }

    #[test]
    fn all_external_members_preserve_native_order_and_original_spelling() {
        let fixture = Fixture::new();
        let config = fixture.file("config/tsconfig.json");
        let first = fixture.file("src/first.ts");
        let second = fixture.file("src/second.ts");
        let alias = fixture.0.join("config/../src/first.ts");
        let files = strings(&[&alias, &second, &first]);
        assert_eq!(select(&config, &files), Some(alias.as_path()));
    }

    #[test]
    #[cfg(unix)]
    fn symlink_to_a_foreign_member_does_not_outrank_real_local_native_member() {
        let fixture = Fixture::new();
        let config = fixture.file("overlay/tsconfig.json");
        let foreign = fixture.file("foreign.ts");
        let alias = fixture.0.join("overlay/alias.ts");
        std::os::unix::fs::symlink(&foreign, &alias).unwrap();
        let local = fixture.file("overlay/main.ts");
        let files = strings(&[&alias, &local]);
        assert_eq!(select(&config, &files), Some(local.as_path()));
        assert_eq!(select(&config, &strings(&[&alias])), Some(alias.as_path()));
    }
}
