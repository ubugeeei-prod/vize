use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};

const RULE: &str = "script/prefer-define-emits-binding";

fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(sfc, "Probe.vue")
}

#[test]
fn reports_and_fixes_only_the_template_callee_with_exact_utf8_bytes() {
    let source = r#"<script setup lang="ts">
const dispatch = defineEmits<{ save: [id: number] }>();
</script>

<template>
  <p>é</p>
  <button @click="$emit('save', 42)">Save</button>
  <button @click="dispatch('save', 42)">Already typed</button>
</template>
"#;
    let start = source.find("$emit").unwrap() as u32;
    let result = lint_sfc(source);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, RULE);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!((diagnostic.start, diagnostic.end), (start, start + 5));
    assert_eq!(
        diagnostic.message,
        "Use the function returned by defineEmits() instead of template `$emit`."
    );
    assert_eq!(
        diagnostic.help.as_deref(),
        Some(
            "Call `dispatch(...)` so event names and payloads are checked against the defineEmits contract."
        )
    );
    let fix = diagnostic.fix.as_ref().unwrap();
    assert_eq!(fix.message, "Replace `$emit` with `dispatch`");
    assert_eq!(fix.edits.len(), 1);
    assert_eq!((fix.edits[0].start, fix.edits[0].end), (start, start + 5));
    assert_eq!(fix.edits[0].new_text, "dispatch");

    let fixed = fix.apply(source);
    assert_eq!(
        fixed,
        source.replacen("$emit('save', 42)", "dispatch('save', 42)", 1)
    );
    assert!(lint_sfc(&fixed).diagnostics.is_empty());
}

#[test]
fn reports_without_a_fix_when_the_macro_result_is_not_a_plain_binding() {
    for declaration in [
        "defineEmits<{ save: [] }>();",
        "const { emit } = defineEmits<{ save: [] }>();",
        "let emit = defineEmits<{ save: [] }>();",
        "var emit = defineEmits<{ save: [] }>();",
        "const first = defineEmits<{ save: [] }>(); const second = defineEmits<{ save: [] }>();",
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\n{declaration}\n</script>\n<template><button @click=\"$emit('save')\" /></template>\n"
        );
        let result = lint_sfc(&source);
        assert_eq!(result.diagnostics.len(), 1, "{declaration}");
        assert!(result.diagnostics[0].fix.is_none(), "{declaration}");
        assert_eq!(
            result.diagnostics[0].help.as_deref(),
            Some(
                "Capture the only defineEmits() call in an immutable plain identifier before replacing `$emit`."
            )
        );
    }
}

#[test]
fn keeps_the_report_but_withholds_a_shadowed_replacement() {
    let source = r#"<script setup>
const dispatch = defineEmits(['save'])
const rows = [1]
</script>
<template>
  <button v-for="dispatch in rows" @click="$emit('save')">Save</button>
</template>
"#;
    let result = lint_sfc(source);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].fix.is_none());
    assert_eq!(
        result.diagnostics[0].help.as_deref(),
        Some(
            "The captured `dispatch` binding cannot be inserted safely in this template scope; rename it before replacing `$emit`."
        )
    );
}

#[test]
fn ignores_non_executable_and_non_component_emit_spellings() {
    let source = r#"<script setup>
const dispatch = defineEmits(['save'])
</script>
<template>
  <!-- $emit('save') -->
  <p title="$emit('save')">$emit('save')</p>
  <button @click="console.log('$emit(\'save\')')">String</button>
  <Child @click="child.$emit('save')" />
  <pre v-pre>{{ $emit('save') }}</pre>
  <button @click="dispatch('save')">Typed</button>
  <button v-for="$emit in handlers" @click="$emit('save')">Shadowed helper</button>
</template>
"#;
    assert!(lint_sfc(source).diagnostics.is_empty());
}

