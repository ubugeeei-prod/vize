import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  type CommandResult,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";
import { testOutputRoot } from "../../tooling/support/lsp/paths.ts";

// Template ref unwrapping is provenance-independent (#4146): the same authored
// template must type identically whether a composable is imported by the SFC or
// injected by a framework auto-import, because the auto-import transform turns
// the name into a `<script setup>` import before Vue ever sees it.
//
// `vue-tsc` is the oracle for the explicitly-imported workspace. It cannot be
// the oracle for the auto-imported one: it types every template identifier
// against the component instance and reports `Property 'x' does not exist` for
// framework globals (issue #913 owns that divergence). The auto-imported
// workspace is therefore pinned to the vue-tsc-checked explicit workspace,
// diagnostic for diagnostic.

const composables = `import {
  computed,
  readonly,
  ref,
  shallowRef,
  type ComputedRef,
  type Ref,
  type ShallowRef,
  type WritableComputedRef,
} from 'vue'

export interface Media { id: string; url: string }
export interface UserLogin { account: { id: string }; server: string; token?: string }

export const mediaList: Ref<Media[]> = ref([])
export const currentUser: ComputedRef<UserLogin | undefined> = computed(() => undefined)
export const draftName: WritableComputedRef<string> = computed({ get: () => 'draft', set: () => {} })
export const shallowList: ShallowRef<Media[]> = shallowRef([])
export const readonlyCount = readonly(ref(0))
export const nullableUser: Ref<UserLogin | null> = ref(null)
export const unionValue: Ref<string | number> = ref('a')
export const flag = ref(false)

// Not a ref: a plain constant that merely has a \`value\` property (#3767).
export const OPTION = { text: 'Login info', value: 'LOGIN_INFO' } as const
`;

const child = `<script setup lang="ts">
import type { Media } from './composables'

defineProps<{ items: Media[]; label: string; count: number }>()
</script>

<template>
  <div />
</template>
`;

// Line 3 is the only difference between the two provenances and both spellings
// are exactly one line, so every authored line and column stays identical.
const explicitLine =
  "import { OPTION, currentUser, draftName, flag, mediaList, nullableUser, readonlyCount, shallowList, unionValue } from './composables'";
const autoLine =
  "// Auto-imported: OPTION currentUser draftName flag mediaList nullableUser readonlyCount shallowList unionValue";

const autoImportNames = [
  "OPTION",
  "currentUser",
  "draftName",
  "flag",
  "mediaList",
  "nullableUser",
  "readonlyCount",
  "shallowList",
  "unionValue",
];

const cleanTemplate = `  <div>{{ mediaList.length }}</div>
  <div>{{ mediaList[0]?.url }}</div>
  <div>{{ currentUser?.account.id }}</div>
  <div>{{ currentUser?.token }}</div>
  <div v-if="draftName === 'draft'">named</div>
  <div>{{ shallowList.length }}</div>
  <div>{{ readonlyCount + 1 }}</div>
  <div>{{ nullableUser?.server }}</div>
  <div>{{ unionValue }}</div>
  <div>{{ OPTION.value }}</div>
  <div>{{ scriptOnlyCount }}</div>
  <div v-if="flag">on</div>
  <div v-for="item in mediaList" :key="item.id">{{ item.url }}</div>
  <input v-model="draftName">
  <button @click="flag = !flag">toggle</button>
  <Child :items="mediaList" :label="draftName" :count="readonlyCount" />
`;

const scriptLine = "const scriptOnlyCount = mediaList.value.length";

type Variant = {
  expected: string[];
  name: string;
  script: string;
  template: string;
};

const variants: Variant[] = [
  { expected: [], name: "clean", script: scriptLine, template: cleanTemplate },
  {
    // The unwrapped binding is a `Media[]`, so the misspelling reports against
    // the element type. Before #4146 the auto-imported side reported TS2339 on
    // `Ref<Media[], Media[]>` instead.
    expected: [
      "src/App.vue(8,21): error TS2551: Property 'lenght' does not exist on type 'Media[]'. Did you mean 'length'?",
    ],
    name: "misspelled member on an unwrapped array",
    script: scriptLine,
    template: cleanTemplate.replace("mediaList.length", "mediaList.lenght"),
  },
  {
    // Unwrapping must not hide `.value` misuse inside a template.
    expected: [
      "src/App.vue(8,21): error TS2551: Property 'value' does not exist on type 'Media[]'. Did you mean 'values'?",
    ],
    name: "explicit .value in a template",
    script: scriptLine,
    template: cleanTemplate.replace("mediaList.length", "mediaList.value"),
  },
  {
    // Negative control: a plain object that merely has a `value` property is
    // not a ref and keeps its own shape.
    expected: [
      `src/App.vue(17,18): error TS2551: Property 'valeu' does not exist on type '{ readonly text: "Login info"; readonly value: "LOGIN_INFO"; }'. Did you mean 'value'?`,
    ],
    name: "plain { value } constant",
    script: scriptLine,
    template: cleanTemplate.replace("OPTION.value", "OPTION.valeu"),
  },
  {
    // Negative control: script code is never unwrapped.
    expected: [
      "src/App.vue(4,35): error TS2339: Property 'length' does not exist on type 'Ref<Media[], Media[]>'.",
    ],
    name: "script-only read without .value",
    script: "const scriptOnlyCount = mediaList.length",
    template: cleanTemplate,
  },
  { expected: [], name: "repaired", script: scriptLine, template: cleanTemplate },
];

