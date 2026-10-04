/** Identical, deterministic full-corpus inputs for both measured CLI builds. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, sep } from "node:path";
import { CORPUS_TSCONFIG, prepareCorpus } from "./check-gate-env.mjs";
import { generateCorpus } from "./generate.mjs";
import { diagnosticFingerprint } from "./typecheck-command.mjs";
import { prepareSharedLeafCorpus } from "./type-snapshot-cli-leaf-corpus.mjs";
import { FILE_COUNT, compareStrings, writeJson } from "./type-snapshot-cli-protocol.mjs";

function sourceFiles(dir, prefix = "") {
  return readdirSync(join(dir, prefix), { withFileTypes: true })
    .flatMap((entry) => {
      if (entry.name === "node_modules") return [];
      const file = join(prefix, entry.name);
      return entry.isDirectory() ? sourceFiles(dir, file) : [file];
    })
    .sort(compareStrings);
}

export function corpusManifest(dir) {
  const files = sourceFiles(dir).map((file) => {
    const bytes = readFileSync(join(dir, file));
    return {
      file: file.split(sep).join("/"),
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    };
  });
  return {
    fileCount: files.filter((file) => file.file.endsWith(".vue")).length,
    totalBytes: files.reduce((sum, file) => sum + file.bytes, 0),
    sha256: diagnosticFingerprint(files),
    files,
  };
}

function sharedBarrelCorpus(workRoot, vuePackageDir) {
  const dir = join(workRoot, "shared-barrel500");
  mkdirSync(join(dir, "shared"), { recursive: true });
  writeFileSync(
    join(dir, "shared/model.ts"),
    'export interface Item { id: number; label: string; active: boolean }\nexport type Variant = "primary" | "secondary";\n',
  );
  writeFileSync(
    join(dir, "shared/panel.ts"),
    'import type { Item, Variant } from "./model";\nexport interface PanelProps { title: string; disabled?: boolean; variant?: Variant; items: Item[] }\n',
  );
  const barrel = [
    'export type { Item, Variant } from "./model";',
    'export type { PanelProps } from "./panel";',
  ];
  for (let index = 0; index < 32; index++) {
    writeFileSync(
      join(dir, `shared/domain-${index}.ts`),
      `import type { Item } from "./model";\nexport interface Domain${index} { id: string; owner?: Item; flags: { enabled: boolean; rank: number } }\n`,
    );
    barrel.push(`export type { Domain${index} } from "./domain-${index}";`);
  }
  writeFileSync(join(dir, "shared/index.ts"), `${barrel.join("\n")}\n`);
  for (let index = 0; index < FILE_COUNT; index++) {
    const domain = index % 32;
    writeFileSync(
      join(dir, `Component${String(index).padStart(4, "0")}.vue`),
      `<script setup lang="ts">
import type { PanelProps, Domain${domain} } from "./shared";
const props = defineProps<PanelProps>();
const local: Domain${domain} = { id: "component-${index}", flags: { enabled: true, rank: ${index} } };
const label = local.id;
const disabledFlag: boolean = props.disabled ?? false;
</script>

<template>
  <section :data-component="${index}">
    <h2>{{ title }}</h2>
    <button type="button" :disabled="disabledFlag">{{ variant }}</button>
    <ul><li v-for="item in items" :key="item.id">{{ item.label }}</li></ul>
    <small>{{ label }}</small>
  </section>
</template>
`,
    );
  }
  const tsconfig = { ...CORPUS_TSCONFIG, include: ["*.vue", "shared/**/*.ts"] };
  writeJson(join(dir, "tsconfig.json"), tsconfig);
  writeJson(join(dir, "package.json"), {
    name: "type-snapshot-shared-barrel500",
    private: true,
    type: "module",
  });
  mkdirSync(join(dir, "node_modules"));
  symlinkSync(vuePackageDir, join(dir, "node_modules/vue"), "dir");
  const namespace = join(dirname(vuePackageDir), "@vue");
  if (existsSync(namespace)) symlinkSync(namespace, join(dir, "node_modules/@vue"), "dir");
  return { id: "shared-barrel500", dir, tsconfig };
}

export function prepareCliCorpora(directory, vuePackageDir) {
  const workRoot = join(directory, "work");
  const generatedInput = join(workRoot, "generated-input");
  generateCorpus({ fileCount: FILE_COUNT, benchDir: generatedInput, log: null });
  const generated = prepareCorpus(
    generatedInput,
    FILE_COUNT,
    join(workRoot, "generated"),
    vuePackageDir,
  );
  const corpora = [
    sharedBarrelCorpus(workRoot, vuePackageDir),
    prepareSharedLeafCorpus(workRoot, vuePackageDir),
    { id: "generated500", dir: generated.dir, tsconfig: CORPUS_TSCONFIG },
  ];
  for (const corpus of corpora) {
    const manifest = corpusManifest(corpus.dir);
    assert.equal(manifest.fileCount, corpus.expectedVueFiles ?? FILE_COUNT);
    const inputDir = join(directory, "inputs", corpus.id);
    mkdirSync(inputDir);
    for (const file of manifest.files) {
      mkdirSync(dirname(join(inputDir, file.file)), { recursive: true });
      copyFileSync(join(corpus.dir, file.file), join(inputDir, file.file));
    }
    writeJson(join(directory, "inputs", `${corpus.id}.manifest.json`), manifest);
    corpus.manifest = manifest;
  }
  return corpora;
}

export function selfTestCorpus() {
  const root = mkdtempSync(join(os.tmpdir(), "type-snapshot-inputs-"));
  try {
    const vue = join(root, "vue");
    mkdirSync(vue);
    writeJson(join(vue, "package.json"), { name: "vue", version: "test" });
    const preparations = [];
    for (const id of ["first", "second"]) {
      const directory = join(root, id);
      for (const subdir of ["inputs", "work"])
        mkdirSync(join(directory, subdir), { recursive: true });
      preparations.push(prepareCliCorpora(directory, vue));
    }
    assert.equal(preparations[0].length, 3);
    for (let index = 0; index < 3; index++) {
      const corpus = preparations[0][index];
      assert.equal(corpus.manifest.fileCount, corpus.expectedVueFiles ?? FILE_COUNT);
      assert.equal(
        corpus.manifest.sha256,
        preparations[1][index].manifest.sha256,
        "generation must be deterministic across project roots",
      );
      assert.equal(
        corpusManifest(join(root, "first", "inputs", corpus.id)).sha256,
        corpus.manifest.sha256,
        "published inputs must equal timed inputs",
      );
      writeFileSync(join(corpus.dir, "node_modules", "ignored-cache"), "not a source input");
      assert.equal(corpusManifest(corpus.dir).sha256, corpus.manifest.sha256);
      writeFileSync(
        join(
          corpus.dir,
          corpus.id === "shared-leaf501-default" ? "Comp0.vue" : "Component0000.vue",
        ),
        "<template>changed</template>\n",
      );
      assert.notEqual(corpusManifest(corpus.dir).sha256, corpus.manifest.sha256);
    }
    console.log("type-snapshot CLI corpus checks passed");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
