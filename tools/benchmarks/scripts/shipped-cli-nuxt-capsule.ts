import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import type { Receipt, Snapshot } from "./shipped-cli-nuxt-protocol.ts";

export const fixture = fileURLToPath(
  new URL("../../../tests/_fixtures/tooling/shipped-cli-nuxt/", import.meta.url),
);

export const sha256 = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
export const readJson = (path: string) => JSON.parse(readFileSync(path, "utf8"));
export const writeJson = (path: string, value: unknown) =>
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
function readReceipt(): Receipt {
  return readJson(join(fixture, "receipt.json")) as Receipt;
}

export function loadSnapshot(): { receipt: Receipt; snapshot: Snapshot } {
  const receipt = readReceipt();
  const compressed = readFileSync(join(fixture, "snapshot.json.gz"));
  assert.equal(sha256(compressed), receipt.snapshotSha256, "frozen snapshot digest differs");
  const json = gunzipSync(compressed);
  assert.equal(sha256(json), receipt.snapshotJsonSha256, "frozen snapshot JSON digest differs");
  assert.equal(
    sha256(readFileSync(join(fixture, "package-lock.json"))),
    receipt.lockSha256,
    "frozen lock digest differs",
  );
  assert.equal(
    sha256(readFileSync(join(fixture, "package.json"))),
    receipt.packageSha256,
    "frozen package digest differs",
  );
  const snapshot = JSON.parse(json.toString("utf8")) as Snapshot;
  assert.equal(Object.keys(snapshot.files).length, receipt.fixtureFileCount);
  assert.equal(Object.keys(snapshot.programInputSha256).length, receipt.programInputCount);
  assert.equal(snapshot.expectedOutput.fileCount, receipt.reportedFileCount);
  assert.equal(snapshot.expectedOutput.programs.length, 1);
  assert.equal(snapshot.expectedOutput.programs[0].files.length, receipt.programInputCount);
  for (const path of [...Object.keys(snapshot.files), ...Object.keys(snapshot.programInputSha256)])
    safeRelative(path);
  return { receipt, snapshot };
}

export function safeRelative(path: string) {
  assert.ok(
    path && !path.startsWith("/") && !path.includes("\\") && !path.split("/").includes(".."),
    `unsafe snapshot path: ${path}`,
  );
}

export function paths(work: string) {
  return {
    project: join(work, "vize-p1-nuxt-scale-consumer"),
    dependencies: join(work, "vize-p1-nuxt-final-consumer"),
  };
}

/** The only content substitutions are the two captured absolute filesystem roots. */
export function relocate(
  text: string,
  receipt: Pick<Receipt, "originalProject" | "originalDependencies">,
  work: string,
  reverse = false,
) {
  const current = paths(work);
  const replacements = [
    [receipt.originalProject, current.project],
    [receipt.originalDependencies, current.dependencies],
  ];
  for (const [original, actual] of replacements)
    text = text.replaceAll(reverse ? actual : original, reverse ? original : actual);
  return text;
}

export function validateOutput(
  actual: unknown,
  snapshot: Snapshot,
  receipt: Receipt,
  work: string,
) {
  const restored = JSON.parse(relocate(JSON.stringify(actual), receipt, work, true));
  assert.deepEqual(
    restored,
    snapshot.expectedOutput,
    "complete JSON diagnostic/configuration/program-input oracle differs",
  );
}

export function validateProducer(
  run: Record<string, unknown>,
  artifact: Record<string, unknown>,
  receipt: Receipt,
) {
  const p = receipt.producer;
  assert.equal(run.id, p.runId, "producer run differs");
  assert.equal(run.head_sha, p.driverSha, "producer source differs");
  assert.equal(run.path, p.workflow, "producer workflow differs");
  assert.equal(run.event, "workflow_dispatch", "producer event differs");
  assert.equal(run.conclusion, "success", "producer did not pass");
  assert.equal(run.status, "completed", "producer is incomplete");
  assert.equal(artifact.id, p.artifactId, "artifact ID differs");
  assert.equal(artifact.name, p.artifactName, "artifact name differs");
  assert.equal(artifact.expired, false, "artifact expired; rebuild and review a new receipt");
  assert.equal(artifact.digest, `sha256:${p.artifactSha256}`, "artifact digest differs");
  const owner = artifact.workflow_run as Record<string, unknown>;
  assert.equal(owner.id, p.runId, "artifact producer run differs");
  assert.equal(owner.head_sha, p.driverSha, "artifact producer source differs");
}
