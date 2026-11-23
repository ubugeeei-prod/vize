import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
const artifacts = path.join(requiredEnv("RUNNER_TEMP"), "nuxt-public-kit-proof");
const before = JSON.parse(
  fs.readFileSync(path.join(artifacts, "before/installed-consumer-census.json"), "utf8"),
);
const after = JSON.parse(
  fs.readFileSync(path.join(artifacts, "installed-consumer-census.json"), "utf8"),
);
assert.equal(before.head, "9157b9fa83dbdc98100be084be288f21d0d1bc18");
for (const packet of [before, after])
  assert.equal(packet.proofSourceSha256, packet.executedProofSourceSha256);
assert.equal(before.proofSourceSha256, after.proofSourceSha256);
const normalized = (packet: Record<string, unknown> & { root: string }, field: string) =>
  JSON.parse(JSON.stringify(packet[field]).replaceAll(path.resolve(packet.root), "<ROOT>"));
for (const field of ["instances", "consumers"])
  assert.deepEqual(normalized(after, field), normalized(before, field));
assert.equal(before.moduleKit.version, "3.11.2");
assert.equal(after.moduleKit.version, "3.21.8");
assert.equal(before.explicitDevOracle, null);
const identity = (packet: Record<string, unknown>) =>
  Object.fromEntries(Object.entries(packet).filter(([name]) => name !== "braces"));
assert.deepEqual(
  before.kitChokidar,
  identity(before.consumers.find((packet: { name: string }) => packet.name === "chokidar")),
);
assert.deepEqual(
  after.explicitDevOracle,
  identity(after.consumers.find((packet: { name: string }) => packet.name === "chokidar")),
);
assert.equal(after.kitChokidar.version, "5.0.0");
assert.equal(JSON.parse(after.kitChokidar.manifestBytes).dependencies.braces, undefined);
fs.writeFileSync(
  path.join(artifacts, "census-transition.json"),
  JSON.stringify(
    { before: before.head, after: after.head, everyOriginalWholeConsumerPreserved: true },
    null,
    2,
  ) + "\n",
);
