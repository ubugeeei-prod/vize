import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const directory = new URL(
  "../_fixtures/differential/compiler/vapor-literal-object-styles-10749/ssr-expose-successor-10923/",
  import.meta.url,
);
const pins = {
  "base-sfc.json": "20994c4313510adca6da6dcdde30e2cee352dba2bc96632c2a3c1721ea1ff846",
  "head-sfc.json": "ef34fdaa2b41bfb1252ce06fb25fdec77f5527758463a3c72db21f15ecb59277",
  "producer-capture.json": "ee5fda65e3359c185a5cfe6587f575e07666775a81bb048e954d2d7acf0bad88",
};
const archive = {
  sourceHead: "6fc192271eafbccaac463104ca27af61b205f58f",
  sourceTree: "afe2befecf8c9375dd4edfd3fca300608646cfec",
  runId: 40494324373,
  artifactId: 12729352671,
  artifactDigest: "sha256:973096b09f9d3fb0f589c4258127a381498a44d41d1f441bba3d802950647c00",
};
const sourceMerge = "9157b9fa83dbdc98100be084be288f21d0d1bc18";
const previousCode = "setup(__props) {";
const currentCode = "setup(__props, { expose: __expose }) {\n    __expose();";
type Packet = Record<string, unknown> & { code: string };
type SfcRow = {
  source: string;
  vapor: boolean;
  ssr: boolean;
  fold: boolean;
  output: Packet;
};

export function verifyVaporSsrExposeRegistration(value: unknown): void {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  const registration = value as Record<string, unknown>;
  assert.deepEqual(
    Object.keys(registration).sort(),
    [
      "archive",
      "changedFacets",
      "codeEdit",
      "indices",
      "rawFiles",
      "schemaVersion",
      "scope",
      "sourceMerge",
      "sourcePullRequest",
    ].sort(),
  );
  assert.equal(registration.schemaVersion, 1);
  assert.equal(registration.sourcePullRequest, 10923);
  assert.equal(registration.sourceMerge, sourceMerge);
  assert.deepEqual(registration.indices, [1, 3]);
  assert.deepEqual(registration.changedFacets, ["code"]);
  assert.deepEqual(registration.codeEdit, {
    previous: previousCode,
    current: currentCode,
    occurrences: 1,
  });
  assert.deepEqual(registration.archive, archive);
  assert.deepEqual(registration.rawFiles, pins);
  assert.equal(typeof registration.scope, "string");
}

function readPinned(name: keyof typeof pins): string {
  const bytes = fs.readFileSync(new URL(name, directory));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), pins[name], name);
  return bytes.toString("utf8");
}

/** Only the two archived SSR rows admit one reviewed main-code change. */
export function frozenVaporSsrExposePair(index: number): { before: SfcRow; after: SfcRow } {
  assert.ok(index === 1 || index === 3, "only original SSR indices 1 and 3 are registered");
  verifyVaporSsrExposeRegistration(
    JSON.parse(fs.readFileSync(new URL("admission.json", directory), "utf8")),
  );
  const producer = JSON.parse(readPinned("producer-capture.json")) as Record<string, unknown> & {
    rawFiles: Record<string, string>;
    transitions: { sfc: unknown };
  };
  for (const [key, value] of Object.entries(archive)) assert.equal(producer[key], value);
  assert.equal(producer.rawFiles["base-sfc.json"], pins["base-sfc.json"]);
  assert.equal(producer.rawFiles["head-sfc.json"], pins["head-sfc.json"]);
  assert.deepEqual(producer.transitions.sfc, {
    indices: [1, 3],
    facets: ["code"],
    sourcePullRequest: 10923,
    sourceMerge,
  });
  const base = JSON.parse(readPinned("base-sfc.json")) as SfcRow[];
  const head = JSON.parse(readPinned("head-sfc.json")) as SfcRow[];
  assert.equal(base.length, 4);
  assert.equal(head.length, 4);
  const before = base[index]!;
  const after = head[index]!;
  assert.equal(before.ssr, true);
  assert.equal(before.fold, false);
  assert.equal(before.vapor, index === 3);
  assert.equal(before.output.code.split(previousCode).length - 1, 1, "one exact setup anchor");
  assert.deepEqual(
    after,
    {
      ...before,
      output: { ...before.output, code: before.output.code.replace(previousCode, currentCode) },
    },
    "every other whole-row/output field remains unchanged",
  );
  return { before, after };
}

/** Actual producer packets must match both frozen sides without normalization. */
export function assertVaporSsrExposeSuccessor(
  index: number,
  before: unknown,
  after: unknown,
): void {
  const frozen = frozenVaporSsrExposePair(index);
  assert.deepEqual(before, frozen.before, `SSR ${index}: exact original complete producer packet`);
  assert.deepEqual(after, frozen.after, `SSR ${index}: exact current complete producer packet`);
}

/** Fixed case indices cannot be reclassified by changing producer fold flags. */
export function assertVaporSfcExposeTransition(before: SfcRow[], after: SfcRow[]): void {
  assert.equal(before.length, 4);
  assert.equal(after.length, 4);
  for (let index = 0; index < 4; index++) {
    const previous = before[index]!;
    const row = after[index]!;
    for (const key of ["source", "vapor", "ssr", "fold"] as const)
      assert.deepEqual(row[key], previous[key]);
    if (index === 1 || index === 3) assertVaporSsrExposeSuccessor(index, previous, row);
    else if (index === 2) {
      assert.equal(row.fold, true);
      assert.notDeepEqual(row.output, previous.output);
    } else {
      assert.equal(index, 0);
      assert.equal(row.fold, false);
      assert.deepEqual(row.output, previous.output);
    }
  }
}
