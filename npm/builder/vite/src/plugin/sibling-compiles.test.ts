/**
 * A build compiles each SFC once for its client and SSR environments
 * (`sibling-compiles.ts`), and every environment still emits what it emitted
 * when each one compiled on its own.
 */

// Must stay the first import: it wraps the native compile functions the plugin captures.
import { labelledComponent, scratchProject } from "../test/native-compile-calls.ts";

import assert from "node:assert/strict";
import { test } from "node:test";

import { buildApp, buildAppWithoutTargets, type AppEnvironments } from "../test/app-build.ts";
import { linkWorkspaceVue } from "../test/startup-scan-project.ts";

const labelled = Array.from({ length: 4 }, (_, i) => `src/components/Item${i}.vue`);
const typed = "src/components/Typed.vue";
const warned = "src/components/Warned.vue";
const styled = "src/components/Styled.vue";

function project(extra: Record<string, string> = {}) {
  const components = [...labelled, typed, warned, styled, ...Object.keys(extra)];
  const files: Record<string, string> = {
    "src/app.ts": `${components
      .map((file, i) => `import C${i} from "./${file.slice("src/".length)}";`)
      .join(
        "\n",
      )}\n\nexport const components = [${components.map((_, i) => `C${i}`).join(", ")}];\n`,
    "src/main.ts": `import { components } from "./app";\n\nconsole.log(components);\n`,
    "src/entry-server.ts": `import { components } from "./app";\n\nexport const render = () => components;\n`,
    "src/worker.ts": `export const handler = (): string => "ok";\n`,
    "src/types.ts": `export interface ItemProps {\n  id: string;\n  label: string;\n}\n`,
    "tsconfig.json": `{ "compilerOptions": { "strict": true } }\n`,
    [typed]: `<script setup lang="ts">
import type { ItemProps } from "../types";

const props = defineProps<ItemProps>();
</script>

<template><p :id="props.id">{{ props.label }}</p></template>
`,
    [warned]: `<script setup lang="ts">
import { reactive } from "vue";
const form = reactive({ name: "" });
</script>

<template><input v-model="form" /></template>
`,
    [styled]: `<script setup lang="ts">
const color = "red";
</script>

<template><p class="styled">styled</p></template>

<style scoped>
.styled { color: v-bind(color); }
</style>
`,
    ...extra,
  };
  for (const file of labelled) files[file] = labelledComponent(file);
  const scratch = scratchProject(files);
  linkWorkspaceVue(scratch.root);
  return { ...scratch, components: components.sort() };
}

const app: AppEnvironments = {
  client: { entry: "src/main.ts" },
  ssr: { entry: "src/entry-server.ts", consumer: "server" },
  // Imports no component, as Nuxt's Nitro pass imports none.
  worker: { entry: "src/worker.ts", consumer: "server" },
};

void test("the SSR environment reuses what the client batch compiled for it, and emits the same files", async () => {
  const built = project();
  try {
    const apart = buildAppWithoutTargets(built.root, app);
    assert.deepEqual(apart.calls, {
      client: [{ api: "batch", ssr: false, files: built.components }],
      ssr: [{ api: "batch", ssr: true, files: built.components }],
      worker: [],
    });

    const shared = await buildApp(built.root, app);
    assert.deepEqual(shared.calls, {
      client: [{ api: "targets", ssr: false, alsoSsr: true, files: built.components }],
      ssr: [],
      worker: [],
    });
    assert.equal(shared.error, undefined);
    assert.deepEqual(Object.keys(shared.outputs).sort(), ["client", "ssr", "worker"]);
    for (const environment of Object.keys(app)) {
      assert.deepEqual(shared.outputs[environment], apart.outputs[environment], environment);
    }
    // Each environment prints a component's warnings once, as it did on its own.
    assert.equal(apart.warnings.client.length > 0, true, "the fixture must warn");
    assert.deepEqual(shared.warnings, apart.warnings);
  } finally {
    built.cleanup();
  }
});

void test("a client-only build makes the native calls it made before", async () => {
  const built = project();
  try {
    const client = { client: app.client };
    const shared = await buildApp(built.root, client);
    const apart = buildAppWithoutTargets(built.root, client);
    assert.deepEqual(shared.calls, {
      client: [{ api: "batch", ssr: false, files: built.components }],
    });
    assert.deepEqual(shared.calls, apart.calls);
    assert.deepEqual(shared.outputs, apart.outputs);
  } finally {
    built.cleanup();
  }
});

void test("an SSR text a host transform rewrote is compiled from that text, on its own", async () => {
  const built = project();
  try {
    const shared = await buildApp(built.root, app, ["rewrite-ssr-item0"]);
    const apart = buildAppWithoutTargets(built.root, app, ["rewrite-ssr-item0"]);
    assert.deepEqual(shared.calls.ssr, [
      { api: "single", ssr: true, files: ["src/components/Item0.vue"] },
    ]);
    const ssr = Object.values(shared.outputs.ssr).join("\n");
    assert.equal(ssr.includes('"rewritten for ssr"'), true);
    assert.equal(Object.values(shared.outputs.client).join("\n").includes("rewritten"), false);
    for (const environment of Object.keys(app)) {
      assert.deepEqual(shared.outputs[environment], apart.outputs[environment], environment);
    }
  } finally {
    built.cleanup();
  }
});

void test("a component that does not compile fails the build where it failed before", async () => {
  const broken = "src/components/Broken.vue";
  const built = project({ [broken]: `<template><div id="a" id="b">broken</div></template>\n` });
  try {
    const shared = await buildApp(built.root, app);
    const apart = buildAppWithoutTargets(built.root, app);
    assert.equal(shared.error?.environment, "client");
    assert.equal(shared.error?.message.includes("Broken.vue"), true, shared.error?.message);
    assert.deepEqual(shared.error, apart.error);
  } finally {
    built.cleanup();
  }
});
