import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
type Manifest = {
  version: number;
  schema: string;
  inputs: Array<{ source: string; bytes: number; sha256: string }>;
  cases: Array<{
    id: string;
    source: string;
    config: string | null;
    argv: string[];
    status: number;
    stderrBytes: number[];
    expected: unknown;
  }>;
};
export function validateComponentCliControls(value: unknown, read: (file: string) => Buffer) {
  assert.equal(
    sha(JSON.stringify(value)),
    "d7fb703384beace2fb6a39bb8e528c4867f571ba81ebd690e22727a8a9d993b2",
  );
  const manifest = value as Manifest;
  assert.equal(manifest.version, 1);
  assert.equal(manifest.schema, "vize.lint.component-cli-runtime-reads");
  assert.deepEqual(
    manifest.cases.map((case_) => case_.id),
    [
      "ecosystem-template-value",
      "ecosystem-unused",
      "ecosystem-comment-only",
      "configured-error",
      "configured-disabled",
    ],
  );
  for (const entry of manifest.inputs) {
    const body = read(entry.source);
    assert.equal(body.length, entry.bytes);
    assert.equal(sha(body), entry.sha256);
  }
  return { cases: manifest.cases, runtimeCredit: false, originalMatrixCredit: false };
}
export function loadComponentCliControls(root: string) {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "cases.json"), "utf8")) as Manifest;
  assert.deepEqual(
    fs.readdirSync(root).sort(),
    ["cases.json", ...manifest.inputs.map((entry) => entry.source)].sort(),
  );
  return validateComponentCliControls(manifest, (file) => {
    const target = path.join(root, file);
    assert.ok(fs.lstatSync(target).isFile());
    return fs.readFileSync(target);
  });
}
