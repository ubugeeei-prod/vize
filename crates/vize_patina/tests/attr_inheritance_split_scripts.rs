//! `script/no-duplicate-attr-inheritance` in SFCs with both `<script>` and
//! `<script setup>`: `inheritAttrs` may be declared in either block.

use vize_patina::{LintPreset, Linter};

const RULE: &str = "script/no-duplicate-attr-inheritance";
const TEMPLATE: &str = "<template>\n  <Presence>\n    <Primitive v-bind=\"$attrs\"><slot /></Primitive>\n  </Presence>\n</template>\n";

fn reported(normal: &str, setup: &str) -> Vec<String> {
    let source = format!(
        "<script lang=\"ts\">\n{normal}\n</script>\n\n<script setup lang=\"ts\">\n{setup}\n</script>\n\n{TEMPLATE}"
    );
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(&source, "Indicator.vue");
    result
        .diagnostics
        .iter()
        .map(|diagnostic| source[diagnostic.start as usize..diagnostic.end as usize].to_owned())
        .collect()
}

#[test]
fn forwarding_is_reported_once_when_neither_block_disables_inheritance() {
    for (normal, setup) in [
        (
            "export interface Props { forceMount?: boolean }",
            "const { forwardRef } = useForwardExpose()",
        ),
        ("import type { X } from '@/x'", ""),
        ("export default { inheritAttrs: true }", ""),
        ("", "defineOptions({ inheritAttrs: true })"),
    ] {
        assert_eq!(reported(normal, setup), ["$attrs"], "{normal} / {setup}");
    }
}

#[test]
fn inheritance_disabled_or_opaque_in_either_block_is_not_reported() {
    for (normal, setup) in [
        ("export default { inheritAttrs: false }", ""),
        (
            "export default defineComponent({ inheritAttrs: false })",
            "",
        ),
        ("", "defineOptions({ inheritAttrs: false })"),
        (
            "const options = { inheritAttrs }\nexport default options",
            "",
        ),
        ("export default { inheritAttrs: flag }", ""),
    ] {
        assert_eq!(
            reported(normal, setup),
            Vec::<String>::new(),
            "{normal} / {setup}"
        );
    }
}