test("template ref unwrapping matches vue-tsc and is provenance-independent", () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();
  const explicitDir = createWorkspace("explicit", false);
  const autoDir = createWorkspace("auto", true);

  for (const variant of variants) {
    writeApp(explicitDir, explicitLine, variant);
    writeApp(autoDir, autoLine, variant);

    const baseline = normalizeVueTsc(runVueTsc(explicitDir, vueTscPath));
    assert.deepEqual(baseline, variant.expected, `vue-tsc baseline drifted for "${variant.name}"`);

    const explicit = normalizeVize(runVizeCheck(explicitDir, corsaPath, ["src"]));
    assert.deepEqual(
      explicit,
      baseline,
      `vize must match vue-tsc exactly for "${variant.name}" (explicit import)`,
    );

    const auto = normalizeVize(runVizeCheck(autoDir, corsaPath, ["src"]));
    assert.deepEqual(
      auto,
      baseline,
      `an auto-imported composable must type like an authored import for "${variant.name}"`,
    );
  }
});

function createWorkspace(name: string, autoImported: boolean): string {
  const dir = path.join(testOutputRoot, `template-ref-unwrap-${name}-${process.pid}`);
  fs.rmSync(dir, { force: true, recursive: true });
  fs.mkdirSync(path.join(dir, "src"), { recursive: true });
  symlinkVueTypes(dir);
  write(dir, "src/composables.ts", composables);
  write(dir, "src/Child.vue", child);
  if (autoImported) {
    fs.mkdirSync(path.join(dir, ".nuxt/types"), { recursive: true });
    write(dir, "nuxt.config.ts", "export default {}\n");
    write(
      dir,
      "tsconfig.json",
      `${JSON.stringify({ extends: "./.nuxt/tsconfig.json" }, null, 2)}\n`,
    );
    write(dir, ".nuxt/tsconfig.json", `${JSON.stringify(nuxtTsconfig, null, 2)}\n`);
    write(dir, ".nuxt/types/imports.d.ts", generatedImports);
  } else {
    write(dir, "tsconfig.json", `${JSON.stringify(tsconfig, null, 2)}\n`);
  }
  return dir;
}

function writeApp(dir: string, provenanceLine: string, variant: Variant): void {
  write(
    dir,
    "src/App.vue",
    `<script setup lang="ts">\nimport Child from './Child.vue'\n${provenanceLine}\n${variant.script}\n</script>\n\n<template>\n${variant.template}</template>\n`,
  );
}

function write(dir: string, file: string, content: string): void {
  const target = path.join(dir, file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, content, "utf8");
}

/** `src/App.vue(8,21): error TS2551: ...`, sorted, App.vue only. */
function normalizeVueTsc(result: CommandResult): string[] {
  return result.stdout
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("src/App.vue("))
    .sort();
}

/** vize's JSON report, rewritten into the vue-tsc line shape. */
function normalizeVize(result: VizeCheckResult): string[] {
  const lines: string[] = [];
  for (const file of result.report.files) {
    if (!file.file.endsWith("App.vue")) continue;
    for (const diagnostic of file.diagnostics) {
      const match = /^(\w+):(\d+):(\d+) \[TS(\d+)] ([\s\S]*)$/.exec(diagnostic.trim());
      assert.ok(match, `unrecognized vize diagnostic: ${diagnostic}`);
      const [, severity, line, column, code, message] = match;
      lines.push(`src/App.vue(${line},${column}): ${severity} TS${code}: ${message}`);
    }
  }
  return lines.sort();
}

const tsconfig = {
  compilerOptions: {
    lib: ["ES2022", "DOM", "DOM.Iterable"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
  },
  include: ["src/**/*.ts", "src/**/*.vue"],
};

const nuxtTsconfig = {
  compilerOptions: tsconfig.compilerOptions,
  include: ["../src/**/*.ts", "../src/**/*.vue", "./types/imports.d.ts"],
};

const generatedImports = `// Generated by nuxt
export {}
declare global {
${autoImportNames
  .map((name) => `  const ${name}: typeof import('../../src/composables')['${name}']`)
  .join("\n")}
}
`;
