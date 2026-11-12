//! Musea `<art>` variants read `<script setup>` bindings like a template.

use super::tests::line_col;
use crate::{LintPreset, Linter};

fn reports(source: &str) -> Vec<(usize, usize, &'static str, String)> {
    let result = Linter::with_preset(LintPreset::Opinionated).lint_sfc(source, "Button.art.vue");
    let mut found: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.rule_name,
                "vue/no-unused-script-vars" | "vue/prefer-const"
            )
        })
        .map(|diagnostic| {
            let (line, column) = line_col(source, diagnostic.start);
            (
                line,
                column,
                diagnostic.rule_name,
                diagnostic.message.as_str().to_string(),
            )
        })
        .collect();
    found.sort();
    found
}

const ART_USES: &str = r#"<script setup lang="ts">
import { ref } from "vue";
import Button from "./Button.vue";
import IconStar from "./IconStar.vue";
import { vTooltip } from "./tooltip";
import { tones } from "./tones";
import Unused from "./Unused.vue";

let clicks = 0;
const anchor = ref<HTMLElement | null>(null);
const label = "Save";
const hidden = "never shown";
</script>

<art title="Button" component="./Button.vue">
  <variant name="Default" default>
    <Button :label="label" @click="clicks++">
      <icon-star />
    </Button>
  </variant>
  <section>
    <variant name="Tones">
      <div ref="anchor" v-tooltip>
        <Button v-for="tone in tones" :key="tone" :tone="tone" />
      </div>
    </variant>
  </section>
</art>
"#;

#[test]
fn art_variant_reads_count_as_uses() {
    assert_eq!(
        reports(ART_USES),
        vec![
            (
                7,
                8,
                "vue/no-unused-script-vars",
                "'Unused' is defined but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                12,
                7,
                "vue/no-unused-script-vars",
                "'hidden' is assigned a value but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
        ]
    );
}

#[test]
fn other_custom_blocks_and_art_metadata_read_nothing() {
    let source = r#"<script setup lang="ts">
import Button from "./Button.vue";
const title = "Button";
</script>

<art title="title" component="./Button.vue">
  <variant name="Button" args='{"title": 1}'>plain text</variant>
</art>

<docs>
  <Button :title="title" />
</docs>
"#;
    assert_eq!(
        reports(source),
        vec![
            (
                2,
                8,
                "vue/no-unused-script-vars",
                "'Button' is defined but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                3,
                7,
                "vue/no-unused-script-vars",
                "'title' is assigned a value but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
        ]
    );
}