#[test]
fn requires_script_setup_and_a_compiler_owned_macro() {
    let sources = [
        r#"<script>const dispatch = defineEmits(['save'])</script>
<template><button @click="$emit('save')" /></template>"#,
        r#"<script setup>const value = 1</script>
<template><button @click="$emit('save')">{{ value }}</button></template>"#,
        r#"<script setup>
const defineEmits = factory
const dispatch = defineEmits(['save'])
</script>
<template><button @click="$emit('save')" /></template>"#,
        r#"<script setup>
import { defineEmits } from './runtime'
const dispatch = defineEmits(['save'])
</script>
<template><button @click="$emit('save')" /></template>"#,
    ];
    for source in sources {
        assert!(lint_sfc(source).diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn fixes_optional_and_multiple_calls_without_overlapping_edits() {
    let source = r#"<script setup>
const dispatch = defineEmits(['save'])
</script>
<template>
  <button @click="$emit?.(eventName, payload)">Optional</button>
  <button @click="$emit('save'); $emit('save')">Twice</button>
</template>
"#;
    let result = lint_sfc(source);
    assert_eq!(result.diagnostics.len(), 3);
    let mut ranges = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let edit = &diagnostic.fix.as_ref().unwrap().edits[0];
            ((edit.start, edit.end), edit.new_text.as_str())
        })
        .collect::<Vec<_>>();
    ranges.sort_by_key(|((start, _), _)| *start);
    assert!(ranges.windows(2).all(|pair| pair[0].0.1 <= pair[1].0.0));
    assert!(
        ranges
            .iter()
            .all(|(_, replacement)| *replacement == "dispatch")
    );
}

#[test]
fn arrow_parameters_suppress_only_shadowed_reports_and_fixes() {
    let helper_shadowed = r#"<script setup>
const dispatch = defineEmits(['save'])
</script>
<template><button @click="([$emit]) => $emit('save')" /></template>
"#;
    assert!(lint_sfc(helper_shadowed).diagnostics.is_empty());

    let replacement_shadowed = r#"<script setup>
const dispatch = defineEmits(['save'])
</script>
<template><button @click="({ key: dispatch = fallback }, ...rest) => $emit('save')" /></template>
"#;
    let result = lint_sfc(replacement_shadowed);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].fix.is_none());
    let start = replacement_shadowed.find("$emit").unwrap() as u32;
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (start, start + 5)
    );

    let unshadowed = r#"<script setup>
const dispatch = defineEmits(['save'])
</script>
<template><button @click="({ dispatch: local }) => $emit('save', local)" /></template>
"#;
    let result = lint_sfc(unshadowed);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].fix.is_some());

    let nested = r#"<script setup>
const dispatch = defineEmits(['inner', 'outer'])
</script>
<template><button @click="((dispatch) => $emit('inner'))(); $emit('outer')" /></template>
"#;
    let result = lint_sfc(nested);
    assert_eq!(result.diagnostics.len(), 2);
    let mut diagnostics = result.diagnostics.iter().collect::<Vec<_>>();
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    assert!(diagnostics[0].fix.is_none());
    assert_eq!(
        diagnostics[1].fix.as_ref().unwrap().edits[0].new_text,
        "dispatch"
    );
}

#[test]
fn entity_decoding_preserves_the_authored_callee_range_and_fix() {
    let source = r#"<script setup>
const dispatch = defineEmits(['save'])
</script>
<template><button @click="&quot;é&quot;; $emit('save')" /></template>
"#;
    let start = source.find("$emit").unwrap() as u32;
    let result = lint_sfc(source);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!((diagnostic.start, diagnostic.end), (start, start + 5));
    let fix = diagnostic.fix.as_ref().unwrap();
    assert_eq!((fix.edits[0].start, fix.edits[0].end), (start, start + 5));
    let fixed = fix.apply(source);
    assert_eq!(
        fixed,
        source.replacen("$emit('save')", "dispatch('save')", 1)
    );
    assert!(lint_sfc(&fixed).diagnostics.is_empty());
}

#[test]
fn ignores_a_capture_named_dollar_emit_and_withholds_unicode_fixes() {
    let already_typed = r#"<script setup>
const $emit = defineEmits(['save'])
</script>
<template><button @click="$emit('save')" /></template>
"#;
    assert!(lint_sfc(already_typed).diagnostics.is_empty());

    let unicode = r#"<script setup>
const 発火 = defineEmits(['save'])
</script>
<template><button @click="$emit('save')" /></template>
"#;
    let result = lint_sfc(unicode);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].fix.is_none());
}

#[test]
fn slot_shadow_withholds_the_fix_and_two_script_blocks_report_once() {
    let shadowed = r#"<script setup>
const dispatch = defineEmits(['save'])
</script>
<template><Child v-slot="{ dispatch }"><button @click="$emit('save')" /></Child></template>
"#;
    let result = lint_sfc(shadowed);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].fix.is_none());

    let paired = r#"<script>export default { name: 'Probe' }</script>
<script setup>const dispatch = defineEmits(['save'])</script>
<template><button @click="$emit(eventName, payload)" /></template>
"#;
    assert_eq!(lint_sfc(paired).diagnostics.len(), 1);
}
