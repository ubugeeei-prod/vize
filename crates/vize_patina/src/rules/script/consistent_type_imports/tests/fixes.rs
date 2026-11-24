//! The rewritten declaration, exactly as `--fix` writes it.

use serde_json::json;

use super::{fixed, lint, reports};

fn separate(source: &str) -> String {
    fixed(source, "module.ts", None)
}

fn inline(source: &str) -> String {
    fixed(
        source,
        "module.ts",
        Some(json!({ "fixStyle": "inline-type-imports" })),
    )
}

#[test]
fn a_declaration_that_is_all_types_gains_the_keyword_in_place() {
    for (source, expected) in [
        (
            "import Foo from 'foo';\nlet foo: Foo;\n",
            "import type Foo from 'foo';\nlet foo: Foo;\n",
        ),
        (
            "import { A, B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
            "import type { A, B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
        ),
        (
            "import { A as a, B as b } from 'foo'\nlet foo: a;\nlet bar: b;\n",
            "import type { A as a, B as b } from 'foo'\nlet foo: a;\nlet bar: b;\n",
        ),
        (
            "import * as A from 'foo';\nlet foo: A.Foo;\n",
            "import type * as A from 'foo';\nlet foo: A.Foo;\n",
        ),
        // Comments and layout inside the declaration are untouched.
        (
            "import {\n  A, // first\n  /* second */ B,\n} from 'foo';\ntype T = A | B;\n",
            "import type {\n  A, // first\n  /* second */ B,\n} from 'foo';\ntype T = A | B;\n",
        ),
    ] {
        assert_eq!(separate(source), expected, "{source}");
    }
}

#[test]
fn a_mixed_declaration_splits_and_the_value_import_keeps_its_place() {
    for (source, expected) in [
        (
            "import { A, B } from 'foo';\nconst foo: A = B();\n",
            "import type { A } from 'foo';\nimport { B } from 'foo';\nconst foo: A = B();\n",
        ),
        (
            "import { A, B, C } from 'foo';\nconst foo: A = B();\nlet bar: C;\n",
            "import type { A, C } from 'foo';\nimport { B } from 'foo';\nconst foo: A = B();\nlet bar: C;\n",
        ),
        (
            "import A, { B, C, D } from 'foo';\nB();\ntype T = { foo: A; bar: C; baz: D };\n",
            "import type A from 'foo';\nimport type { C, D } from 'foo';\nimport { B } from 'foo';\nB();\ntype T = { foo: A; bar: C; baz: D };\n",
        ),
        (
            "import A, { B } from 'foo';\nB();\ntype T = A;\n",
            "import type A from 'foo';\nimport { B } from 'foo';\nB();\ntype T = A;\n",
        ),
        (
            "import A, { B } from 'foo';\nA();\ntype T = B;\n",
            "import type { B } from 'foo';\nimport A from 'foo';\nA();\ntype T = B;\n",
        ),
        (
            "import Default, * as Rest from 'module';\nconst a: Rest.A = Default;\n",
            "import type * as Rest from 'module';\nimport Default from 'module';\nconst a: Rest.A = Default;\n",
        ),
        (
            "import Default, * as Rest from 'module';\nconst a: Default = Rest;\n",
            "import type Default from 'module';\nimport * as Rest from 'module';\nconst a: Default = Rest;\n",
        ),
        // A default and a named or namespace type import cannot share a declaration.
        (
            "import A, { B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
            "import type A from 'foo';\nimport type { B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
        ),
        (
            "import Default, * as Rest from 'module';\nconst a: Default = '';\nconst b: Rest.A = '';\n",
            "import type Default from 'module';\nimport type * as Rest from 'module';\nconst a: Default = '';\nconst b: Rest.A = '';\n",
        ),
        (
            "import A, {} from 'foo';\nlet foo: A;\n",
            "import type A from 'foo';\nlet foo: A;\n",
        ),
        // An import nothing uses, and one already marked `type`, stay put.
        (
            "import { Foo, Bar } from 'foo';\nfunction test(foo: Foo) {}\n",
            "import type { Foo } from 'foo';\nimport { Bar } from 'foo';\nfunction test(foo: Foo) {}\n",
        ),
        (
            "import { A, B, type C } from 'foo';\ntype T = A | C;\nconst b = B;\n",
            "import type { A } from 'foo';\nimport { B, type C } from 'foo';\ntype T = A | C;\nconst b = B;\n",
        ),
        (
            "import { type A, B } from 'foo';\ntype T = A | B;\n",
            "import type { B } from 'foo';\nimport { type A } from 'foo';\ntype T = A | B;\n",
        ),
    ] {
        assert_eq!(separate(source), expected, "{source}");
    }
}

#[test]
fn a_split_keeps_the_layout_of_the_original_list() {
    assert_eq!(
        separate(
            "  import {\n    Config,\n    OnEvent,\n    StorageProviderFactory,\n    URLHelper,\n  } from '../fundamentals'\n\n  type A = Config | OnEvent | URLHelper\n  StorageProviderFactory()\n"
        ),
        "  import type {\n    Config,\n    OnEvent,\n    URLHelper,\n  } from '../fundamentals'\n  import {\n    StorageProviderFactory,\n  } from '../fundamentals'\n\n  type A = Config | OnEvent | URLHelper\n  StorageProviderFactory()\n"
    );
    assert_eq!(
        separate("import {A, B} from \"foo\";\r\nconst foo: A = B();\r\n"),
        "import type {A} from \"foo\";\r\nimport {B} from \"foo\";\r\nconst foo: A = B();\r\n"
    );
}

