//! `prefer`, `disallowTypeAnnotations`, and how the rule is switched on.

use serde_json::json;

use super::{ALL_TYPES, RULE, fixed, nothing, report, reports};
use crate::{LintPreset, Linter};

const AVOID: &str = "Use an `import` instead of an `import type`.";
const ANNOTATION: &str = "`import()` type annotations are forbidden.";

fn no_type_imports() -> Option<serde_json::Value> {
    Some(json!({ "prefer": "no-type-imports" }))
}

#[test]
fn no_type_imports_reports_every_type_keyword_whatever_its_uses() {
    for (source, lines) in [
        ("import type Foo from 'foo';\nlet foo: Foo;\n", vec![1]),
        ("import type { Foo } from 'foo';\nlet foo: Foo;\n", vec![1]),
        (
            "import { type A, B, type C } from 'foo';\ntype T = A | C;\nconst b = B;\n",
            vec![1, 1],
        ),
        (
            "import type * as Type from 'foo';\nexport { Type };\nexport type { Type };\n",
            vec![1],
        ),
    ] {
        let expected: Vec<_> = lines.into_iter().map(|line| report(line, AVOID)).collect();
        assert_eq!(
            reports(source, "module.ts", no_type_imports()),
            expected,
            "{source}"
        );
    }
    for source in [
        "import Foo from 'foo';\nlet foo: Foo;\n",
        "import { A } from 'foo';\ntype T = A;\n",
        "import * as Type from 'foo' with { type: 'json' };\nconst a: typeof Type = Type;\n",
    ] {
        assert_eq!(
            reports(source, "module.ts", no_type_imports()),
            nothing(),
            "{source}"
        );
    }
}

#[test]
fn no_type_imports_removes_the_keyword_and_keeps_comments() {
    for (source, expected) in [
        (
            "import type Foo from 'foo';\nlet foo: Foo;\n",
            "import Foo from 'foo';\nlet foo: Foo;\n",
        ),
        (
            "import type { Foo } from 'foo';\nlet foo: Foo;\n",
            "import { Foo } from 'foo';\nlet foo: Foo;\n",
        ),
        (
            "import type /*comment*/ * as AllType from 'foo';\nimport type // comment\nDefType from 'foo';\nimport type /*comment*/ { Type } from 'foo';\n\ntype T = { a: AllType; b: DefType; c: Type };\n",
            "import /*comment*/ * as AllType from 'foo';\nimport // comment\nDefType from 'foo';\nimport /*comment*/ { Type } from 'foo';\n\ntype T = { a: AllType; b: DefType; c: Type };\n",
        ),
        (
            "import { type A, B, type C } from 'foo';\ntype T = A | C;\nconst b = B;\n",
            "import { A, B, C } from 'foo';\ntype T = A | C;\nconst b = B;\n",
        ),
    ] {
        assert_eq!(
            fixed(source, "module.ts", no_type_imports()),
            expected,
            "{source}"
        );
    }
}

#[test]
fn import_type_annotations_are_reported_unless_allowed() {
    let source =
        "let foo: import('foo');\nlet bar: import('foo').Bar;\nconst lazy = () => import('foo');\n";
    assert_eq!(
        reports(source, "module.ts", None),
        [report(1, ANNOTATION), report(2, ANNOTATION)]
    );
    assert_eq!(
        reports(source, "module.ts", no_type_imports()),
        [report(1, ANNOTATION), report(2, ANNOTATION)]
    );
    assert_eq!(
        reports(
            source,
            "module.ts",
            Some(json!({ "disallowTypeAnnotations": false }))
        ),
        nothing()
    );
    assert_eq!(
        reports(
            "<script setup lang=\"ts\">\ndefineProps<{ item: import('./foo').Foo }>();\n</script>\n<template><p /></template>\n",
            "Component.vue",
            None
        ),
        [report(2, ANNOTATION)]
    );
}

#[test]
fn the_rule_is_opt_in_and_belongs_to_no_preset() {
    let source = "<script setup lang=\"ts\">\nimport { Foo } from './foo';\ndefineProps<{ item: Foo }>();\n</script>\n<template><p /></template>\n";
    for preset in LintPreset::ALL {
        let named: Vec<_> = Linter::with_preset(preset)
            .lint_sfc(source, "Component.vue")
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| diagnostic.message.to_string())
            .collect();
        assert_eq!(named, Vec::<String>::new(), "{}", preset.as_str());
    }
    let opted_in: Vec<_> = Linter::with_preset(LintPreset::HappyPath)
        .with_additional_rules(vec![RULE.into()])
        .lint_sfc(source, "Component.vue")
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| diagnostic.message.to_string())
        .collect();
    assert_eq!(opted_in, [ALL_TYPES]);
}

#[test]
fn a_disable_comment_and_a_severity_override_apply() {
    let source = "<script setup lang=\"ts\">\n// eslint-disable-next-line script/consistent-type-imports\nimport { Foo } from './foo';\n// vize-disable-next-line script/consistent-type-imports\nimport { Bar } from './bar';\nimport { Baz } from './baz';\ndefineProps<{ item: Foo | Bar | Baz }>();\n</script>\n<template><p /></template>\n";
    assert_eq!(
        reports(source, "Component.vue", None),
        [report(6, ALL_TYPES)]
    );
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_rule_severity_overrides(vec![(RULE.into(), crate::Severity::Error)])
        .lint_sfc(source, "Component.vue");
    assert_eq!((result.error_count, result.warning_count), (1, 0));
}
