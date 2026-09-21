//! `parse_define_art` agrees with the full script-setup analysis on every
//! statement shape — the ones that register `defineArt()` and the ones that
//! do not.

use super::parse_define_art;
use crate::script_parser::parse_script_setup;

const BATTERY: &[&str] = &[
    // No macro at all, and a file that fails to parse.
    "const a = 1;",
    "const = ;",
    "",
    // Identifier component resolved through a default import.
    "import Button from './Button.vue';\ndefineArt(Button, { title: 'Button' });",
    // Named, namespace and aliased imports.
    "import { Card as Panel } from '@/ui';\ndefineArt(Panel, { category: 'Layout' });",
    "import * as UI from '@/ui';\ndefineArt(UI, {});",
    // Type-only imports are not bindings: the component stays unresolved.
    "import type Button from './Button.vue';\ndefineArt(Button);",
    "import { type Button } from './Button.vue';\ndefineArt(Button);",
    // A later value import overrides an earlier one of the same name.
    "import A from './a.vue';\nimport A from './b.vue';\ndefineArt(A);",
    // An import after the call is not yet recorded at the call.
    "defineArt(Late);\nimport Late from './Late.vue';",
    // String-literal component with every option.
    "defineArt('./base-button.vue', { title: 'Base', description: 'd', category: 'c', \
     tags: ['a', 'b'], status: 'draft', order: 3 });",
    // Registered declarator shapes, with the unwrapping casts.
    "import B from './B.vue';\nconst art = defineArt(B, { title: 'T' });",
    "import B from './B.vue';\nconst art = defineArt(B) as any;",
    "import B from './B.vue';\nlet art = (defineArt(B)!);",
    "import B from './B.vue';\nconst [art] = defineArt(B);",
    // withDefaults processes its inner call.
    "import B from './B.vue';\nwithDefaults(defineArt(B, { title: 'W' }), {});",
    "import B from './B.vue';\nconst w = withDefaults(defineArt(B), {});",
    // The last registered call wins.
    "defineArt('./a.vue', { title: 'first' });\ndefineArt('./b.vue', { title: 'second' });",
    // Shapes the full analysis never registers.
    "import B from './B.vue';\nconst { a } = defineArt(B);",
    "import B from './B.vue';\n(defineArt(B));",
    "import B from './B.vue';\nvoid defineArt(B);",
    "import B from './B.vue';\nif (true) { defineArt(B); }",
    "import B from './B.vue';\nfunction f() { defineArt(B); }",
    "import B from './B.vue';\nexport const art = defineArt(B);",
    "import B from './B.vue';\nfoo.defineArt(B);",
    "defineArt();",
    "defineArt(42);",
    // TypeScript syntax the analysis parses.
    "import B from './B.vue';\ninterface P { a: string }\ndefineArt<P>(B, { order: 1 });",
];

#[test]
fn agrees_with_the_full_analysis_on_every_shape() {
    for source in BATTERY {
        let full = parse_script_setup(source);
        assert_eq!(
            parse_define_art(source).as_ref(),
            full.macros.define_art(),
            "{source}"
        );
    }
}

#[test]
fn registered_shapes_read_exactly() {
    let art = parse_define_art(
        "import { Card as Panel } from '@/ui';\nconst art = defineArt(Panel, { title: 'P', tags: ['x'] }) as any;",
    )
    .expect("registered");
    assert_eq!(
        (
            art.component_name.as_str(),
            art.component_source.as_deref(),
            art.title.as_deref(),
            art.tags.iter().map(|tag| tag.as_str()).collect::<Vec<_>>()
        ),
        ("Panel", Some("@/ui"), Some("P"), vec!["x"])
    );
    assert_eq!(parse_define_art("const { a } = defineArt(B);"), None);
}
