import assert from "node:assert/strict";
import { test } from "node:test";

import {
  baselineIntentionalReason,
  patinaIntentionalReason,
} from "../../tools/support/fixtures/lint_divergence/records.ts";

const source = `<script>
import { nextTick } from 'vue'
export default {
  mounted() {
    this.$nextTick(() => this.measure())
  },
}
</script>
<script lang="ts" setup>
nextTick(() => undefined)
await nextTick(() => undefined)
</script>
`;

function finding(ruleId: string, line: number, column: number, length: number, message = "") {
  return {
    file: "src/Panel.vue",
    ruleId,
    severity: "warning",
    line,
    column,
    endLine: line,
    endColumn: column + length,
    message,
  };
}

test("next-tick-style findings in <script setup> are a documented extension", () => {
  assert.match(
    patinaIntentionalReason(finding("vue/next-tick-style", 10, 1, 8), source) ?? "",
    /#12450.*#12670/u,
  );
});

test("next-tick-style findings in component options stay counted", () => {
  assert.equal(patinaIntentionalReason(finding("vue/next-tick-style", 5, 10, 9), source), null);
});

test("valid-next-tick findings in <script setup> are a documented extension", () => {
  assert.match(
    patinaIntentionalReason(finding("script/valid-next-tick", 11, 7, 8), source) ?? "",
    /#12794/u,
  );
});

test("valid-next-tick findings in component options stay counted", () => {
  assert.equal(patinaIntentionalReason(finding("script/valid-next-tick", 5, 10, 9), source), null);
});

