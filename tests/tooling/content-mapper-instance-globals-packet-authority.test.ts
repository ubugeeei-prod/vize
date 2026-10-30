import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type PacketGoldens,
} from "./support/content-mapper-instance-globals-10617.ts";
import {
  hasCompleteHostMatrix,
  hasCompletePacketQualification,
  selectPacketGoldens,
} from "./support/content-mapper-instance-globals-packets-10617.ts";

import { assertProfileComparatorNegatives } from "./support/content-mapper-instance-globals-profiles-10617.ts";

test(
  "profile controls reject hidden parsing, cache scope collapse and truncated evidence",
  assertProfileComparatorNegatives,
);

test("the original twenty provisional packets retain their complete immutable body", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const original = files.get("source-packets.json")!;
  assert.equal(sha(original), "bb2434fbf6d0f73165b5da6fc5674495c64727a4a0c19dcf09c01635d94d2cb9");
  const legacy = JSON.parse(original);
  assert.equal(legacy.complete, false);
  assert.equal(legacy.packets.length, 20);
  assert.equal(legacy.producer.sourceRevision, "125f93608ca1660ba68757e8ab0cac58b0451b2a");
  assert.deepEqual(
    selectPacketGoldens({ ...authority, currentSourcePackets: undefined }, files, cases),
    legacy,
  );
});

test("CURRENT cannot relabel the provisional capture or omit planned source coordinates", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const legacy = JSON.parse(files.get("source-packets.json")!);
  const name = "source-packets-current-125f93608ca1.json";
  const candidateFiles = new Map(files);
  const current = { ...authority, currentSourcePackets: name };
  assert.throws(() => selectPacketGoldens(current, candidateFiles, cases), /must be present/u);
  const raw = JSON.stringify(legacy);
  candidateFiles.set(name, raw);
  assert.throws(() => selectPacketGoldens(current, candidateFiles, cases), /authority row/u);
  const candidate = {
    ...current,
    files: [...authority.files, { path: name, bytes: Buffer.byteLength(raw), sha256: sha(raw) }],
  };
  assert.throws(() => selectPacketGoldens(candidate, candidateFiles, cases), /provisional/u);
  const incomplete = JSON.stringify({ ...legacy, complete: true });
  candidateFiles.set(name, incomplete);
  candidate.files[candidate.files.length - 1] = {
    path: name,
    bytes: Buffer.byteLength(incomplete),
    sha256: sha(incomplete),
  };
  assert.throws(
    () => selectPacketGoldens(candidate, candidateFiles, cases),
    /every planned packet/u,
  );
});

test("packet selection rejects foreign paths and byte changes outside its authority", () => {
  const { authority, files, cases } = readGlobalsFixture();
  assert.throws(() =>
    selectPacketGoldens(
      { ...authority, currentSourcePackets: "../source-packets.json" },
      files,
      cases,
    ),
  );
  const edited = new Map(files);
  edited.set("source-packets.json", `${edited.get("source-packets.json")} `);
  assert.throws(() =>
    selectPacketGoldens({ ...authority, currentSourcePackets: undefined }, edited, cases),
  );
});

test("complete packets cannot qualify omitted or skipped semantic and protocol checks", () => {
  const { files } = readGlobalsFixture();
  const provisional = JSON.parse(files.get("source-packets.json")!) as PacketGoldens;
  // Synthetic decision input only: the actual old fixture remains provisional.
  // The authority selector independently refuses its twenty rows as CURRENT.
  const golden = { ...provisional, complete: true };
  const sourceCases = golden.packets.map(({ label }: { label: string }, index: number) => ({
    id: index + 3,
    label,
    entry: { id: label.split("/")[1]! },
  }));
  const additionalHostIds = ["official-nightly", "pinned-source"];
  const outcomes = sourceCases.flatMap(({ id, label, entry }) =>
    [
      label,
      ...additionalHostIds.map((host) => `${label}/${host}`),
      `protocol/${id}/${entry.id}`,
      `whole-packet/${label}`,
    ].map((checkId) => ({
      id: checkId,
      outcome: "pass",
    })),
  );
  const controls = JSON.parse(files.get("receiver-plan-controls.json")!) as {
    programs: Array<{ id: string }>;
  };
  outcomes.push({ id: "receiver-plan/closed-protocol", outcome: "pass" });
  for (const { id } of controls.programs) {
    const prefix = `receiver-plan/${id}`;
    for (const suffix of [
      "frozen-independent-5.9",
      "symbols-and-geometry",
      "reported-beta",
      ...additionalHostIds,
    ])
      outcomes.push({ id: `${prefix}/${suffix}`, outcome: "pass" });
  }
  const baseline = { discoveryOnly: false, golden, sourceCases, additionalHostIds, outcomes };
  assert.equal(hasCompletePacketQualification(baseline), true);
  assert.equal(
    hasCompletePacketQualification({
      ...baseline,
      additionalHostIds: ["official-nightly"],
      outcomes: outcomes.filter(({ id }) => !id.endsWith("/pinned-source")),
    }),
    true,
    "generic two-host packets retain no matrix qualification credit",
  );
  assert.equal(hasCompletePacketQualification({ ...baseline, discoveryOnly: true }), false);
  assert.equal(hasCompletePacketQualification({ ...baseline, golden: provisional }), false);
  for (const outcome of ["discovery-unqualified", "skip", "gap"]) {
    const changed = outcomes.map((row, index) => (index === 0 ? { ...row, outcome } : row));
    assert.equal(hasCompletePacketQualification({ ...baseline, outcomes: changed }), false);
  }
  for (const prefix of [sourceCases[0]!.label, "protocol/", "whole-packet/", "receiver-plan/"]) {
    const missing = outcomes.filter(({ id }) => !id.startsWith(prefix));
    assert.equal(hasCompletePacketQualification({ ...baseline, outcomes: missing }), false);
  }
  assert.equal(
    hasCompletePacketQualification({ ...baseline, outcomes: [...outcomes, outcomes[0]!] }),
    false,
  );
  assert.equal(
    hasCompletePacketQualification({
      ...baseline,
      golden: { ...golden, packets: [...golden.packets].reverse() },
    }),
    false,
  );
});

test("only an admitted exact third-host roster acquires matrix qualification", () => {
  assert.equal(hasCompleteHostMatrix(true, true, ["official-nightly", "pinned-source"]), true);
  for (const ids of [
    [],
    ["official-nightly"],
    ["foreign", "pinned-source"],
    ["official-nightly", "foreign"],
    ["official-nightly", "pinned-source", "foreign"],
  ]) {
    assert.equal(hasCompleteHostMatrix(true, true, ids), false);
  }
  assert.equal(hasCompleteHostMatrix(false, true, ["official-nightly", "pinned-source"]), false);
  assert.equal(hasCompleteHostMatrix(true, false, ["official-nightly", "pinned-source"]), false);
});
