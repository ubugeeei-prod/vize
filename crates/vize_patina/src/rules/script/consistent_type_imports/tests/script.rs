//! Plain TypeScript modules: the script side of typescript-eslint
//! `consistent-type-imports`, case by case from its own test suite.

use super::{ALL_TYPES, nothing, report, reports, ts};

#[test]
fn value_uses_keep_a_value_import() {
    for source in [
        "import Foo from 'foo';\nconst foo: Foo = new Foo();\n",
        "import foo from 'foo';\nconst a: foo.Foo = foo.fn();\n",
        "import { A, B } from 'foo';\nconst foo: A = B();\nconst bar = new A();\n",
        "import Foo from 'foo';\n",
        "import { A, B } from 'foo';\nconst b = B;\n",
        "import { A, B, C as c } from 'foo';\nconst d = c;\n",
        "import {} from 'foo';\n",
        "import 'foo';\n",
        "import { Bar } from './bar';\nexport type { Baz } from './baz';\nexport class Foo extends Bar {}\n",
        "import Default, * as Rest from 'module';\nconst a: typeof Default = Default;\nconst b: typeof Rest = Rest;\n",
        // A shadowing type parameter or local type is not a use of the import.
        "import Foo from 'foo';\ntype T<Foo> = Foo;\n",
        "import Foo from 'foo';\nfunction fn() {\n  type Foo = {};\n  let foo: Foo;\n}\n",
        // `import A = B.C` reads `B` at runtime.
        "import * as ns from 'foo';\nimport Alias = ns.Member;\nexport const a: Alias = 1;\n",
        // `import x = require()` is not an import declaration.
        "import fs = require('fs');\nexport type Stats = fs.Stats;\n",
    ] {
        assert_eq!(ts(source), nothing(), "{source}");
    }
}

#[test]
fn an_export_of_the_binding_keeps_the_written_import_kind() {
    for source in [
        "import Type from 'foo';\nexport { Type };\nexport default Type;\n",
        "import { Type } from 'foo';\nexport { Type };\nexport default Type;\n",
        "import * as Type from 'foo';\nexport { Type };\nexport default Type;\n",
        "import type Type from 'foo';\nexport { Type };\nexport default Type;\nexport type { Type };\n",
        "import type { Type } from 'foo';\nexport { Type };\nexport default Type;\nexport type { Type };\n",
        "import type * as Type from 'foo';\nexport { Type };\nexport default Type;\nexport type { Type };\n",
        "import A from 'foo';\nexport = A;\n",
        "import type A from 'foo';\nexport = A;\n",
        "import type A from 'foo';\nexport = {} as A;\n",
        "import { type A } from 'foo';\nexport = {} as A;\n",
    ] {
        assert_eq!(ts(source), nothing(), "{source}");
    }
    for source in [
        "import Type from 'foo';\nexport type { Type };\n",
        "import { Type } from 'foo';\nexport type { Type };\n",
        "import * as Type from 'foo';\nexport type { Type };\n",
        "import A from 'foo';\nexport = {} as A;\n",
    ] {
        assert_eq!(ts(source), [report(1, ALL_TYPES)], "{source}");
    }
}

#[test]
fn imports_already_written_as_types_are_left_alone() {
    for source in [
        "import { type A } from 'foo';\ntype T = A;\n",
        "import { type A, B } from 'foo';\ntype T = A;\nconst b = B;\n",
        "import { type A, type B } from 'foo';\ntype T = A;\ntype Z = B;\n",
        "import { B } from 'foo';\nimport { type A } from 'foo';\ntype T = A;\nconst b = B;\n",
        "import { B } from 'foo';\nimport type A from 'baz';\ntype T = A;\nconst b = B;\n",
        "import type Type from 'foo';\ntype T = typeof Type;\ntype U = typeof Type.foo;\n",
        "import type * as constants from './constants';\nexport type Y = {\n  [constants.X]: ReadonlyArray<string>;\n};\n",
        // TypeScript reports a type-only import used as a value in a script.
        "import type T from 'mod';\nconst x = T;\n",
        "import type { T } from 'mod';\nconst x = T;\n",
        "import { type T } from 'mod';\nconst x = T;\n",
    ] {
        assert_eq!(ts(source), nothing(), "{source}");
    }
}

#[test]
fn a_declaration_whose_every_import_is_a_type_use_is_reported_whole() {
    for source in [
        "import Foo from 'foo';\nlet foo: Foo;\ntype Bar = Foo;\ninterface Baz {\n  foo: Foo;\n}\nfunction fn(a: Foo): Foo {}\n",
        "import { A, B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
        "import { A as a, B as b } from 'foo';\nlet foo: a;\nlet bar: b;\n",
        "import Foo from 'foo';\ntype Bar = typeof Foo;\n",
        "import foo from 'foo';\ntype Bar = foo.Bar;\n",
        "import foo from 'foo';\ntype Baz = (typeof foo.bar)['Baz'];\n",
        "import * as A from 'foo';\nlet foo: A.Foo;\n",
        "import A, { B } from 'foo';\nlet foo: A;\nlet bar: B;\n",
        "import A, {} from 'foo';\nlet foo: A;\n",
        "import Default, * as Rest from 'module';\nconst a: Default = '';\nconst b: Rest.A = '';\n",
        "import * as constants from './constants';\nexport type Y = {\n  [constants.X]: ReadonlyArray<string>;\n};\n",
        "import { Foo } from 'foo';\nclass A implements Foo {}\ninterface B extends Foo {}\n",
        "import { type A, B } from 'foo';\ntype T = A | B;\n",
    ] {
        assert_eq!(ts(source), [report(1, ALL_TYPES)], "{source}");
    }
}

