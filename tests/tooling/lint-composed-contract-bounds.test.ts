import assert from "node:assert/strict";
import { test } from "node:test";

import { patinaIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

const finding = {
  file: "src/ToggleGroupRoot.vue",
  ruleId: "script/require-default-prop",
  severity: "error",
  line: 14,
  column: 40,
  endLine: 14,
  endColumn: 60,
  message: "Prop 'defaultValue' requires a default value.",
};

test("generic type parameters cannot expand the composed-contract worklist", () => {
  const source = `<script lang="ts">
interface ToggleGroupRootProps<T> extends PrimitiveProps, FormFieldProps, ValueProps<T> {
  orientation?: string
}
type ToggleGroupRootEmits = {
  change: [value: string]
}
interface ToggleGroupRootContext {
  value: string
}
</script>
<script setup lang="ts">
const props = withDefaults(
  defineProps<ToggleGroupRootProps<string>>(),
  {},
)
</script>
`;

  assert.equal(patinaIntentionalReason(finding, source), null);
});

test("bounded composed-contract traversal retains local heritage classification", () => {
  const source = `<script lang="ts">
interface SharedProps {
  label?: string
}
interface IntermediateProps extends SharedProps {}
interface RootProps extends IntermediateProps {}
</script>
<script setup lang="ts">
const props = withDefaults(
  defineProps<RootProps>(),
  {},
)
</script>
`;
  const inheritedFinding = {
    ...finding,
    line: 3,
    column: 3,
    endLine: 3,
    endColumn: 17,
  };

  assert.match(
    patinaIntentionalReason(inheritedFinding, source) ?? "",
    /local interface heritage/u,
  );
});
