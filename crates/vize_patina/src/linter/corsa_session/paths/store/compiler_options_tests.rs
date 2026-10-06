//! Authored effective options keep their native indexed-access semantics.
use super::session_tsconfig_contents;
use serde_json::{Value, json};
use std::{fs, path::PathBuf, time::SystemTime};

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("vize-lint-options-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn native_session_retains_whole_config_for_authored_index_access_options() {
    let cases: &[(&str, &[(&str, &str)], Option<bool>)] = &[
        (
            "absent",
            &[(
                "tsconfig.json",
                r#"{"compilerOptions":{},"include":["src/**/*.vue"]}"#,
            )],
            None,
        ),
        (
            "true",
            &[(
                "tsconfig.json",
                r#"{"compilerOptions":{"noUncheckedIndexedAccess":true}}"#,
            )],
            Some(true),
        ),
        (
            "false",
            &[(
                "tsconfig.json",
                r#"{"compilerOptions":{"noUncheckedIndexedAccess":false}}"#,
            )],
            Some(false),
        ),
        (
            "extended",
            &[
                (
                    "base.json",
                    r#"{"compilerOptions":{"noUncheckedIndexedAccess":true}}"#,
                ),
                ("tsconfig.json", r#"{"extends":"./base.json"}"#),
            ],
            Some(true),
        ),
        (
            "override",
            &[
                (
                    "base.json",
                    r#"{"compilerOptions":{"noUncheckedIndexedAccess":true}}"#,
                ),
                (
                    "tsconfig.json",
                    r#"{"extends":"./base.json","compilerOptions":{"noUncheckedIndexedAccess":false}}"#,
                ),
            ],
            Some(false),
        ),
        (
            "array",
            &[
                (
                    "true.json",
                    r#"{"compilerOptions":{"noUncheckedIndexedAccess":true}}"#,
                ),
                (
                    "false.json",
                    r#"{"compilerOptions":{"noUncheckedIndexedAccess":false}}"#,
                ),
                (
                    "tsconfig.json",
                    r#"{"extends":["./true.json","./false.json"]}"#,
                ),
            ],
            Some(false),
        ),
        (
            "nearest",
            &[
                (
                    "tsconfig.json",
                    r#"{"compilerOptions":{"noUncheckedIndexedAccess":false}}"#,
                ),
                (
                    "src/tsconfig.json",
                    r#"{"compilerOptions":{"noUncheckedIndexedAccess":true}}"#,
                ),
            ],
            Some(true),
        ),
    ];
    for &(name, files, flag) in cases {
        let project = Project::new();
        project.write("src/MyTouch.vue", "<template><p>test</p></template>\n");
        for &(path, contents) in files {
            project.write(path, contents);
        }
        // The complete historical stub is independently fixed here. Only the
        // one effective authored option may add a field to this whole object.
        let mut expected = json!({
            "compilerOptions": {
                "target":"ES2022", "module":"ESNext", "moduleResolution":"bundler",
                "allowImportingTsExtensions":true, "lib":["ES2022","DOM","DOM.Iterable"],
                "rootDirs":[".","../../.."], "strict":true, "noEmit":true, "skipLibCheck":true
            },
            "include":["**/*.patina.ts"]
        });
        if let Some(flag) = flag {
            expected["compilerOptions"]["noUncheckedIndexedAccess"] = json!(flag);
        }
        let rendered = session_tsconfig_contents(
            &project.0,
            &project.0.join("src/MyTouch.vue").to_string_lossy(),
        );
        assert_eq!(
            serde_json::from_str::<Value>(&rendered).unwrap(),
            expected,
            "{name}"
        );
        assert_eq!(
            rendered.as_str(),
            format!("{}\n", serde_json::to_string_pretty(&expected).unwrap()),
            "{name}"
        );
        for &(path, contents) in files {
            assert_eq!(
                fs::read_to_string(project.0.join(path)).unwrap(),
                contents,
                "{name}"
            );
        }
    }
}
