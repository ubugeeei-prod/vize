//! Exact type-level behavior for the modern and Vue 2 helper declarations.

use super::super::vue_type_helpers;
use std::process::Command;
use vize_carton::{config::VueVersion, cstr};

#[test]
fn kebab_case_handles_long_keys_without_widening_literals() {
    let Some(tsgo) = super::resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::tempdir().unwrap();
    for (legacy, dialect) in [(false, VueVersion::V3), (true, VueVersion::V2)] {
        let declaration = vue_type_helpers(legacy, dialect)
            .lines()
            .find(|line| line.starts_with("type __VizeKebabCase<"))
            .unwrap();
        let long = "longProperty".repeat(40);
        let expected = "long-property".repeat(40);
        let source = cstr!(
            r#"{declaration}
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;
type Empty = Assert<Equal<__VizeKebabCase<"">, "">>;
type Short = Assert<Equal<__VizeKebabCase<"open">, "open">>;
type Digits = Assert<Equal<__VizeKebabCase<"test2_ID">, "test2_-i-d">>;
type Acronym = Assert<Equal<__VizeKebabCase<"HTTPServerURL">, "-h-t-t-p-server-u-r-l">>;
type Unicode = Assert<Equal<__VizeKebabCase<"ÅngströmValue">, "-ångström-value">>;
type Union = Assert<Equal<__VizeKebabCase<"someKey" | "OtherKey">, "some-key" | "-other-key">>;
type Wide = Assert<Equal<__VizeKebabCase<string>, string>>;
type Template = Assert<Equal<__VizeKebabCase<`some${{string}}`>, `some${{string}}`>>;
type Never = Assert<Equal<__VizeKebabCase<never>, never>>;
type Long = Assert<Equal<__VizeKebabCase<"{long}">, "{expected}">>;
"#
        );
        let path = project.path().join(cstr!("kebab-{legacy}.ts").as_str());
        std::fs::write(&path, source).unwrap();
        let output = Command::new(&tsgo)
            .args([
                "--ignoreConfig",
                "--noEmit",
                "--strict",
                "--target",
                "ES2022",
            ])
            .arg(path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "legacy={legacy}: {}\n{}",
            std::str::from_utf8(&output.stdout).unwrap(),
            std::str::from_utf8(&output.stderr).unwrap()
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}
