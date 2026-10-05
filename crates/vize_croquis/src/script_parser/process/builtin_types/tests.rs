use crate::script_parser::parse_script_setup;

fn facts(source: &str) -> Vec<(String, String, String, String)> {
    let parsed = parse_script_setup(source);
    let mut facts: Vec<_> = parsed
        .types
        .builtin_reactive_types()
        .map(|fact| {
            let (start, end) = fact.span();
            (
                fact.name().to_string(),
                format!("{:?}", fact.kind()),
                fact.value_type().to_string(),
                source
                    .get(start as usize..end as usize)
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    facts.sort();
    facts
}

#[test]
fn builtin_aliases_and_numeric_operators_have_complete_source_facts() {
    let source = "import { ref as r, computed as c, shallowRef } from 'vue';\nconst café = r(0);\nconst active = r(false);\nconst label = shallowRef('hello');\nconst doubled = c(() => café.value * 2);\nconst combined = c(() => doubled.value + café.value / 2 - 1 ** 2 % 3);\n";
    assert_eq!(
        facts(source),
        vec![
            (
                "active".into(),
                "Ref".into(),
                "boolean".into(),
                "active".into()
            ),
            ("café".into(), "Ref".into(), "number".into(), "café".into()),
            (
                "combined".into(),
                "Computed".into(),
                "number".into(),
                "combined".into()
            ),
            (
                "doubled".into(),
                "Computed".into(),
                "number".into(),
                "doubled".into()
            ),
            (
                "label".into(),
                "Ref".into(),
                "string".into(),
                "label".into()
            ),
        ]
    );
}

#[test]
fn unknown_or_non_builtin_expressions_have_no_source_type_fact() {
    for source in [
        "import { ref, computed } from './fake'; const n = ref(0); const x = computed(() => n.value * 2);",
        "import type { ref } from 'vue'; const n = ref(0);",
        "import { ref } from 'vue'; let n = ref(0);",
        "import { ref } from 'vue'; const n: unknown = ref(0);",
        "import { ref } from 'vue'; const n = ref<number>(0);",
        "import { ref } from 'vue'; const n = ref(0 as number);",
        "import { ref } from 'vue'; const n = ref(1n);",
        "import { ref } from 'vue'; const n = ref(readCount());",
        "import { computed } from 'vue'; const x = computed(readCount);",
        "import { computed } from 'vue'; const x = computed(() => unknown.value * 2);",
        "import { computed } from 'vue'; const x = computed(() => 1n * 2n);",
        "import { computed } from 'vue'; const x = computed(() => 'a' * 2);",
        "import { computed } from 'vue'; const x = computed(async () => 1 * 2);",
        "import { computed } from 'vue'; const x = computed((n: number) => n * 2);",
        "import { computed } from 'vue'; const x = computed((): number => 1 * 2);",
        "import { computed } from 'vue'; const x = computed(() => { return 1 * 2; });",
        "import { ref } from 'vue'; function f(ref: any) { const n = ref(0); }",
        "import { ref } from 'vue'; function f() { const n = ref(0); }",
        "import { ref } from 'vue'; const ref = fake; const n = ref(0);",
        "import { ref } from 'vue'; const n = ref(0); const broken = ;",
    ] {
        assert_eq!(facts(source), Vec::new(), "whole facts for {source}");
    }
    assert_eq!(
        facts(
            "import { ref, computed } from 'vue'; const n = ref(0); const x = computed(() => n.value.toFixed());"
        ),
        vec![("n".into(), "Ref".into(), "number".into(), "n".into())]
    );
}

#[test]
fn later_binding_ownership_and_native_world_lifecycle_do_not_reuse_stale_facts() {
    assert_eq!(
        facts(
            "import { ref, computed } from 'vue'; const n = ref(0); let n = dynamic(); const x = computed(() => n.value * 2);"
        ),
        Vec::new()
    );
    let source = "import { ref } from 'vue'; const count = ref(0);";
    let parsed = parse_script_setup(source);
    let mut croquis = crate::Croquis::new();
    parsed.apply_to_croquis(&mut croquis);
    let before = croquis
        .types
        .builtin_reactive_types()
        .next()
        .unwrap()
        .span();
    croquis.shift_script_offsets(17);
    let fact = croquis.types.builtin_reactive_types().next().unwrap();
    assert_eq!(fact.span(), (before.0 + 17, before.1 + 17));
    assert_eq!(croquis.binding_spans.get("count"), Some(&fact.span()));
    assert_eq!(
        facts("import { ref } from 'vue'; const count = ref(dynamic());"),
        Vec::new()
    );
    let debug = format!("{:?}", croquis.types);
    assert!(
        !debug.contains("builtin_reactive"),
        "compatibility dump retains its old shape"
    );
}
