use super::rewrite_default;

macro_rules! assert_snapshot {
    ($value:expr) => {
        insta::with_settings!({ snapshot_path => "../snapshots" }, {
            insta::assert_snapshot!($value);
        });
    };
}

#[test]
fn test_rewrite_default_object() {
    let (result, has_default) = rewrite_default("export default {}", "_sfc_main", false);
    assert!(has_default);
    assert_snapshot!(result.as_str());
}

#[test]
fn test_rewrite_default_with_other_code() {
    let input = r#"
import { ref } from 'vue'

const count = ref(0)

export default {
  name: 'MyComponent'
}
"#;
    let (result, has_default) = rewrite_default(input, "_sfc_main", false);
    assert!(has_default);
    assert_snapshot!(result.as_str());
}

#[test]
fn test_rewrite_default_class() {
    let (result, has_default) = rewrite_default("export default class Foo {}", "_sfc_main", false);
    assert!(has_default);
    assert_snapshot!(result.as_str());
}

#[test]
fn test_rewrite_default_async_generator_function() {
    let (result, has_default) = rewrite_default(
        "export default async function* load() { yield await next() }",
        "_sfc_main",
        false,
    );
    assert!(has_default);
    assert_snapshot!(result.as_str());
}

#[test]
fn test_no_default_export() {
    let (result, has_default) = rewrite_default("export const a = {}", "_sfc_main", false);
    assert!(!has_default);
    assert_snapshot!(result.as_str());
}

#[test]
fn test_named_default_export() {
    let input = "const a = 1\nexport { a as default }";
    let (result, has_default) = rewrite_default(input, "_sfc_main", false);
    assert!(has_default);
    assert_snapshot!(result.as_str());
}
