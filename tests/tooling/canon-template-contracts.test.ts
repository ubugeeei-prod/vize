import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";
import { assertVueTsc } from "./support/vue-tsc-oracle.ts";

const require = createRequire(import.meta.url);
const vueRequire = createRequire(require.resolve("vue/package.json"));
const { parseStringStyle } = vueRequire("@vue/shared") as {
  parseStringStyle: (source: string) => Record<string, string>;
};

function project(prefix: string): string {
  const directory = workspace(prefix);
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        module: "esnext",
        moduleResolution: "bundler",
        skipLibCheck: true,
      },
      include: ["*.vue", "*.ts"],
    }),
  );
  return directory;
}

test("static component styles have Vue's normalized object type", async () => {
  const directory = project("static-style-contract-");
  const file = path.join(directory, "App.vue");
  try {
    for (const style of [
      "display: block;",
      "background:url(a;b); color:red",
      "color:red; /* remove: this; */ color:blue; --tone: teal",
      "padding: 1px; orphan; color: green; empty:",
      "font-family: 'a:b'; content: 'text'",
      "color:; display: block; color: /* empty */ ",
      "--カラー: 青; color: red",
      "__proto__: ignored; color: red; constructor: own",
      "\uFEFFcolor\uFEFF: \uFEFFred\uFEFF; \u0085padding: \u00851px\u0085",
      "background: a;b); color: red; padding: (a;b",
      "color: red; /* unfinished; padding: 1px",
    ]) {
      const expected = parseStringStyle(style);
      fs.writeFileSync(
        file,
        `<script setup lang="ts">\nconst expected = ${JSON.stringify(expected)} as const;\ndeclare function Comp(props: { style: typeof expected }): void;\n</script>\n<template><Comp style="${style}" /></template>`,
      );
      assert.deepEqual(await check(directory), [], style);
    }
    fs.writeFileSync(
      file,
      '<script setup lang="ts">\ndeclare function Comp(props: { style: { display: "grid" } }): void;\n</script>\n<template><Comp style="display: block" /></template>',
    );
    const diagnostics = await check(directory);
    assert.ok(diagnostics.length > 0, "a normalized style still checks the component contract");
    assert.ok(diagnostics.every((d) => d.file === "App.vue" && d.code === 2322));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("the template $ binding retains Vue's internal instance type", async () => {
  const directory = project("internal-instance-contract-");
  const file = path.join(directory, "App.vue");
  try {
    fs.writeFileSync(file, "<template>{{ $.uid }}</template>");
    assert.deepEqual(await check(directory), []);
    fs.writeFileSync(file, "<template>{{ $.missingInstanceProperty }}</template>");
    assert.deepEqual(
      (await check(directory)).map(({ file, code }) => ({ file, code })),
      [{ file: "App.vue", code: 2339 }],
    );
    fs.writeFileSync(
      file,
      '<script setup lang="ts">const $ = { authored: 1 };</script><template>{{ $.authored }}</template>',
    );
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("a primitive authored default export keeps its exact type with or without setup", async () => {
  const directory = project("authored-default-contract-");
  const child = path.join(directory, "Child.vue");
  const consumer = path.join(directory, "main.ts");
  try {
    for (const setup of ["", '<script setup lang="ts"></script>']) {
      for (const [value, type] of [
        ["1", "1"],
        ['"component"', '"component"'],
        ["null", "null"],
        ["true", "true"],
      ]) {
        fs.writeFileSync(
          child,
          `${setup}<script lang="ts">export default ${value} as ${type};</script>`,
        );
        fs.writeFileSync(
          consumer,
          `import child from './Child.vue';\nconst actual: ${type} = child;\nconst inverse: typeof child = ${value};\nexport { actual, inverse };`,
        );
        assert.deepEqual(await check(directory), [], `${setup || "normal script"}: ${type}`);
      }
    }
    fs.writeFileSync(
      child,
      '<script setup lang="ts"></script><script lang="ts">export default 1 as const;</script>',
    );
    fs.writeFileSync(
      consumer,
      "import child from './Child.vue';\nexport const incorrect: 2 = child;",
    );
    assert.deepEqual(
      (await check(directory)).map(({ file, code, line }) => ({ file, code, line })),
      [{ file: "main.ts", code: 2322, line: 2 }],
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("normal-script options preserve setup component inference", async () => {
  const directory = project("setup-options-contract-");
  try {
    fs.writeFileSync(
      path.join(directory, "Child.vue"),
      `<script lang="ts">export default { inheritAttrs: false, name: "Child" };</script>
<script setup lang="ts">
defineProps<{ name: string }>();
defineEmits<{ change: [value: string] }>();
defineSlots<{ default(props: { value: string }): unknown }>();
</script>`,
    );
    fs.writeFileSync(
      path.join(directory, "NoEmits.vue"),
      '<script setup lang="ts">defineExpose({ ping: () => "pong" });</script>',
    );
    fs.writeFileSync(
      path.join(directory, "main.ts"),
      `import Child from './Child.vue';
import NoEmits from './NoEmits.vue';
import type { Component, ComponentPublicInstance } from 'vue';
type Emit<C> = C extends new (...args: any[]) => { $emit: infer E } ? E : never;
declare function listen<C>(component: C, handler: Emit<C>): void;
listen(Child, (event, value) => {
  const name: 'change' = event;
  const text: string = value;
  // @ts-expect-error An inferred string must not silently become any.
  const invalid: number = value;
  return [name, text, invalid];
});
declare const child: InstanceType<typeof Child>;
const component: Component = Child;
// @ts-expect-error Vue's strict emit contract is not an unconstrained public instance.
const base: ComponentPublicInstance = child;
void component; void base;
child.$slots.default({ value: "text" });
// @ts-expect-error A declared slot must not silently accept a wrong payload.
child.$slots.default({ value: 42 });
// @ts-expect-error A closed slot map must not gain a compatibility index signature.
child.$slots.missing({ value: "text" });
declare const noEmits: InstanceType<typeof NoEmits>;
noEmits.$emit("any-event", 42);`,
    );
    assertVueTsc(directory);
    assert.deepEqual(await check(directory), []);
    fs.writeFileSync(
      path.join(directory, "Parent.vue"),
      `<script setup lang="ts">import Child from './Child.vue';</script>
<template><Child :name="42" /></template>`,
    );
    assert.deepEqual(
      (await check(directory)).map(({ file, code }) => ({ file, code })),
      [{ file: "Parent.vue", code: 2322 }],
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
