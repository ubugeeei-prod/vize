import assert from "node:assert/strict";
import { test } from "node:test";

import { patinaIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

const collision = /same-named binding or template variable collides/u;

/** The reason for a finding over the `occurrence`-th match of `text`. */
function reason(
  source: string,
  ruleId: "script/no-dupe-keys" | "vue/no-template-shadow",
  text: string,
  name: string,
  occurrence = 0,
): string | null {
  let start = -1;
  for (let seen = 0; seen <= occurrence; seen++) start = source.indexOf(text, start + 1);
  assert.ok(start >= 0, text);
  const position = (offset: number) => {
    const before = source.slice(0, offset).split("\n");
    return { line: before.length, column: (before.at(-1)?.length ?? 0) + 1 };
  };
  const from = position(start),
    to = position(start + text.length);
  return patinaIntentionalReason(
    {
      file: "src/Layout.vue",
      ruleId,
      severity: "warning",
      line: from.line,
      column: from.column,
      endLine: to.line,
      endColumn: to.column,
      message:
        ruleId === "script/no-dupe-keys"
          ? `[vize:script/no-dupe-keys] Duplicated key '${name}'`
          : `[vize:vue/no-template-shadow] Variable '${name}' shadows a variable in an outer scope`,
    },
    source,
  );
}

const layout = `<script setup lang="ts">
import type { LayoutProps } from './layout'
interface Props extends LayoutProps {}
const props = withDefaults(defineProps<Props>(), { sidebarWidth: 180 })
const sidebarCollapse = defineModel<boolean>('sidebarCollapse', {
  default: false,
})
</script>
`;

test("a defineModel binding colliding with an imported prop type is classified", () => {
  const declarator = `sidebarCollapse = defineModel<boolean>('sidebarCollapse', {
  default: false,
})`;
  assert.match(
    reason(layout, "script/no-dupe-keys", declarator, "sidebarCollapse") ?? "",
    collision,
  );
  assert.match(
    reason(layout, "script/no-dupe-keys", "'sidebarCollapse'", "sidebarCollapse") ?? "",
    collision,
  );
});

test("a binding colliding with a picked imported prop is classified past a renamed key", () => {
  const source = `<script setup lang="ts">
import type { RovingFocusGroupProps } from '@/RovingFocus'
export interface GroupProps extends Pick<RovingFocusGroupProps, 'dir' | 'loop'> {}
const props = defineProps<GroupProps>()
const { dir: propDir } = toRefs(props)
const dir = useDirection(propDir)
</script>
`;
  assert.match(
    reason(source, "script/no-dupe-keys", "dir = useDirection(propDir)", "dir") ?? "",
    collision,
  );
});

test("a binding colliding with an imported runtime props object is classified", () => {
  const source = `<script setup lang="ts">
import { affixProps } from './affix'
const props = defineProps(affixProps)
const target = ref<HTMLElement>()
</script>
`;
  assert.match(
    reason(source, "script/no-dupe-keys", "target = ref<HTMLElement>()", "target") ?? "",
    collision,
  );
});

test("a props destructure that reads the prop back does not declare it again", () => {
  const source = `<script setup lang="ts">
import type { PageHeaderProps } from './MkPageHeader.vue'
const props = withDefaults(defineProps<PageHeaderProps & { reversed?: boolean }>(), {
  reversed: false,
})
const headerProps = computed(() => {
  const { reversed, tab, ...rest } = props
  return rest
})
const tab = defineModel<string>('tab')
</script>
`;
  const declarator = "tab = defineModel<string>('tab')";
  assert.match(reason(source, "script/no-dupe-keys", declarator, "tab") ?? "", collision);
  assert.match(reason(source, "script/no-dupe-keys", "'tab'", "tab") ?? "", collision);
  for (const other of ["= other", "= props.nested", "= props?.nested", "= props[key]"]) {
    const nested = source.replace("} = props\n", `} ${other}\n`);
    assert.equal(reason(nested, "script/no-dupe-keys", declarator, "tab"), null, other);
  }
});

test("duplicate keys the SFC can declare itself stay counted", () => {
  const sameFileMember = `<script setup lang="ts">
import type { Base } from './base'
interface Props extends Base { open?: boolean }
const props = defineProps<Props>()
const open = ref(false)
</script>
`;
  assert.equal(reason(sameFileMember, "script/no-dupe-keys", "open = ref(false)", "open"), null);
  const runtimeKey = `<script setup lang="ts">
import { affixProps } from './affix'
const props = defineProps({ ...affixProps, target: String })
const target = ref<HTMLElement>()
</script>
`;
  assert.equal(
    reason(runtimeKey, "script/no-dupe-keys", "target = ref<HTMLElement>()", "target"),
    null,
  );
  const declaredTwice = `${layout.replace(
    "</script>",
    "function reset() {\n  const sidebarCollapse = false\n}\n</script>",
  )}`;
  assert.equal(
    reason(declaredTwice, "script/no-dupe-keys", "'sidebarCollapse'", "sidebarCollapse"),
    null,
  );
  const localOnly = `<script setup lang="ts">
interface Props { open?: boolean }
const props = defineProps<Props>()
const open = ref(false)
</script>
`;
  assert.equal(reason(localOnly, "script/no-dupe-keys", "open = ref(false)", "open"), null);
});

test("a template variable shadowing an imported prop is classified", () => {
  const source = `<script setup lang="ts">
import type { PinInputRootProps } from '..'
const props = defineProps<PinInputRootProps>()
</script>

<template>
  <PinInputInput v-for="(id, index) in 5" :key="id" :index="index" />
</template>
`;
  assert.match(reason(source, "vue/no-template-shadow", "id", "id") ?? "", collision);
});

test("a slot interface repeating the prop name does not declare a prop", () => {
  const source = `<script setup lang="ts">
import type { CollapsibleRootProps } from 'reka-ui'
export interface CollapsibleProps extends Pick<CollapsibleRootProps, 'open'> {}
export interface CollapsibleSlots {
  default?(props: { open: boolean }): VNode[]
}
const props = defineProps<CollapsibleProps>()
defineSlots<CollapsibleSlots>()
</script>

<template>
  <CollapsibleRoot v-slot="{ open }"><slot :open="open" /></CollapsibleRoot>
</template>
`;
  assert.match(reason(source, "vue/no-template-shadow", "open", "open", 2) ?? "", collision);
});

test("template variables the SFC can declare itself stay counted", () => {
  const binding = `<script setup lang="ts">
import type { CollapsibleRootProps } from 'reka-ui'
const props = defineProps<CollapsibleRootProps>()
const open = ref(false)
</script>

<template>
  <CollapsibleRoot v-slot="{ open }" />
</template>
`;
  assert.equal(reason(binding, "vue/no-template-shadow", "open", "open", 1), null);
  const member = `<script setup lang="ts">
import type { CollapsibleRootProps } from 'reka-ui'
interface Props extends CollapsibleRootProps { open?: boolean }
const props = defineProps<Props>()
</script>

<template>
  <CollapsibleRoot v-slot="{ open }" />
</template>
`;
  assert.equal(reason(member, "vue/no-template-shadow", "open", "open", 1), null);
});
