/** Identical, deterministic full-corpus inputs for both measured CLI builds. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, sep } from "node:path";
import { CORPUS_TSCONFIG, prepareCorpus } from "./check-gate-env.mjs";
import { generateCorpus } from "./generate.mjs";
import { prepareSharedLeafCorpus } from "./retained-template-reads-cli-leaf-corpus.mjs";
import { diagnosticFingerprint } from "./typecheck-command.mjs";
import { FILE_COUNT, compareStrings, writeJson } from "./retained-template-reads-cli-protocol.mjs";

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
    prepareSharedLeafCorpus(workRoot, vuePackageDir),
    {
      id: "generated500",
      dir: generated.dir,
      tsconfig: CORPUS_TSCONFIG,
      expectedVueFiles: FILE_COUNT,
      args: ["."],
    },
  ];
  for (const corpus of corpora) {
    const manifest = corpusManifest(corpus.dir);
    assert.equal(manifest.fileCount, corpus.expectedVueFiles);
    const inputDir = join(directory, "inputs", corpus.id);
    mkdirSync(inputDir);
    for (const file of manifest.files) {
      mkdirSync(dirname(join(inputDir, file.file)), { recursive: true });
      copyFileSync(join(corpus.dir, file.file), join(inputDir, file.file));
    }
    writeJson(join(directory, "inputs", `${corpus.id}.manifest.json`), manifest);
    corpus.manifest = manifest;
    if (corpus.id === "shared-leaf501-default") corpus.expectedRetainedReads = 40_000;
  }
  return corpora;
}

export function selfTestCorpus() {
  const root = mkdtempSync(join(os.tmpdir(), "retained-template-reads-inputs-"));
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
    assert.equal(preparations[0].length, 2);
    for (let index = 0; index < 2; index++) {
      const corpus = preparations[0][index];
      assert.equal(corpus.manifest.fileCount, corpus.expectedVueFiles);
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
        join(corpus.dir, corpus.manifest.files.find((file) => file.file.endsWith(".vue")).file),
        "<template>changed</template>\n",
      );
      assert.notEqual(corpusManifest(corpus.dir).sha256, corpus.manifest.sha256);
    }
    console.log("retained-template-reads CLI corpus checks passed");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
