import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { sourceDeclarations, sourceSdk } from "../../tools/benchmarks/scripts/canon-source-sdk.ts";

test("source execution copies all nine declarations while published package bytes stay intact", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-sdk-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const provider = path.join(root, "provider"),
    original = path.join(provider, "node_modules/vize"),
    project = path.join(root, "source");
  fs.mkdirSync(original, { recursive: true });
  fs.mkdirSync(project);
  const publishedManifest = JSON.stringify({
    name: "vize",
    version: "0.602.0",
    typescript: { contentMapper: { exec: ["./published-mapper"] } },
  });
  fs.writeFileSync(path.join(original, "package.json"), publishedManifest);
  for (const file of sourceDeclarations) {
    const target = path.join(original, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, `published ${file}\n`);
  }
  sourceSdk(project, provider, "/immutable/source/vize");
  assert.equal(fs.readFileSync(path.join(original, "package.json"), "utf8"), publishedManifest);
  for (const file of sourceDeclarations) {
    assert.equal(fs.readFileSync(path.join(original, file), "utf8"), `published ${file}\n`);
    const target = path.join(project, "node_modules/vize", file);
    assert.equal(fs.lstatSync(target).isSymbolicLink(), false);
    assert.deepEqual(fs.readFileSync(target), fs.readFileSync(path.resolve("npm/cli", file)));
  }
  const manifest = JSON.parse(
    fs.readFileSync(path.join(project, "node_modules/vize/package.json"), "utf8"),
  ) as { typescript: { contentMapper: { exec: string[] } } };
  assert.deepEqual(manifest.typescript.contentMapper.exec, [
    "/immutable/source/vize",
    "content-mapper",
  ]);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(project, "source-sdk-custody.json"), "utf8"),
  ) as {
    declarations: { file: string; source: { sha256: string }; installed: { sha256: string } }[];
  };
  assert.deepEqual(
    receipt.declarations.map((row) => row.file),
    sourceDeclarations,
  );
  for (const row of receipt.declarations) assert.equal(row.source.sha256, row.installed.sha256);
});
