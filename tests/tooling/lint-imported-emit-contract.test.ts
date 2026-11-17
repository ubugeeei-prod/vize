import assert from "node:assert/strict";
import { test } from "node:test";

import { patinaIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

const imported = /emits type imported from another module/u;

/** Patina's finding for `event` over the `occurrence`-th match of `text`. */
function reason(source: string, text: string, event: string, occurrence = 0): string | null {
  let start = -1;
  for (let seen = 0; seen <= occurrence; seen++) start = source.indexOf(text, start + 1);
  assert.ok(start >= 0, text);
  const before = source.slice(0, start).split("\n");
  const column = (before.at(-1)?.length ?? 0) + 1;
  return patinaIntentionalReason(
    {
      file: "src/Item.vue",
      ruleId: "vue/no-shadow-native-events",
      severity: "warning",
      line: before.length,
      column,
      endLine: before.length,
      endColumn: column + text.length,
      message: `[vize:vue/no-shadow-native-events] Avoid shadowing the native event \`${event}\``,
    },
    source,
  );
}

test("an event only an imported emits type declares is classified", () => {
  const direct = `<script setup lang="ts">
import type { MiniMapNodeEmits } from './types'
const emits = defineEmits<MiniMapNodeEmits>()
</script>`;
  assert.match(reason(direct, "MiniMapNodeEmits", "click", 1) ?? "", imported);
  const heritage = `<script setup lang="ts">
import type { MenuItemEmits } from '@/Menu'
export type DropdownMenuItemEmits = MenuItemEmits
const emits = defineEmits<DropdownMenuItemEmits>()
</script>`;
  assert.match(reason(heritage, "DropdownMenuItemEmits", "select", 1) ?? "", imported);
});

test("an event the file spells stays counted", () => {
  const member = `<script setup lang="ts">
import type { RootEmits } from 'reka-ui'
type GroupEmits = Omit<RootEmits, 'update:modelValue'> & { change: [event: Event] }
const emits = defineEmits<GroupEmits>()
</script>`;
  assert.equal(reason(member, "GroupEmits", "change", 1), null);
  const signature = `<script setup lang="ts">
import type { Base } from './base'
type Emits = Base & { (e: 'blur'): void }
defineEmits<Emits>()
</script>`;
  assert.equal(reason(signature, "Emits", "blur", 2), null);
  assert.match(reason(signature, "Emits", "focus", 2) ?? "", imported);
  const local = `<script setup lang="ts">
interface Emits { click: [] }
defineEmits<Emits>()
</script>`;
  assert.equal(reason(local, "Emits", "click", 2), null);
});