#[test]
fn inline_style_marks_named_specifiers_where_they_stand() {
    for (source, expected) in [
        (
            "import { A, B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
            "import { type A, type B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
        ),
        (
            "import { A, B } from 'foo';\ntype T = A;\nB();\n",
            "import { type A, B } from 'foo';\ntype T = A;\nB();\n",
        ),
        (
            "import { /* comment */ A, B } from 'foo';\ntype T = A;\nB();\n",
            "import { /* comment */ type A, B } from 'foo';\ntype T = A;\nB();\n",
        ),
        (
            "import A, { B, C } from 'foo';\ntype T = B;\ntype U = C;\nA();\n",
            "import A, { type B, type C } from 'foo';\ntype T = B;\ntype U = C;\nA();\n",
        ),
        // A default or namespace import has no inline form.
        (
            "import Foo from 'foo';\nlet foo: Foo;\n",
            "import type Foo from 'foo';\nlet foo: Foo;\n",
        ),
        (
            "import A, { B, C as D } from 'foo';\ntype T = B;\ntype U = D;\ntype V = A;\n",
            "import type A from 'foo';\nimport { type B, type C as D } from 'foo';\ntype T = B;\ntype U = D;\ntype V = A;\n",
        ),
        // Import attributes stay with the value declaration.
        (
            "import data, { Shape } from './data.json' with { type: 'json' };\nexport const a: Shape = data;\n",
            "import data, { type Shape } from './data.json' with { type: 'json' };\nexport const a: Shape = data;\n",
        ),
    ] {
        assert_eq!(inline(source), expected, "{source}");
    }
}

#[test]
fn a_split_that_would_lose_a_comment_or_an_attribute_is_reported_without_a_fix() {
    for source in [
        "import A, { /* comment */ B } from 'foo';\nA();\ntype T = B;\n",
        "import data, { Shape } from './data.json' with { type: 'json' };\nexport const a: Shape = data;\n",
    ] {
        let result = lint(source, "module.ts", None);
        let fixes: Vec<bool> = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.fix.is_some())
            .collect();
        assert_eq!(fixes, [false], "{source}");
        assert_eq!(separate(source), source);
    }
}

#[test]
fn every_fix_is_one_replacement_of_its_declaration() {
    // The applier drops overlapping edits one by one, so a fix made of
    // several edits could be half applied.
    let source = "import A, { B, C } from 'foo';\nimport { D, E } from 'bar';\nB();\nE();\ntype T = A | C | D;\n";
    let result = lint(source, "module.ts", None);
    let edits: Vec<(u32, u32)> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let fix = diagnostic.fix.as_ref().unwrap();
            assert_eq!(fix.edits.len(), 1);
            (fix.edits[0].start, fix.edits[0].end)
        })
        .collect();
    assert_eq!(edits, [(0, 30), (31, 58)]);
    assert_eq!(
        separate(source),
        "import type A from 'foo';\nimport type { C } from 'foo';\nimport { B } from 'foo';\nimport type { D } from 'bar';\nimport { E } from 'bar';\nB();\nE();\ntype T = A | C | D;\n"
    );
}

#[test]
fn the_fix_edits_the_script_block_of_an_sfc() {
    let source = "<script setup lang=\"ts\">\n  import Card, { type Size, Shape, Kind } from './Card.vue';\n  const a: Shape = Kind.a;\n</script>\n<template><Card /></template>\n";
    assert_eq!(
        fixed(source, "Component.vue", None),
        "<script setup lang=\"ts\">\n  import type { Shape } from './Card.vue';\n  import Card, { type Size, Kind } from './Card.vue';\n  const a: Shape = Kind.a;\n</script>\n<template><Card /></template>\n"
    );
    // The pug view shifts template offsets, never the script's.
    let pug = "<template lang=\"pug\">\ndiv\n  Card\n</template>\n<script setup lang=\"ts\">\nimport Card, { Shape } from './Card.vue';\ndefineProps<{ shape: Shape }>();\n</script>\n";
    assert_eq!(
        fixed(pug, "Component.vue", None),
        "<template lang=\"pug\">\ndiv\n  Card\n</template>\n<script setup lang=\"ts\">\nimport type { Shape } from './Card.vue';\nimport Card from './Card.vue';\ndefineProps<{ shape: Shape }>();\n</script>\n"
    );
}

#[test]
fn a_type_import_the_template_evaluates_becomes_a_value_import() {
    for (options, expected) in [
        (
            None,
            "<script setup lang=\"ts\">\nimport type { Kind } from './shape';\nimport { Shape } from './shape';\nimport { Size } from './size';\nimport Default from './default';\ndefineProps<{ kind: Kind }>();\n</script>\n<template><p :title=\"Shape.name\">{{ Size }}</p><component :is=\"Default\" /></template>\n",
        ),
        (
            Some(json!({ "fixStyle": "inline-type-imports" })),
            "<script setup lang=\"ts\">\nimport { Shape, type Kind } from './shape';\nimport { Size } from './size';\nimport Default from './default';\ndefineProps<{ kind: Kind }>();\n</script>\n<template><p :title=\"Shape.name\">{{ Size }}</p><component :is=\"Default\" /></template>\n",
        ),
    ] {
        let source = "<script setup lang=\"ts\">\nimport type { Shape, Kind } from './shape';\nimport { type Size } from './size';\nimport type Default from './default';\ndefineProps<{ kind: Kind }>();\n</script>\n<template><p :title=\"Shape.name\">{{ Size }}</p><component :is=\"Default\" /></template>\n";
        let after = fixed(source, "Component.vue", options.clone());
        assert_eq!(after, expected);
        assert_eq!(reports(&after, "Component.vue", options), super::nothing());
    }
}
