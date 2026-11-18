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
  const objectAlias = slider.replace("interface BaseProps", "type BaseProps =");
  assert.match(reason(objectAlias, "disabled") ?? "", inherited);
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

/** A shadow finding naming `name`, over the first `text`. */
function shadow(
  source: string,
  ruleId: "vue/no-props-shadow" | "vue/no-template-shadow",
  name: string,
  text = name,
): string | null {
  const start = source.indexOf(text);
  assert.ok(start >= 0, text);
  const before = source.slice(0, start).split("\n");
  const line = before.length,
    column = (before.at(-1)?.length ?? 0) + 1;
  return patinaIntentionalReason(
    {
      file: "src/Link.vue",
      ruleId,
      severity: "warning",
      line,
      column,
      endLine: line,
      endColumn: column + name.length,
      message:
        ruleId === "vue/no-props-shadow"
          ? `[vize:vue/no-props-shadow] Declaration of \`${name}\` shadows the prop with the same name`
          : `[vize:vue/no-template-shadow] Variable '${name}' shadows a variable in an outer scope`,
    },
    source,
  );
}

const link = `<script lang="ts">
import type { AnchorHTMLAttributes } from './html'
interface NuxtLinkProps {
  to?: string
  href?: string
}
export interface LinkProps extends NuxtLinkProps, /** @vue-ignore */ Omit<AnchorHTMLAttributes, 'href' | 'rel'> {
  as?: string
}
</script>
<script setup lang="ts">
const props = defineProps<LinkProps>()
const to = computed(() => props.to ?? props.href)
</script>
<template><NuxtLink v-slot="{ href }" custom><a :href="href" /></NuxtLink></template>
`;
const heritage = /declared only in a same-file interface the defineProps type extends/u;

test("a shadowed prop the props inherit through extends is classified", () => {
  assert.match(shadow(link, "vue/no-props-shadow", "to", "to = computed") ?? "", heritage);
  assert.match(shadow(link, "vue/no-template-shadow", "href", "href }") ?? "", heritage);
});

test("shadowed props eslint-plugin-vue can read stay counted", () => {
  const own = link.replace("as?: string", "as?: string\n  to?: string");
  assert.equal(shadow(own, "vue/no-props-shadow", "to", "to = computed"), null);
  const omitted = link.replace("extends NuxtLinkProps, ", "extends Omit<NuxtLinkProps, 'to'>, ");
  assert.equal(shadow(omitted, "vue/no-props-shadow", "to", "to = computed"), null);
  assert.match(shadow(omitted, "vue/no-template-shadow", "href", "href }") ?? "", heritage);
  const bound = link.replace("const to =", "const href = props.href\nconst to =");
  assert.equal(shadow(bound, "vue/no-template-shadow", "href", "href }"), null);
  const imported = link.replace("import type {", "import { href, type");
  assert.equal(shadow(imported, "vue/no-template-shadow", "href", "href }"), null);
});
