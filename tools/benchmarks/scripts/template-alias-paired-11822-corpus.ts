import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

export interface AuthoredError {
  file: string;
  line: number;
  column: number;
  code: number;
}
export interface CorpusManifest {
  schema: "vize.public-mapper-corpus.v1";
  components: number;
  source: "newly-authored-typed-props-model-listeners";
  expectedErrors: AuthoredError[];
  files: { path: string; bytes: number; sha256: string }[];
}

const leaf = `<script setup lang="ts">
const props = defineProps<{ modelValue: number; payload: { id: string; enabled: boolean } }>();
const emit = defineEmits<{ 'update:modelValue': [value: number]; change: [value: number, label: string] }>();
function select(): void {
  emit('update:modelValue', props.modelValue + 1);
  emit('change', props.modelValue, props.payload.id);
}
</script>
<template><button @click="select">{{ props.payload.id }}</button></template>
`;

function parent(index: number, invalid: boolean): string {
  return `<script setup lang="ts">
import { ref } from 'vue';
import Leaf from '../Leaf.vue';
const state = ref(1);
const payload = { id: 'case-${index}', enabled: true };
const oracle: number = ${invalid ? "'wrong'" : "1"};
void oracle;
function onChange(value: number, label: string): void { state.value = value; void label; }
</script>
<template><Leaf v-model="state" :payload="payload" @change="onChange" /></template>
`;
}

/** A public workload authored here; no extracted or external project input. */
export function corpusFiles(components = 2500, errors = 12) {
  assert.ok(Number.isSafeInteger(components) && components >= 2 && components <= 5000);
  assert.ok(Number.isSafeInteger(errors) && errors >= 1 && errors < components);
  const parents = components - 1;
  const invalid = new Set(
    Array.from({ length: errors }, (_, index) =>
      errors === 1 ? 0 : Math.floor((index * (parents - 1)) / (errors - 1)),
    ),
  );
  assert.equal(invalid.size, errors);
  const files = new Map<string, string>([["src/Leaf.vue", leaf]]);
  const expectedErrors: AuthoredError[] = [];
  const imports: string[] = [];
  const entries: string[] = [];
  for (let index = 0; index < parents; index++) {
    const name = "GeneratedComponent" + String(index).padStart(4, "0");
    const path = `src/generated/${name}.vue`;
    const text = parent(index, invalid.has(index));
    files.set(path, text);
    imports.push(`import ${name} from './generated/${name}.vue';`);
    entries.push(name);
    if (invalid.has(index)) {
      const line = text.split("\n").findIndex((row) => row.startsWith("const oracle:")) + 1;
      assert.ok(line > 0);
      expectedErrors.push({ file: path, line, column: 7, code: 2322 });
    }
  }
  files.set(
    "src/index.ts",
    imports.join("\n") +
      "\nexport const cases: readonly unknown[] = [" +
      entries.join(",") +
      "];\n",
  );
  files.set(
    "package.json",
    JSON.stringify({ name: "vize-public-mapper-corpus", private: true, type: "module" }, null, 2) +
      "\n",
  );
  files.set(
    "tsconfig.base.json",
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          noEmit: true,
          target: "ES2024",
          module: "ESNext",
          moduleResolution: "Bundler",
          jsx: "preserve",
          types: [],
          allowImportingTsExtensions: true,
          skipLibCheck: false,
          incremental: false,
        },
        include: ["src/**/*.ts", "src/**/*.vue"],
      },
      null,
      2,
    ) + "\n",
  );
  files.set(
    "tsconfig.vize-content-mapper.json",
    JSON.stringify(
      {
        extends: "./tsconfig.base.json",
        contentMappers: [{ package: "vize", extensions: [".vue"] }],
      },
      null,
      2,
    ) + "\n",
  );
  return { files, expectedErrors };
}

export function materializeCorpus(root: string, components = 2500, errors = 12): CorpusManifest {
  const plan = corpusFiles(components, errors);
  mkdirSync(root);
  const manifest: CorpusManifest = {
    schema: "vize.public-mapper-corpus.v1",
    components,
    source: "newly-authored-typed-props-model-listeners",
    expectedErrors: plan.expectedErrors,
    files: [],
  };
  for (const [path, text] of [...plan.files].sort(([left], [right]) => left.localeCompare(right))) {
    const target = join(root, path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, text, { flag: "wx" });
    manifest.files.push({
      path,
      bytes: Buffer.byteLength(text),
      sha256: createHash("sha256").update(text).digest("hex"),
    });
  }
  writeFileSync(join(root, "corpus.json"), JSON.stringify(manifest, null, 2) + "\n", {
    flag: "wx",
  });
  return manifest;
}
