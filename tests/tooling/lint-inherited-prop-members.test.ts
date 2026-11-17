import assert from "node:assert/strict";
import { test } from "node:test";

import { patinaIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

const inherited = /never follows interface heritage/u;

/** A require-prop-comment finding on the first `name` member after `after`. */
function reason(source: string, name: string, after = ""): string | null {
  const start = source.indexOf(`${name}`, source.indexOf(after));
  assert.ok(start >= 0, name);
  const before = source.slice(0, start).split("\n");
  const line = before.length,
    column = (before.at(-1)?.length ?? 0) + 1;
  return patinaIntentionalReason(
    {
      file: "src/Slider.vue",
      ruleId: "vue/require-prop-comment",
      severity: "warning",
      line,
      column,
      endLine: line,
      endColumn: column + name.length,
      message: `[vize:vue/require-prop-comment] The "${name}" property should have a JSDoc comment.`,
    },
    source,
  );
}

const slider = `<script setup lang="ts">
interface BaseProps {
  disabled?: boolean;
  minValue?: number;
}
interface NumberProps extends BaseProps {
  type: 'integer';
}
interface StringProps extends BaseProps {
  type: 'decimal';
}
const props = withDefaults(defineProps<NumberProps | StringProps>(), { minValue: 0 });
</script>
`;

test("a member the props inherit through extends is classified", () => {
  assert.match(reason(slider, "disabled") ?? "", inherited);
  assert.match(reason(slider, "minValue") ?? "", inherited);
  const aliased = slider
    .replace("defineProps<NumberProps | StringProps>()", "defineProps<Props>()")
    .replace("const props", "type Props = NumberProps | StringProps\nconst props");
  assert.match(reason(aliased, "disabled") ?? "", inherited);
});

test("members eslint-plugin-vue reads itself stay counted", () => {
  assert.equal(reason(slider, "type", "interface NumberProps"), null);
  const direct = slider.replace("NumberProps | StringProps>", "NumberProps & BaseProps>");
  assert.equal(reason(direct, "disabled"), null);
  const used = slider.replace("const props", "const fallback: BaseProps = {}\nconst props");
  assert.equal(reason(used, "disabled"), null);
  const unrelated = slider
    .replace("interface NumberProps extends BaseProps", "interface NumberProps")
    .replace("interface StringProps extends BaseProps", "interface StringProps")
    .replace("</script>", "interface Other extends BaseProps {}\n</script>");
  assert.equal(reason(unrelated, "disabled"), null);
});
