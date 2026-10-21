//! Preserve the complete dependency classification order after sharing a parse.

use oxc_span::SourceType;

use super::ImportRewriter;

#[test]
fn shared_module_list_preserves_vue_probes_and_literal_order() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("Extensionless.vue"), "<template />").unwrap();
    std::fs::write(root.path().join("Script.ts"), "export {};").unwrap();
    let source = r#"import './Child.vue';
export { default } from './Child.vue';
import './Extensionless';
import './Extensionless.vue';
type External = import('../Parent.vue');
import Common = require('./Required.vue');
const lazy = import('./Dynamic.vue');
const common = require('./Required.vue');
declare module './Augment.vue' {}
import './Script';
import './Missing';
import './Generated.vue.ts'; import './Generated.vue.tsx';
import '#alias'; import 'package/Child.vue'; import '/absolute.vue';
// import './Comment.vue';
const text = "import './Text.vue'";
const computed = import('./Computed.vue' + suffix);
"#;
    let rewriter = ImportRewriter::new();
    let modules = rewriter.collect_all_specifiers(source, SourceType::ts());
    assert_eq!(
        modules,
        vec![
            "./Child.vue",
            "./Extensionless",
            "./Extensionless.vue",
            "../Parent.vue",
            "./Required.vue",
            "./Dynamic.vue",
            "./Augment.vue",
            "./Script",
            "./Missing",
            "./Generated.vue.ts",
            "./Generated.vue.tsx",
            "#alias",
            "package/Child.vue",
            "/absolute.vue",
        ]
    );
    let expected = vec![
        "./Child.vue",
        "./Extensionless.vue",
        "../Parent.vue",
        "./Required.vue",
        "./Dynamic.vue",
        "./Augment.vue",
    ];
    assert_eq!(
        rewriter.relative_vue_specifiers(&modules, Some(root.path())),
        expected
    );
    assert_eq!(
        rewriter.collect_relative_vue_specifiers(source, SourceType::ts(), Some(root.path())),
        expected,
    );
    assert_eq!(
        rewriter.relative_vue_specifiers(&modules, None),
        vec![
            "./Child.vue",
            "./Extensionless.vue",
            "../Parent.vue",
            "./Required.vue",
            "./Dynamic.vue",
            "./Augment.vue",
        ]
    );
}

#[test]
fn incomplete_edits_keep_imports_available_to_dependency_rewrites() {
    for incomplete in ["items.value.", "pick("] {
        let source = format!(
            r#"import Button from './MyButton.vue';
import type {{ Item }} from './types';
const items = {{}} as {{ value: Item[] }};
{incomplete}
const selected = null;
"#
        );
        let rewriter = ImportRewriter::new();

        assert_eq!(
            rewriter.collect_all_specifiers(&source, SourceType::ts()),
            vec!["./MyButton.vue", "./types"],
            "{incomplete}",
        );

        let rewritten = rewriter.rewrite(&source, SourceType::ts(), None);
        assert_eq!(
            rewritten.code,
            format!(
                r#"import Button from './MyButton.vue.ts';
import type {{ Item }} from './types';
const items = {{}} as {{ value: Item[] }};
{incomplete}
const selected = null;
"#
            ),
            "{incomplete}",
        );
    }
}

#[test]
fn incomplete_calls_recover_imports_before_a_large_generated_suffix() {
    let suffix = (0..256)
        .map(|index| format!("declare const __generated_{index}: unknown;\n"))
        .collect::<Vec<_>>()
        .join("");
    let source = format!(
        "import Button from './MyButton.vue';\n\
         import type {{ Item }} from './types';\n\
         pick(\n\
         const selected = null;\n\
         {suffix}"
    );
    let rewriter = ImportRewriter::new();

    assert_eq!(
        rewriter.collect_all_specifiers(&source, SourceType::ts()),
        vec!["./MyButton.vue", "./types"],
    );
}
