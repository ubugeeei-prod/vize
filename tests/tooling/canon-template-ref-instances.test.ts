import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
  workspace,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

// A component ref holds the instance the template instantiates: the props
// bound next to `ref` choose the type parameters of a generic component, and
// `$refs` holds the same registry once the project asks for it.

const generic = `<script setup lang="ts" generic="const T">
const { foo } = defineProps<{ foo: T }>();
defineExpose({ foo });
</script>
<template><div /></template>`;

function project(files: Record<string, string>): string {
  const directory = workspace("template-ref-instances-");
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        module: "ESNext",
        moduleResolution: "Bundler",
      },
      include: ["*.vue"],
    }),
  );
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(directory, name), source);
  }
  return directory;
}

async function compare(directory: string, expected: string[]): Promise<void> {
  const oracle = vueTscDiagnostics(directory).sort(compareIdentity);
  assert.deepEqual(
    oracle.map((d) => `${d.file}:${d.line}:${d.column} TS${d.code}`),
    expected,
  );
  assert.deepEqual((await check(directory)).map(diagnosticIdentity).sort(compareIdentity), oracle);
}

const holder = (
  script: string,
  template: string,
  options = "",
) => `${options}<script setup lang="ts">
import { useTemplateRef } from 'vue';
import Generic from './Generic.vue';
const takesString = (value: string) => value;
const takesNumber = (value: number) => value;
${script}
</script>
<template>${template}</template>`;

test("useTemplateRef holds the generic instance its bound props instantiate", async () => {
  const template = `
  <Generic ref="one" :foo="1" />
  <Generic v-for="item in ['a', 'b']" ref="many" :key="item" :foo="item" />
  <Generic v-if="one" ref="guarded" :foo="one.foo" />`;
  const directory = project({
    "Generic.vue": generic,
    "App.vue": holder(
      `const one = useTemplateRef('one');
const many = useTemplateRef('many');
const guarded = useTemplateRef('guarded');
if (one.value) takesNumber(one.value.foo);
if (many.value?.[0]) takesString(many.value[0].foo);
if (guarded.value) takesNumber(guarded.value.foo);`,
      template,
    ),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      holder(
        `const one = useTemplateRef('one');
const many = useTemplateRef('many');
if (one.value) takesString(one.value.foo);
if (many.value?.[0]) takesNumber(many.value[0].foo);`,
        template,
      ),
    );
    await compare(directory, ["App.vue:8:28 TS2345", "App.vue:9:34 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("inferTemplateDollarRefs types the template's own $refs by the same registry", async () => {
  const directory = project({
    "Generic.vue": generic,
    "App.vue": holder(
      "",
      `
  <Generic ref="one" :foo="1" /><a ref="link" />
  {{ takesNumber($refs.one!.foo) }}{{ takesString($refs.link.href) }}`,
      "<!-- @inferTemplateDollarRefs true -->\n",
    ),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      holder(
        "",
        `
  <Generic ref="one" :foo="1" /><a ref="link" />
  {{ takesString($refs.one!.foo) }}{{ takesNumber($refs.link.href) }}{{ $refs.missing }}`,
        "<!-- @inferTemplateDollarRefs true -->\n",
      ),
    );
    await compare(directory, [
      "App.vue:11:18 TS2345",
      "App.vue:11:51 TS2345",
      "App.vue:11:79 TS2339",
    ]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("inferComponentDollarRefs exposes the registry on the public instance", async () => {
  const child = `<!-- @inferComponentDollarRefs true -->
<script setup lang="ts">
import Generic from './Generic.vue';
</script>
<template><Generic ref="one" :foo="1" /></template>`;
  const parent = (expression: string) =>
    holder(
      `import Child from './Child.vue';
const child = useTemplateRef('child');
${expression}`,
      '<Child ref="child" />',
    );
  const directory = project({
    "Generic.vue": generic,
    "Child.vue": child,
    "App.vue": parent("if (child.value?.$refs.one) takesNumber(child.value.$refs.one.foo);"),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent("if (child.value?.$refs.one) takesString(child.value.$refs.one.foo);"),
    );
    await compare(directory, ["App.vue:8:41 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