test("custom-event-name-casing keeps the model prop name after update:", () => {
  const reported = (name: string) =>
    baselineIntentionalReason(
      finding(
        "vue/custom-event-name-casing",
        3,
        8,
        name.length + 2,
        `Custom event name '${name}' must be camelCase.`,
      ),
      source,
    );
  assert.match(reported("update:selected-date") ?? "", /#9879.*#12592.*#12587/u);
  assert.equal(reported("selected-date"), null);
  assert.equal(reported("close-dialog:done"), null);
});

/** A finding over the `occurrence`-th match of `text` in `markup`. */
function classFinding(
  markup: string,
  ruleId: string,
  text: string,
  message: string,
  occurrence = 0,
) {
  let start = -1;
  for (let seen = 0; seen <= occurrence; seen++) start = markup.indexOf(text, start + 1);
  assert.ok(start >= 0, text);
  const before = markup.slice(0, start).split("\n");
  return {
    file: "src/Classes.vue",
    ruleId,
    severity: "warning",
    line: before.length,
    column: (before.at(-1)?.length ?? 0) + 1,
    endLine: before.length,
    endColumn: (before.at(-1)?.length ?? 0) + 1 + text.length,
    message,
  };
}

test("a class repeated only through an identifier key is Patina's finding", () => {
  const markup = `<template>
  <ul :class="{ hidden: wide }" class="hidden sm:flex" />
  <ul :class="['grid', { grid: cond }]" />
  <ul :class="{ 'flex': cond }" class="flex" />
  <ul :class="{ gap }" class="gap gap" />
</template>
`;
  const patina = (text: string, name: string, occurrence = 0) =>
    patinaIntentionalReason(
      classFinding(
        markup,
        "html/no-duplicate-class",
        text,
        `[vize:html/no-duplicate-class] Duplicate class name '${name}'`,
        occurrence,
      ),
      markup,
    );
  assert.match(patina("<ul", "hidden") ?? "", /identifier object key/u);
  assert.match(patina("['grid', { grid: cond }]", "grid") ?? "", /identifier object key/u);
  // A string-literal key and a static value repeated on its own are upstream's too.
  assert.equal(patina("<ul", "flex", 2), null);
  assert.equal(patina("<ul", "gap", 3), null);
});

test("a fragment glued to an interpolation is not a whole class", () => {
  const markup = `<template>
  <a :class="\`text-\${color}-500 hover:text-\${color}-600 focus:text-\${color}-600\`" />
  <i :class="[\`node--\${size}\`, \`node--\${type}\`]" />
  <b :class="['px-2 ', \`px-2 \${size}\`]" />
</template>
`;
  const upstream = (text: string, name: string) =>
    baselineIntentionalReason(
      classFinding(markup, "vue/no-duplicate-class-names", text, `Duplicate class name '${name}'.`),
      markup,
    );
  assert.match(upstream("`text-", "-600") ?? "", /glued to a \$\{\} interpolation/u);
  assert.match(upstream("[`node--", "node--") ?? "", /glued to a \$\{\} interpolation/u);
  // A whole token bounded by whitespace is a class Patina also compares.
  assert.equal(upstream("['px-2 '", "px-2"), null);
});

test("a ref call directly under a type assertion is Patina's stricter stance", () => {
  const script = `<script setup lang="ts">
import { ref, type Ref } from 'vue'
const editor = ref(null) as unknown as Ref<HTMLElement>
const focused = (ref()) satisfies Ref<Date | undefined>
const plain = ref(null)
const wrapped = cast(ref(null)) as Ref<HTMLElement>
</script>
`;
  const typedRef = (text: string, occurrence = 0) =>
    patinaIntentionalReason(
      classFinding(
        script,
        "script/require-typed-ref",
        text,
        "[vize:script/require-typed-ref] This ref() should have an explicit type argument.",
        occurrence,
      ),
      script,
    );
  assert.match(typedRef("ref(null)") ?? "", /#12069/u);
  assert.match(typedRef("ref()") ?? "", /#12069/u);
  assert.equal(typedRef("ref(null)", 1), null);
  assert.equal(typedRef("ref(null)", 2), null);
});

test("SVG elements outside an <svg> ancestor are native to Vue", () => {
  const keyed = (tag: string) =>
    baselineIntentionalReason(
      {
        file: "src/Arrow.vue",
        ruleId: "vue/v-if-else-key",
        severity: "error",
        line: 1,
        column: 1,
        endLine: 1,
        endColumn: 2,
        message: `Conditionally rendered repeated component '${tag}' expected to have a 'key' attribute.`,
      },
      '<template><Primitive><path v-if="a" /><path v-else /></Primitive></template>',
    );
  for (const tag of ["path", "g", "line", "polyline", "text"])
    assert.match(keyed(tag) ?? "", /native/u);
  for (const tag of ["MyIcon", "my-path", "Path"]) assert.equal(keyed(tag), null);
});

test("a payload call authored before defineEmits is Patina's finding", () => {
  const sfc = `<template>
  <button @click="emit('toggled', a, b)" />
</template>
<script setup lang="ts">
function insert(text: string) {
  emit('execute-action', 'insertText', text)
}
const emit = defineEmits<{ 'execute-action': [string, string]; toggled: [number, number] }>()
emit('toggled', 1, 2)
</script>
`;
  const payload = (text: string) =>
    patinaIntentionalReason(
      classFinding(
        sfc,
        "vue/prefer-single-event-payload",
        text,
        "[vize:vue/prefer-single-event-payload] Pass a single payload object instead of multiple arguments.",
      ),
      sfc,
    );
  assert.match(payload("text)") ?? "", /before the macro declaration/u);
  assert.equal(payload("2)"), null);
  assert.equal(payload("b)"), null);
});

test("emits eslint-plugin-vue cannot see stay Patina's reading", () => {
  const unused = (sfc: string, event: string) =>
    baselineIntentionalReason(
      {
        file: "src/Picker.vue",
        ruleId: "vue/no-unused-emit-declarations",
        severity: "warning",
        line: 1,
        column: 1,
        endLine: 1,
        endColumn: 2,
        message: `\`${event}\` is defined as emit but never used.`,
      },
      sfc,
    );
  const asserted = `<script setup lang="ts">
const emit = defineEmits<{ apply: [unknown]; change: [] }>()
function pick(payload: unknown) {
  (emit as any)('apply', payload)
}
</script>`;
  assert.match(unused(asserted, "apply") ?? "", /type assertion/u);
  const early = `<script setup lang="ts">
function edit() {
  emit("edit-environment")
}
const emit = defineEmits<{ (e: "edit-environment"): void; (e: "select"): void }>()
</script>`;
  assert.match(unused(early, "edit-environment") ?? "", /before the macro declaration/u);
  assert.equal(unused(early, "select"), null);
  const plain = `<script setup lang="ts">
const emit = defineEmits<{ apply: []; change: [] }>()
emit('change')
</script>`;
  assert.equal(unused(plain, "apply"), null);
});

test("an inline intersection prop beside a defaults spread stays Patina's check", () => {
  const sfc = `<script setup lang="ts" generic="T">
import { StatefulProps, statefulPropsDefaults } from './stateful'
type Props = StatefulProps<T> & {
  clearValue?: T
}
const props = withDefaults(defineProps<Props>(), {
  ...statefulPropsDefaults,
})
</script>`;
  const requireDefault = (source: string) => {
    const start = source.indexOf("clearValue?: T");
    const before = source.slice(0, start).split("\n");
    const line = before.length,
      column = (before.at(-1)?.length ?? 0) + 1;
    return patinaIntentionalReason(
      {
        file: "src/VaFormField.vue",
        ruleId: "script/require-default-prop",
        severity: "warning",
        line,
        column,
        endLine: line,
        endColumn: column + 14,
        message: "[vize:script/require-default-prop] Prop 'clearValue' requires a default value.",
      },
      source,
    );
  };
  assert.match(requireDefault(sfc) ?? "", /inline intersection member/u);
  assert.equal(requireDefault(sfc.replace("  ...statefulPropsDefaults,\n", "")), null);
  const inline = sfc
    .replace("type Props = StatefulProps<T> & {\n  clearValue?: T\n}\n", "")
    .replace("defineProps<Props>()", "defineProps<StatefulProps<T> & {\n  clearValue?: T\n}>()");
  assert.equal(requireDefault(inline), null);
});

test("an asserted method handler is Vue's method handler", () => {
  const handler = (value: string) => {
    const sfc = `<template>\n  <div @keydown="${value}" />\n</template>`;
    const start = sfc.indexOf(value);
    return baselineIntentionalReason(
      {
        file: "src/SelectContentImpl.vue",
        ruleId: "vue/v-on-handler-style",
        severity: "warning",
        line: 2,
        column: start - sfc.lastIndexOf("\n", start),
        endLine: 2,
        endColumn: start - sfc.lastIndexOf("\n", start) + value.length,
        message:
          "Prefer method handler over inline handler in v-on. Note that you may need to create a new method.",
      },
      sfc,
    );
  };
  for (const value of ["(handleKeyDown as any)", "(menu.onKeydown satisfies Handler)", "handler!"])
    assert.match(handler(value) ?? "", /unwraps parentheses and TypeScript/u, value);
  for (const value of [
    "handleKeyDown($event)",
    "(handleKeyDown as (e: Event) => void)",
    "open = true",
  ])
    assert.equal(handler(value), null, value);
});
