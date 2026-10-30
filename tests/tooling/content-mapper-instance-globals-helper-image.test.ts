import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { makeSourceProject, sha } from "./support/content-mapper-instance-globals-10617.ts";

test("10617 source adapter preserves the complete current helper declaration image", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-instance-globals-helper-image-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const repoRoot = fileURLToPath(new URL("../../", import.meta.url));
  const published = path.join(root, "published");
  fs.mkdirSync(path.join(published, "node_modules"), { recursive: true });
  const binary = path.join(root, "source-cli-identity");
  fs.writeFileSync(binary, "source CLI identity control; not an executable consumer\n");
  const project = path.join(root, "source");
  const receipt = makeSourceProject(project, published, repoRoot, binary, new Map());
  const adapter = path.join(project, "node_modules/vize");
  const helpers = [
    "content-mapper-helpers.d.ts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.cts",
    "generated/content-mapper-types.d.ts",
  ];
  for (const name of helpers) {
    assert.deepEqual(
      fs.readFileSync(path.join(adapter, name)),
      fs.readFileSync(path.join(repoRoot, "npm/cli", name)),
      name,
    );
  }
  const manifest = JSON.parse(fs.readFileSync(path.join(repoRoot, "npm/cli/package.json"), "utf8"));
  manifest.private = true;
  manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  const manifestBytes = `${JSON.stringify(manifest, null, 2)}\n`;
  assert.equal(fs.readFileSync(path.join(adapter, "package.json"), "utf8"), manifestBytes);
  assert.deepEqual(receipt, {
    role: "source protocol adapter, not public-package acceptance",
    manifestSha256: sha(manifestBytes),
    binarySha256: sha(fs.readFileSync(binary)),
    helperFiles: helpers.map((name) => ({
      name,
      sha256: sha(fs.readFileSync(path.join(repoRoot, "npm/cli", name))),
    })),
  });
  assert.deepEqual(fs.readdirSync(adapter).sort(), [
    "content-mapper-helpers.d.cts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.ts",
    "generated",
    "package.json",
  ]);
  assert.deepEqual(fs.readdirSync(path.join(adapter, "generated")), ["content-mapper-types.d.ts"]);
});
