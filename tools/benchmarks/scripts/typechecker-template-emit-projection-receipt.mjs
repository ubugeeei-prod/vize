import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, relative, resolve, sep } from "node:path";
import {
  loadRetainedProjectionFixture,
  qualifyLiveProjectionCapture,
  qualifyRetainedProjectionHistory,
  sha256,
} from "./typechecker-template-emit-projection-history.mjs";

const capture = process.env.VIZE_TEMPLATE_EMIT_TS40_CAPTURE;
assert(capture, "VIZE_TEMPLATE_EMIT_TS40_CAPTURE is required");
assert.match(process.env.SOURCE_SHA ?? "", /^[0-9a-f]{40}$/u);
const root = resolve(capture);
const receiptPath = join(root, "receipts.json");
assert(!existsSync(receiptPath), "projection receipt already exists");

const retained = loadRetainedProjectionFixture();
const historicalProjection = qualifyRetainedProjectionHistory(retained);
const currentProjection = qualifyLiveProjectionCapture(root, retained);
for (const row of retained.manifest.packets.baseline.files)
  assert(existsSync(join(root, row.path)), `current projection omitted ${row.path}`);

function files(directory, output = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((left, right) =>
    left.name.localeCompare(right.name, "en"),
  )) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files(path, output);
    else {
      assert(entry.isFile(), `unsupported projection capture member: ${path}`);
      output.push(path);
    }
  }
  return output;
}

const rawFiles = files(root).sort((left, right) =>
  relative(root, left).localeCompare(relative(root, right), "en"),
);
const rawPaths = rawFiles.map((path) => relative(root, path).split(sep).join("/"));
assert.deepEqual(
  rawPaths,
  retained.manifest.packets.incident.files.map(({ path }) => path),
  "current projection packet membership drifted",
);
const verifierUrl = new URL(import.meta.url);
const historyVerifierUrl = new URL(
  "./typechecker-template-emit-projection-history.mjs",
  import.meta.url,
);
writeFileSync(
  receiptPath,
  `${JSON.stringify(
    {
      schema: 2,
      sourceSha: process.env.SOURCE_SHA,
      verifierSha256: sha256(readFileSync(verifierUrl)),
      historyVerifierSha256: sha256(readFileSync(historyVerifierUrl)),
      currentProjection,
      historicalProjection,
      completeRawInputsAndOutputs: rawFiles.map((path, index) => ({
        path: rawPaths[index],
        sha256: sha256(readFileSync(path)),
      })),
    },
    null,
    2,
  )}\n`,
);
