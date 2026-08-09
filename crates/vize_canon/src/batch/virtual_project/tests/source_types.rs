use std::path::Path;

use super::super::build::source_type_for_path;

#[test]
fn source_type_for_path_supports_typescript_and_javascript_families() {
    assert_eq!(
        source_type_for_path(Path::new("foo.ts")),
        Some(oxc_span::SourceType::ts())
    );
    assert_eq!(
        source_type_for_path(Path::new("foo.tsx")),
        Some(oxc_span::SourceType::tsx())
    );
    assert_eq!(
        source_type_for_path(Path::new("foo.js")),
        Some(oxc_span::SourceType::unambiguous())
    );
    assert_eq!(
        source_type_for_path(Path::new("foo.jsx")),
        Some(oxc_span::SourceType::unambiguous().with_jsx(true))
    );
    assert_eq!(
        source_type_for_path(Path::new("foo.mjs")),
        Some(oxc_span::SourceType::mjs())
    );
    assert_eq!(
        source_type_for_path(Path::new("foo.cjs")),
        Some(oxc_span::SourceType::cjs())
    );
    assert_eq!(source_type_for_path(Path::new("foo.vue")), None);
}
