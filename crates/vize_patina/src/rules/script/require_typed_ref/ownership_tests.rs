use super::RequireTypedRef;
use crate::rules::script::{ScriptLintResult, ScriptLinter};

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequireTypedRef));
    linter.lint(source, 0)
}

#[test]
fn authored_binding_annotations_satisfy_direct_ref_calls() {
    for source in [
        "import { ref } from 'vue'; const current: Ref<string | null> = ref(null);",
        "import { ref } from 'vue'; export const current: Ref<string | null> = ref(null);",
        "import { ref } from 'vue'; let current: Ref<string | undefined> = ref(undefined);",
        "import { ref } from 'vue'; const current: Ref<string | undefined> = (ref());",
        "import { ref as r } from 'vue'; const current: Alias<string | null> = r(null);",
        "import { ref } from 'vue'; const current: import('vue').Ref<string | null> = ref(null);",
        "import { ref } from 'vue'; const current: Vue.Ref<string | null> = ref(null);",
    ] {
        let result = lint(source);
        assert_eq!(
            (result.error_count, result.warning_count),
            (0, 0),
            "{source}"
        );
        assert!(result.diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn annotations_do_not_silence_untyped_nested_or_other_binding_calls() {
    for source in [
        "import { ref } from 'vue'; const current = ref(null);",
        "import { ref } from 'vue'; const current = ref(null) as Ref<string | null>;",
        "import { ref } from 'vue'; const values: Ref<string | null>[] = [ref(null)];",
        "import { ref } from 'vue'; const current: Ref<string | null> = wrap(ref(null));",
        "import { ref } from 'vue'; const current: () => Ref<string | null> = () => ref(null);",
        "import { ref } from 'vue'; const current: Ref<unknown> = ref(ref(null));",
        "import { ref } from 'vue'; const { value }: { value: string | null } = ref(null);",
        "import { ref } from 'vue'; const current: Ref<string | null> = ref(null), other = ref(null);",
        "import { ref } from 'vue'; const current: Ref<string | null> = ref(null); const other = ref(undefined);",
    ] {
        let result = lint(source);
        assert_eq!(
            (result.error_count, result.warning_count),
            (0, 1),
            "{source}"
        );
        assert_eq!(result.diagnostics.len(), 1, "{source}");
        assert_eq!(result.diagnostics[0].rule_name, "script/require-typed-ref");
        let start = result.diagnostics[0].start as usize;
        let end = result.diagnostics[0].end as usize;
        assert!(
            matches!(source.get(start..end), Some("ref(null)" | "ref(undefined)")),
            "{source}"
        );
    }
}
