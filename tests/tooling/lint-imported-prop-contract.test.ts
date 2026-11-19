import assert from "node:assert/strict";
import { test } from "node:test";

import {
  booleanDefaultFromImportedContract,
  propFindingFromImportedContract,
} from "../../tools/support/fixtures/lint_divergence/imported_prop_contract.ts";
import { patinaIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

const imported = /type contract imported from another module/u;

/** A finding over the `occurrence`-th match of `text` in `source`. */
function at(
  source: string,
  ruleId: string,
  text: string,
  message: string,
  occurrence = 0,
): Parameters<typeof patinaIntentionalReason>[0] {
  let start = -1;
  for (let seen = 0; seen <= occurrence; seen++) start = source.indexOf(text, start + 1);
  assert.ok(start >= 0, text);
  const position = (offset: number) => {
    const before = source.slice(0, offset).split("\n");
    return { line: before.length, column: (before.at(-1)?.length ?? 0) + 1 };
  };
  const from = position(start),
    to = position(start + text.length);
  return {
    file: "src/Wrapper.vue",
    ruleId,
    severity: ruleId === "script/require-default-prop" ? "error" : "warning",
    line: from.line,
    column: from.column,
    endLine: to.line,
    endColumn: to.column,
    message,
  };
}

const requireDefault = (source: string, text: string, prop: string, occurrence = 0) =>
  patinaIntentionalReason(
    at(
      source,
      "script/require-default-prop",
      text,
      `[vize:script/require-default-prop] Prop '${prop}' requires a default value.`,
      occurrence,
    ),
    source,
  );

const booleanDefault = (source: string, text: string, occurrence = 0) =>
  patinaIntentionalReason(
    at(
      source,
      "script/no-boolean-default",
      text,
      "[vize:script/no-boolean-default] Unexpected default on a Boolean prop.",
      occurrence,
    ),
    source,
  );

/** The classifier alone, before any other intentional-divergence rule. */
function range(source: string, text: string, occurrence = 0): [number, number] {
  let start = -1;
  for (let seen = 0; seen <= occurrence; seen++) start = source.indexOf(text, start + 1);
  assert.ok(start >= 0, text);
  return [start, start + text.length];
}
const importedDefault = (source: string, text: string, prop: string, occurrence = 0) =>
  propFindingFromImportedContract(
    `Prop '${prop}' requires a default value.`,
    ...range(source, text, occurrence),
    source,
  );
const importedBoolean = (source: string, text: string, occurrence = 0) =>
  booleanDefaultFromImportedContract(...range(source, text, occurrence), source);

const intersection = `<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import type { DropdownMenuContentProps } from 'reka-ui'

const props = withDefaults(
  defineProps<DropdownMenuContentProps & { class?: HTMLAttributes['class'], inset?: boolean }>(),
  { sideOffset: 4, loop: true, inset: false },
)
</script>
`;

test("an imported optional prop reported on the type argument is classified", () => {
  assert.match(requireDefault(intersection, "DropdownMenuContentProps", "as", 1) ?? "", imported);
});

test("a local optional prop next to an imported type stays counted", () => {
  assert.equal(requireDefault(intersection, "DropdownMenuContentProps", "class", 1), null);
});

test("an imported Boolean default is classified, a local one stays counted", () => {
  assert.match(booleanDefault(intersection, "true") ?? "", imported);
  assert.equal(booleanDefault(intersection, "false"), null);
});

test("local aliases and heritage lead to the import", () => {
  const source = `<script setup lang="ts">
import type { AvatarProps as Base } from './avatar'
interface Shared extends Base {}
type Props = Shared
const { shape = 'circle', square = false } = defineProps<Props>()
</script>
`;
  assert.equal(importedDefault(source, "Props", "icon", 3), true);
  assert.equal(importedBoolean(source, "false"), true);
});

test("only types the props argument reaches declare props", () => {
  const source = `<script lang="ts">
import type { MenuProps } from '@/Menu'
export interface RootProps extends MenuProps { defaultOpen?: boolean }
export interface RootContext { modal: Ref<boolean>; dir: Ref<Direction> }
</script>
<script setup lang="ts">
const props = withDefaults(defineProps<RootProps>(), {
  // Keep Vue's boolean coercion out of \`open\`.
  modal: true,
  defaultOpen: false,
})
</script>
`;
  assert.equal(importedDefault(source, "RootProps", "dir", 1), true);
  assert.equal(importedBoolean(source, "true"), true);
  assert.equal(importedBoolean(source, "false"), false);
  const reached = source.replace("extends MenuProps {", "extends MenuProps, RootContext {");
  assert.equal(importedDefault(reached, "RootProps", "dir", 1), false);
  assert.equal(importedBoolean(reached, "true"), false);
});

test("a same-file contract without imports is never classified", () => {
  const source = `<script lang="ts">
export interface Props { label?: string; open?: boolean }
</script>
<script setup lang="ts">
const props = withDefaults(defineProps<Props>(), { open: true })
</script>
`;
  assert.equal(importedDefault(source, "Props", "label", 2), false);
  assert.equal(importedBoolean(source, "true"), false);
});

test("only the defineProps type argument and defaults are accepted", () => {
  const source = `<script setup lang="ts">
import type { AlertProps } from './alert'
const fallback = true
const props = withDefaults(defineProps<AlertProps>(), { closable: true })
</script>
`;
  assert.equal(requireDefault(source, "AlertProps", "title"), null);
  assert.match(requireDefault(source, "AlertProps", "title", 1) ?? "", imported);
  assert.equal(booleanDefault(source, "true"), null);
  assert.match(booleanDefault(source, "true", 1) ?? "", imported);
});

test("runtime object keys do not declare props; multi-line local types do", () => {
  const source = `<script setup lang="ts">
import type { ButtonProps } from './button'
type Props = ButtonProps
  & {
    extra?: string
  }
const props = withDefaults(defineProps<Props>(), { plain: false })
const style = computed(() => ({ size: props.size, extra: 1 }))
</script>
`;
  assert.equal(importedDefault(source, "Props", "size", 4), true);
  assert.equal(importedDefault(source, "Props", "extra", 4), false);
  assert.equal(importedBoolean(source, "false"), true);
});

test("props named in other imported-contract rules classify, a type name never does", () => {
  const source = `<script setup lang="ts">
import type { SettingProps } from './types'
import type { ScalarFloatingOptions } from '../ScalarFloating'
withDefaults(defineProps<SettingProps & ScalarFloatingOptions>(), {
  formSchema: () => [],
})
</script>`;
  const reason = (ruleId: string, message: string) =>
    patinaIntentionalReason(
      at(source, ruleId, "SettingProps & ScalarFloatingOptions", message),
      source,
    );
  assert.match(
    reason(
      "script/no-required-prop-with-default",
      "[vize:script/no-required-prop-with-default] Prop 'formSchema' is required but also declares a default",
    ) ?? "",
    imported,
  );
  assert.match(
    reason(
      "vue/prop-name-casing",
      '[vize:vue/prop-name-casing] Prop "onUpdate:visible" is not in camelCase',
    ) ?? "",
    imported,
  );
  assert.equal(
    reason(
      "vue/prop-name-casing",
      '[vize:vue/prop-name-casing] Prop "ScalarFloatingOptions" is not in camelCase',
    ),
    null,
  );
});
