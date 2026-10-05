import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = fileURLToPath(new URL("../../../../", import.meta.url));
export const originalRoot = path.join(
  repoRoot,
  "tests/_fixtures/differential/typechecker/vite-plus-relative-tsconfig",
);
export const originalNames = ["vite.config.mts", "tsconfig.app.json", "src/Counter.vue"];
export const diagnostic = "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'.";
export const emptyReport = {
  files: [],
  programs: [],
  errorCount: 0,
  warningCount: 0,
  fileCount: 0,
};

export function sha256(value: string | Uint8Array): string {
  return createHash("sha256").update(value).digest("hex");
}

type Identity = { bytes: number; sha256: string };
type Manifest = {
  schema: string;
  version: number;
  issue: string;
  reporter: { login: string; id: number };
  issueBody: Identity;
  files: Record<string, Identity>;
  reportedVersions: Record<string, string>;
  runtimeScope: string;
};

export function originalInputs(root = originalRoot) {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, "manifest.json"), "utf8"),
  ) as Manifest;
  assert.equal(manifest.schema, "vize.issue.original-inputs");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.issue, "https://github.com/ubugeeei-prod/vize/issues/8017");
  assert.deepEqual(manifest.reporter, { login: "ubugeeei", id: 71201308 });
  assert.deepEqual(Object.keys(manifest.files), originalNames);
  const body = fs.readFileSync(path.join(root, "issue-body.md"));
  assertIdentity(body, manifest.issueBody);
  const inputs: Record<string, string> = {};
  for (const name of originalNames) {
    const content = fs.readFileSync(path.join(root, name));
    assertIdentity(content, manifest.files[name]);
    const marker = `\x60${name}\x60\n\n\x60\x60\x60`;
    const start = body.indexOf(marker);
    assert.notEqual(start, -1);
    const afterLanguage = body.indexOf("\n", start + marker.length) + 1;
    const end = body.indexOf("\x60\x60\x60", afterLanguage);
    assert.notEqual(end, -1);
    assert.deepEqual(content, body.subarray(afterLanguage, end));
    inputs[name] = content.toString("utf8");
  }
  return { manifest, inputs };
}

function assertIdentity(bytes: Buffer, identity: Identity): void {
  assert.equal(bytes.length, identity.bytes);
  assert.equal(sha256(bytes), identity.sha256);
}
