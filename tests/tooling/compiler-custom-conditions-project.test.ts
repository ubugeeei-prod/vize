import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  loadCompilerProjectInputs,
  materializeCompilerProjectInputs,
} from "../differential/compiler-project.ts";

const base = new URL("../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(fs.readFileSync(new URL("manifest.json", base), "utf8"));
const row = manifest.cases.find(
  (row: { id: string }) => row.id === "compiler/project/macro-export-conditions-11918",
);
const read = (file: string) => fs.readFileSync(new URL(file, base));

test("the conditions project authenticates and materializes the complete unbuilt package", () => {
  const files = loadCompilerProjectInputs(row, read);
  assert.deepEqual(
    files.map((file) => file.path),
    [
      "Layout.vue",
      "tsconfig.json",
      "node_modules/lib/package.json",
      "node_modules/lib/src/index.ts",
      "node_modules/lib/dist/index.js",
    ],
  );
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-export-conditions-"));
  try {
    materializeCompilerProjectInputs(directory, files);
    for (const file of files)
      assert.deepEqual(fs.readFileSync(path.join(directory, file.path)), file.bytes);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(directory, "tsconfig.json"), "utf8")).compilerOptions
        .customConditions,
      ["source"],
    );
    assert.equal(fs.existsSync(path.join(directory, "node_modules/lib/dist/index.d.ts")), false);
    assert.throws(() => materializeCompilerProjectInputs(directory, files.slice(0, 4)));
    assert.throws(() => materializeCompilerProjectInputs(directory, [...files].reverse()));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("every source-condition package input keeps its pinned bytes and finite role", () => {
  for (const [index, file] of row.inputs.files.entries()) {
    assert.throws(() =>
      loadCompilerProjectInputs(row, (name) =>
        name.endsWith(file.path) ? Buffer.from("changed") : read(name),
      ),
    );
    const invalid = structuredClone(row);
    invalid.inputs.files[index].role = "unknown";
    assert.throws(() => loadCompilerProjectInputs(invalid, read));
  }
});
