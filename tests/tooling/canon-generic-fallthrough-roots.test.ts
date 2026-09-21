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

// A generic component forwards what its root is instantiated to: the root's
// props and listeners are read in terms of the component's own type
// parameters, so a parent's listener sees the type its own props chose.

const root = `<script setup lang="ts" generic="T">
defineProps<{ foo?: T; label: string }>();
defineEmits<{ foo: [value: T]; close: [] }>();
</script>
<template><span /></template>`;

const forwarding = `<script setup lang="ts" generic="T">
import Root from './Root.vue';
defineProps<{ bar?: T }>();
</script>
<template><Root :foo="bar" label="root" /></template>`;

function project(files: Record<string, string>, options: Record<string, boolean> = {}): string {
  const directory = workspace("generic-fallthrough-roots-");
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
      vueCompilerOptions: { strictTemplates: true, fallthroughAttributes: true, ...options },
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

const parent = (usage: string) => `<script setup lang="ts">
import Forwarding from './Forwarding.vue';
const takesString = (value: string) => value;
const takesNumber = (value: number) => value;
</script>
<template>${usage}</template>`;

test("a generic component forwards its root's listeners at its own instantiation", async () => {
  const directory = project({
    "Root.vue": root,
    "Forwarding.vue": forwarding,
    "App.vue": parent(`
  <Forwarding bar="text" @foo="(value) => takesString(value)" @close="() => {}" />
  <Forwarding :bar="1" @foo="(value) => takesNumber(value)" :foo="2" />`),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" @foo="(value) => takesNumber(value)" />
  <Forwarding :bar="1" @missing="() => {}" />
  <Forwarding :bar="1" foo="not a number" />`),
    );
    await compare(directory, ["App.vue:7:55 TS2345", "App.vue:8:25 TS2353", "App.vue:9:24 TS2322"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("forwarded props stay next to the slots a generic component infers", async () => {
  const directory = project({
    "Root.vue": root,
    "Forwarding.vue": forwarding.replace(
      '<Root :foo="bar" label="root" />',
      '<Root :foo="bar" label="root"><slot name="item" :item="bar" /></Root>',
    ),
    "App.vue": parent(`
  <Forwarding bar="text" @foo="(value) => takesString(value)">
    <template #item="{ item }">{{ takesString(item!) }}</template>
  </Forwarding>`),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" @foo="(value) => takesNumber(value)">
    <template #item="{ item }">{{ takesNumber(item!) }}</template>
  </Forwarding>`),
    );
    await compare(directory, ["App.vue:7:55 TS2345", "App.vue:8:47 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("a generic component forwards to every root of a conditional chain", async () => {
  const directory = project({
    "Root.vue": root,
    "Other.vue": `<script setup lang="ts" generic="T">
defineProps<{ other?: T }>();
defineEmits<{ other: [value: T[]] }>();
</script>
<template><span /></template>`,
    "Forwarding.vue": `<script setup lang="ts" generic="T">
import Root from './Root.vue';
import Other from './Other.vue';
defineProps<{ bar?: T; flip?: boolean }>();
</script>
<template><Root v-if="flip" :foo="bar" label="root" /><Other v-else :other="bar" /></template>`,
    "App.vue": parent(`
  <Forwarding bar="text" @foo="(value) => takesString(value)" @other="(values) => takesString(values[0]!)" />`),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" @other="(values) => takesNumber(values[0]!)" />`),
    );
    await compare(directory, ["App.vue:7:58 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("a generic component asks its parent for the required props its root leaves unbound", async () => {
  const directory = project(
    {
      "Root.vue": `<script setup lang="ts">
defineProps<{ foo?: string; label: string }>();
defineEmits<{ close: [reason: string] }>();
</script>
<template><span /></template>`,
      "Forwarding.vue": `<script setup lang="ts" generic="T extends string">
import Root from './Root.vue';
defineProps<{ bar?: T }>();
</script>
<template><Root :foo="bar" /></template>`,
      "App.vue": parent(`
  <Forwarding bar="text" label="from the parent" @close="(reason) => takesString(reason)" />`),
    },
    { checkRequiredFallthroughAttributes: true },
  );
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" />
  <Forwarding bar="text" label="from the parent" @close="(reason) => takesNumber(reason)" />`),
    );
    await compare(directory, ["App.vue:7:4 TS2345", "App.vue:8:82 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
