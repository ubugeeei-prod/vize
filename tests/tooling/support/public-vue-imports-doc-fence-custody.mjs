import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { currentConfigExample } from "./docs-config-successor.ts";
import { sha256 } from "../lint-default-correctness-reference.mjs";

export const codeFences = (source) =>
  [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map((match) => match[0]);

export function validateFenceSuccessor(successor) {
  assert.deepEqual(
    Object.keys(successor).sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
    [
      "afterCodeFences",
      "authority",
      "beforeCodeFences",
      "change",
      "oldSources",
      "originalCustody",
      "originalEntry",
      "schema",
      "sourceLaw",
      "version",
    ],
  );
  assert.equal(successor.schema, "vize.docs.public-vue-imports-fence-successor");
  assert.equal(successor.version, 1);
  assert.deepEqual(successor.originalCustody, {
    path: "tests/_fixtures/docs-ja-rule-original-custody.json",
    sha256: "165c6b367ca2928177f379ebbb6c8d71a7e454d585f45f826f5a5168dcc8dad3",
    entries: 317,
  });
  assert.deepEqual(successor.sourceLaw, {
    producerPath: "crates/vize_patina/src/rules/script/prefer_import_from_vue.rs",
    producerSha256: "79d3814ecd8822cf81ad475ecfc850897ee907b587e2312749edb8b0e8795abb",
    corpusPath: "tests/_fixtures/differential/lint/public-vue-imports-8090/cases.json",
    corpusSha256: "d52480f474c2a6dba9681aa620a0c5e6dd2597accd91ad286000230a1b9d2389",
    tablePath: "crates/vize_patina/src/rules/script/prefer_import_from_vue/public_exports.rs",
    tableSha256: "8d02fc6a9e3f6fc90cf9dac3948cbd7569403ba366e84757157f1c9bc546c78c",
  });
  assert.equal(successor.originalEntry.id, "script/prefer-import-from-vue");
  assert.equal(successor.beforeCodeFences.length, 4);
  assert.equal(successor.oldSources.length, 2);
  assert.deepEqual(
    successor.oldSources.map(({ source: _source, ...identity }) => identity),
    [
      {
        path: "docs/content/ja/rules/reference/script-prefer-import-from-vue.md",
        revision: "4ae56daecf4cc31d2d2558b92451b07d448bc259",
        gitBlob: "4a3764848b8c1c3b590de5119966ee1d42bc1785",
        sha256: "9380b281e11b9bf852387f0c1281806b989647555a9eedbbad9c52d44933d2d1",
      },
      {
        path: "docs/content/rules/reference/script-prefer-import-from-vue.md",
        revision: "4ae56daecf4cc31d2d2558b92451b07d448bc259",
        gitBlob: "bc61f51745b1c0145a56c63469492dda5b051cb3",
        sha256: "02f667617b4e8247c47fe57e679af0a5ed8702eac48e1e9362f7982692533ada",
      },
    ],
  );
  const wanted = [successor.originalEntry.path, successor.originalEntry.englishPath];
  assert.deepEqual(
    successor.oldSources.map((source) => source.path),
    wanted,
  );
  for (const source of successor.oldSources) {
    assert.equal(source.revision, "4ae56daecf4cc31d2d2558b92451b07d448bc259");
    assert.equal(sha256(source.source), source.sha256);
    assert.equal(
      createHash("sha1")
        .update(`blob ${Buffer.byteLength(source.source)}\0`)
        .update(source.source)
        .digest("hex"),
      source.gitBlob,
    );
    assert.deepEqual(codeFences(source.source), successor.beforeCodeFences);
  }
  assert.equal(
    sha256(successor.beforeCodeFences.join("\0")),
    successor.originalEntry.wholeCodeSha256,
  );
  assert.deepEqual(successor.change, {
    anchor: "import { ref, h } from 'vue'",
    added: "import { isString } from '@vue/shared'",
    changedFenceIndex: 3,
    changedFences: 1,
  });
  const { anchor, added } = successor.change;
  assert.equal(successor.beforeCodeFences.filter((fence) => fence.includes(anchor)).length, 1);
  assert.ok(successor.beforeCodeFences[3].includes(anchor));
  const expected = successor.beforeCodeFences.map((fence) =>
    fence.replace(anchor + "\n", anchor + "\n" + added + "\n"),
  );
  assert.deepEqual(successor.afterCodeFences, expected);
}

export function createFenceVerifier(root) {
  const fixture = path.join(
    root,
    "tests/_fixtures/differential/lint/public-vue-imports-8090/docs-fence-successor.json",
  );
  const successor = JSON.parse(fs.readFileSync(fixture, "utf8"));
  validateFenceSuccessor(successor);
  const original = fs.readFileSync(path.join(root, successor.originalCustody.path));
  assert.equal(sha256(original), successor.originalCustody.sha256);
  const entries = JSON.parse(original).entries;
  assert.equal(entries.length, 317);
  assert.deepEqual(
    entries.find((entry) => entry.id === successor.originalEntry.id),
    successor.originalEntry,
  );
  // The source-law files are live code; require them without pinning their bytes.
  for (const kind of ["producer", "corpus", "table"])
    assert(fs.existsSync(path.join(root, successor.sourceLaw[kind + "Path"])), kind);
  return (entry, filename, fences) => {
    if (entry.id !== successor.originalEntry.id) return false;
    assert.deepEqual(entry, successor.originalEntry);
    assert.ok([entry.path, entry.englishPath].includes(filename));
    assert.deepEqual(
      fences,
      successor.afterCodeFences.map(currentConfigExample),
      filename + ": complete source-authored private example successor",
    );
    return true;
  };
}
