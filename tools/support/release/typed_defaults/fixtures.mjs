import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { apiRoot, gitBlob, sha256 } from "./custody.mjs";
import { returnDirectory, returnObserver } from "./return_prefilter.mjs";

export const typedDirectory = "tests/_fixtures/differential/lint/typed-prop-default-prefilter-8564";
export const originals = {
  [`${returnDirectory}/cases.json`]:
    "52d3515ca7673895b0817c92ffc72f1136de43ae908dbe140906cd1c9c71d16b",
  "tests/tooling/lint-return-prefilter.test.mjs": returnObserver,
  [`${typedDirectory}/cases.json`]:
    "b9e78a4082b0e2250afc8b652d0dabdcdb50bc506ec63ae2d15df236607c632d",
  "tests/_fixtures/lint-default-correctness/cases.json":
    "7fa33faea83c868ad5bb01cacd624240486ecea35177a637f09e56ec5f73e9fc",
  "tests/_fixtures/lint-default-migrations/cases.json":
    "30f05bb221d3e33149b40046df704906bfb8bdbe0107a0d043af1f32965eb0bd",
  "tests/_fixtures/n8n-cli-adoption.json":
    "8b0c91b481d28f8477f4af222d98b5b4a73ef5c92baaa86bae34fe4cb4b0b5d1",
  "tests/tooling/lint-default-correctness-reference.mjs":
    "d0fe46840f15d3aee0cad8b0394b4c67dd1f738a9ddaba39f55073fefd9df6b7",
};

// Immutable authored observer roles retain their original path and digest.
// Active test runners evolve; these non-executable archives preserve the source
// bytes used to define the original release acceptance vectors.
export const originalSourcePaths = {
  "tests/tooling/lint-return-prefilter.test.mjs":
    "tests/_fixtures/differential/lint/return-prefilter-8090/original-return-observer.mjs.txt",
};

export async function loadFixtures(head, destination, json, treeSha) {
  assert.match(treeSha ?? "", /^[0-9a-f]{40}$/);
  const tree = await json(`${apiRoot}/git/trees/${treeSha}?recursive=1`);
  assert.equal(tree.sha, treeSha, "original tree must belong to authenticated H");
  assert.equal(tree.truncated, false, "complete immutable source tree required");
  const files = new Map(tree.tree.map((entry) => [entry.path, entry]));
  const authorities = [];
  async function read(name, expectedHash) {
    const sourcePath = originalSourcePaths[name] ?? name;
    const entry = files.get(sourcePath);
    assert(
      entry && entry.type === "blob" && ["100644", "100755"].includes(entry.mode),
      "original source must be regular",
    );
    const blob = await json(`${apiRoot}/git/blobs/${entry.sha}`);
    assert.equal(blob.encoding, "base64");
    const bytes = Buffer.from(blob.content, "base64");
    assert(bytes.length <= 256 * 1024, "bounded authored fixture required");
    assert.equal(blob.sha, entry.sha);
    assert.equal(blob.size, bytes.length);
    assert.equal(gitBlob(bytes), entry.sha);
    assert.equal(sha256(bytes), expectedHash, `original authored bytes changed: ${name}`);
    const file = path.join(destination, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes, { flag: "wx" });
    authorities.push({
      path: name,
      ...(sourcePath === name ? {} : { sourcePath }),
      source: head,
      blob: entry.sha,
      bytes: bytes.length,
      sha256: expectedHash,
    });
    return bytes;
  }
  const source = new Map();
  for (const [name, hash] of Object.entries(originals)) source.set(name, await read(name, hash));
  const typed = JSON.parse(source.get(`${typedDirectory}/cases.json`).toString("utf8"));
  const returns = JSON.parse(source.get(`${returnDirectory}/cases.json`).toString("utf8"));
  const first = JSON.parse(
    source.get("tests/_fixtures/lint-default-correctness/cases.json").toString("utf8"),
  );
  const second = JSON.parse(
    source.get("tests/_fixtures/lint-default-migrations/cases.json").toString("utf8"),
  );
  const projection = JSON.parse(
    source.get("tests/_fixtures/n8n-cli-adoption.json").toString("utf8"),
  );
  assert.equal(typed.cases.length, 8);
  assert.equal(returns.cases.length, 16);
  assert.deepEqual(returns.rules, [
    "script/return-in-computed-property",
    "script/return-in-emits-validator",
  ]);
  assert.deepEqual(
    typed.cases.filter((entry) => entry.diagnostics.length).map((entry) => entry.id),
    ["original-vaul", "with-defaults", "destructure-alias", "unicode-crlf"],
  );
  assert.equal(first.cases.length, 15);
  assert.equal(second.cases.length, 12);
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[typed.rule], undefined);
  for (const rule of returns.rules) assert.equal(projection.linter.rules[rule], undefined);
  for (const [directory, corpus] of [
    [typedDirectory, typed],
    [returnDirectory, returns],
    ["tests/_fixtures/lint-default-correctness", first],
    ["tests/_fixtures/lint-default-migrations", second],
  ]) {
    for (const entry of corpus.cases) {
      assert.match(entry.filename, /^[A-Za-z0-9_-]+\.(?:vue|html|[cm]?[jt]sx?)$/);
      const bytes = await read(
        `${directory}/${entry.filename}.fixture`,
        sha256(Buffer.from(entry.source)),
      );
      assert.equal(bytes.toString("utf8"), entry.source);
      if (entry.sourceSha256) assert.equal(sha256(bytes), entry.sourceSha256);
    }
  }
  const helper = await import(
    pathToFileURL(path.join(destination, "tests/tooling/lint-default-correctness-reference.mjs"))
      .href
  );
  helper.loadCorpus(destination);
  return { typed, returns, first, second, projection, wholeJson: helper.wholeJson, authorities };
}
