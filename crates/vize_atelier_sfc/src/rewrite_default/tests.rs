#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
use super::rewrite_default;

#[test]
fn test_rewrite_default_object() {
    let (result, has_default) = rewrite_default("export default {}", "_sfc_main", false);
    assert!(has_default);
    insta::assert_snapshot!(result.as_str());
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
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_rewrite_default_class() {
    let (result, has_default) = rewrite_default("export default class Foo {}", "_sfc_main", false);
    assert!(has_default);
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_rewrite_default_async_generator_function() {
    let (result, has_default) = rewrite_default(
        "export default async function* load() { yield await next() }",
        "_sfc_main",
        false,
    );
    assert!(has_default);
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_no_default_export() {
    let (result, has_default) = rewrite_default("export const a = {}", "_sfc_main", false);
    assert!(!has_default);
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_named_default_export() {
    let input = "const a = 1\nexport { a as default }";
    let (result, has_default) = rewrite_default(input, "_sfc_main", false);
    assert!(has_default);
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn default_rewrites_preserve_statement_boundaries() {
    let cases = [
        (
            "export default {}; after()",
            "const _sfc_main = {}; after()",
        ),
        (
            "export default value; after()",
            "const _sfc_main = value; after()",
        ),
        (
            "export default class Named {} after()",
            "class Named {}\nconst _sfc_main = Named; after()",
        ),
        (
            "export default class {} after()",
            "const _sfc_main = class {}; after()",
        ),
        (
            "export default function named() {} after()",
            "function named() {}\nconst _sfc_main = named; after()",
        ),
        (
            "export default function() {} after()",
            "const _sfc_main = function() {}; after()",
        ),
        (
            "const value = {}; export { value as default }; after()",
            "const value = {}; const _sfc_main = value; after()",
        ),
        (
            "export { default } from './other'; after()",
            "import { default as __VUE_DEFAULT__ } from './other'\nconst _sfc_main = __VUE_DEFAULT__; after()",
        ),
        (
            "export default {}\n// next statement\nafter()",
            "const _sfc_main = {}\n// next statement\nafter()",
        ),
        (
            "export default class Named {}\n(after())",
            "class Named {}\nconst _sfc_main = Named;\n(after())",
        ),
        (
            "export default function named() {} [after()]",
            "function named() {}\nconst _sfc_main = named; [after()]",
        ),
        (
            "export default value\n(after())",
            "const _sfc_main = value\n(after())",
        ),
    ];
    let mut failures = Vec::new();
    for (source, expected) in cases {
        let (code, has_default) = rewrite_default(source, "_sfc_main", false);
        assert!(has_default);
        if code.as_str() != expected {
            failures.push(format!("{source}\nexpected: {expected}\nactual: {code}"));
        }
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &code, oxc_span::SourceType::mjs()).parse();
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            failures.push(format!("{source}\nparse errors: {:?}", parsed.diagnostics));
        }
    }
    assert_eq!(failures, Vec::<std::string::String>::new());
}

#[test]
fn default_export_statement_boundary_has_a_pinned_ssr_module() {
    let source = include_str!(
        "../../../../tests/_fixtures/differential/compiler/default-export-statement-boundary-11617/Layout.vue.txt"
    );
    let descriptor = crate::parse_sfc(source, crate::SfcParseOptions::default()).expect("SFC");
    let result = crate::compile_sfc(
        &descriptor,
        crate::SfcCompileOptions {
            parse: crate::SfcParseOptions {
                filename: "Layout.vue".into(),
                ..Default::default()
            },
            template: crate::TemplateCompileOptions {
                ssr: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert_eq!(result.errors.len(), 0);
    assert_eq!(
        result.code.as_str(),
        include_str!(
            "../../../../tests/_fixtures/differential/compiler/default-export-statement-boundary-11617/module.expected.txt"
        )
        .trim_end()
    );
}
