import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export function stageProjectNative(
  artifactRoot: string,
  sourceBinary: string,
  sourceReceipt: string,
  expectedSha256: string,
): { artifacts: string; binary: string } {
  fs.mkdirSync(artifactRoot, { recursive: true });
  // Sticky target caches can contain prior evidence. Never reuse or remove it.
  const artifacts = fs.mkdtempSync(path.join(artifactRoot, "run-"));
  const binary = path.join(artifacts, "source-native.node");
  fs.copyFileSync(sourceBinary, binary, fs.constants.COPYFILE_EXCL);
  assert.equal(createHash("sha256").update(fs.readFileSync(binary)).digest("hex"), expectedSha256);
  fs.copyFileSync(
    sourceReceipt,
    path.join(artifacts, "build-receipt.json"),
    fs.constants.COPYFILE_EXCL,
  );
  // Keep this invocation's complete or failed evidence for the existing upload.
  return { artifacts, binary };
}

export function cleanupProjectNative(artifactRoot: string, ownedDirectory: string): void {
  const artifacts = path.resolve(ownedDirectory);
  assert.equal(path.dirname(artifacts), path.resolve(artifactRoot));
  assert.match(path.basename(artifacts), /^run-[A-Za-z0-9]{6}$/u);
  const entry = fs.lstatSync(artifacts);
  assert.ok(entry.isDirectory() && !entry.isSymbolicLink());
  assert.equal(path.dirname(fs.realpathSync(artifacts)), fs.realpathSync(artifactRoot));
  assert.ok(fs.statSync(path.join(artifacts, "qualification.json")).isFile());
  fs.rmSync(artifacts, { recursive: true });
}

if (process.argv[2] === "--cleanup") {
  const ownedDirectory = process.env.VIZE_OXLINT_STAGING_DIRECTORY;
  assert.ok(ownedDirectory, "only this successful invocation's output may be cleaned");
  cleanupProjectNative(
    fileURLToPath(new URL("../../../target/oxlint-original-project-transport", import.meta.url)),
    ownedDirectory,
  );
}