#[test]
fn a_mixed_declaration_names_the_imports_that_are_only_types() {
    for (source, message) in [
        (
            "import { A, B } from 'foo';\nconst foo: A = B();\n",
            "Import \"A\" is only used as a type.",
        ),
        (
            "import { A, B, C } from 'foo';\nconst foo: A = B();\nlet bar: C;\n",
            "Imports \"A\" and \"C\" are only used as types.",
        ),
        (
            "import A, { B, C, D } from 'foo';\nB();\ntype T = { foo: A; bar: C; baz: D };\n",
            "Imports \"A\", \"C\", and \"D\" are only used as types.",
        ),
        (
            "import Default, * as Rest from 'module';\nconst a: Rest.A = '';\n",
            "Import \"Rest\" is only used as a type.",
        ),
        // An import nothing uses is neither a type nor a value: it stays.
        (
            "import { Foo, Bar } from 'foo';\nfunction test(foo: Foo) {}\n",
            "Import \"Foo\" is only used as a type.",
        ),
        (
            "import { A, B, type C } from 'foo';\ntype T = A | C;\nconst b = B;\n",
            "Import \"A\" is only used as a type.",
        ),
    ] {
        assert_eq!(ts(source), [report(1, message)], "{source}");
    }
}

#[test]
fn every_declaration_is_judged_on_its_own() {
    assert_eq!(
        ts(
            "import type Already1Def from 'foo';\nimport type { Already1 } from 'foo';\nimport A, { B } from 'foo';\nimport { C, D, E } from 'bar';\nimport type { Already2 } from 'bar';\ntype T = { b: B; c: C; d: D };\n"
        ),
        [
            report(3, "Import \"B\" is only used as a type."),
            report(4, "Imports \"C\" and \"D\" are only used as types."),
        ]
    );
}

#[test]
fn an_ambient_module_block_imports_into_its_own_scope() {
    let source = "declare module '*.vue' {\n  import { DefineComponent } from 'vue';\n  import { ref } from 'vue';\n  const component: DefineComponent;\n  export const count: typeof ref;\n  export default component;\n}\n";
    assert_eq!(
        reports(source, "shims.d.ts", None),
        [report(2, ALL_TYPES), report(3, ALL_TYPES)]
    );
    assert_eq!(
        super::fixed(source, "shims.d.ts", None),
        "declare module '*.vue' {\n  import type { DefineComponent } from 'vue';\n  import type { ref } from 'vue';\n  const component: DefineComponent;\n  export const count: typeof ref;\n  export default component;\n}\n"
    );
    // A split keeps the block's indentation.
    let source = "declare module 'm' {\n  import A, { B } from 'foo';\n  export const a: A;\n  export const b: typeof B;\n  export default B;\n}\n";
    assert_eq!(
        super::fixed(source, "shims.d.ts", None),
        "declare module 'm' {\n  import type A from 'foo';\n  import { B } from 'foo';\n  export const a: A;\n  export const b: typeof B;\n  export default B;\n}\n"
    );
    // The block's name is not the module scope's name of the same spelling.
    let source = "import { Shape } from './shape';\nexport const shape = Shape;\ndeclare module './shape' {\n  import { Shape } from './other';\n  export const other: Shape;\n}\n";
    assert_eq!(ts(source), [report(4, ALL_TYPES)]);
    let source = "declare module '*.vue' {\n  import type { DefineComponent } from 'vue';\n  const component: DefineComponent;\n  export default component;\n}\n";
    assert_eq!(
        reports(
            source,
            "shims.d.ts",
            Some(serde_json::json!({ "prefer": "no-type-imports" }))
        ),
        [report(2, "Use an `import` instead of an `import type`.")]
    );
}

#[test]
fn import_attributes_cannot_follow_a_type_only_import() {
    assert_eq!(
        ts(
            "import * as Type from 'foo' with { type: 'json' };\nconst a: typeof Type = 1 as never;\n"
        ),
        nothing()
    );
    assert_eq!(
        ts(
            "import data, { Shape } from './data.json' with { type: 'json' };\nexport const a: Shape = data;\n"
        ),
        [report(1, "Import \"Shape\" is only used as a type.")]
    );
}

#[test]
fn javascript_modules_and_unreadable_semantics_are_left_alone() {
    // Type syntax only exists in TypeScript.
    assert_eq!(
        reports(
            "import { A } from 'foo';\nexport const a = 1;\n",
            "module.js",
            None
        ),
        nothing()
    );
    // A redeclaration leaves the symbol table unable to prove a use absent.
    assert_eq!(
        ts("import { A } from 'foo';\nlet a = 1;\nlet a = 2;\ntype T = A;\n"),
        nothing()
    );
    // Direct `eval` can read any binding of the module.
    assert_eq!(
        ts("import { A } from 'foo';\ntype T = A;\neval('A');\n"),
        nothing()
    );
    assert_eq!(
        ts("import { A } from 'foo';\ntype T = A;\nconst evaluated = 1;\n"),
        [report(1, ALL_TYPES)]
    );
}
