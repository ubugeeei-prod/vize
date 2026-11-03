import assert from "node:assert/strict";
import {
  sha,
  type GlobalCase,
  type PacketGoldens,
} from "./content-mapper-instance-globals-10617.ts";

type PacketAuthority = {
  files: Array<{ path: string; bytes: number; sha256: string }>;
  currentSourcePackets?: string;
};

/** Select an authenticated current successor without changing historical packets. */
export function selectPacketGoldens(
  authority: PacketAuthority,
  files: ReadonlyMap<string, string>,
  cases: readonly GlobalCase[],
): PacketGoldens | undefined {
  const current = authority.currentSourcePackets;
  const name = current ?? "source-packets.json";
  if (current !== undefined) assert.match(current, /^source-packets-current-[a-f0-9]{12}\.json$/u);
  const raw = files.get(name);
  if (raw === undefined) {
    assert.equal(current, undefined, "the selected current packet set must be present");
    return undefined;
  }
  const rows = authority.files.filter((row) => row.path === name);
  assert.equal(rows.length, 1, "the packet set must have one complete authority row");
  assert.equal(Buffer.byteLength(raw), rows[0]!.bytes, name);
  assert.equal(sha(raw), rows[0]!.sha256, name);
  const golden = JSON.parse(raw) as PacketGoldens;
  assert.equal(golden.schema, "vize.instance-global-complete-transforms");
  assert.equal(golden.version, 1);
  assert.equal(golden.issue, 10617);
  if (current !== undefined) {
    assert.equal(golden.complete, true, "a provisional packet set cannot become CURRENT");
    const producer = golden.producer;
    assert.ok(producer && typeof producer === "object" && !Array.isArray(producer));
    const revision = (producer as Record<string, unknown>).sourceRevision;
    assert.equal(typeof revision, "string");
    assert.match(revision as string, /^[a-f0-9]{40}$/u);
    assert.equal(current, `source-packets-current-${(revision as string).slice(0, 12)}.json`);
    assert.deepEqual(
      golden.packets.map((packet) => packet.label),
      ["lf", "crlf-unicode"].flatMap((variant) => cases.map(({ id }) => `${variant}/${id}`)),
      "CURRENT must retain every planned packet in its original producer order",
    );
  }
  return golden;
}

type QualificationOutcome = { id: string; outcome: string };

/** Complete snapshots cannot acquire credit from skipped or omitted semantic checks. */
export function hasCompletePacketQualification({
  discoveryOnly,
  golden,
  sourceCases,
  additionalHostIds,
  outcomes,
}: {
  discoveryOnly: boolean;
  golden: PacketGoldens | undefined;
  sourceCases: readonly { id: number; label: string; entry: { id: string } }[];
  additionalHostIds: readonly string[];
  outcomes: readonly QualificationOutcome[];
}): boolean {
  const labels = sourceCases.map(({ label }) => label);
  if (discoveryOnly || golden?.complete !== true || labels.length === 0) return false;
  if (golden.packets.length !== labels.length) return false;
  if (!golden.packets.every((packet, index) => packet.label === labels[index])) return false;
  if (new Set(labels).size !== labels.length) return false;
  const ids = new Set(outcomes.map(({ id }) => id));
  if (ids.size !== outcomes.length || !outcomes.every(({ outcome }) => outcome === "pass")) {
    return false;
  }
  if (!isKnownHostRoster(additionalHostIds)) return false;
  const receiverIds = ["receiver-plan/closed-protocol"];
  for (const name of [
    "unresolved-value",
    "unresolved-type-query",
    "unrelated-function-name",
    "outer-script-binding",
  ]) {
    const prefix = `receiver-plan/${name}`;
    receiverIds.push(`${prefix}/frozen-independent-5.9`, `${prefix}/symbols-and-geometry`);
    for (const host of ["reported-beta", ...additionalHostIds])
      receiverIds.push(`${prefix}/${host}`);
  }
  if (!receiverIds.every((id) => ids.has(id))) return false;
  return sourceCases.every(({ id, label, entry }) =>
    [
      label,
      ...additionalHostIds.map((hostId) => `${label}/${hostId}`),
      `protocol/${id}/${entry.id}`,
      `whole-packet/${label}`,
    ].every((checkId) => ids.has(checkId)),
  );
}

function isKnownHostRoster(ids: readonly string[]): boolean {
  return (
    ids[0] === "official-nightly" &&
    (ids.length === 1 || (ids.length === 2 && ids[1] === "pinned-source"))
  );
}

/** Only the dedicated, admitted third physical host can acquire matrix credit. */
export function hasCompleteHostMatrix(
  packetsQualified: boolean,
  sourceHostAdmitted: boolean,
  additionalHostIds: readonly string[],
): boolean {
  return (
    packetsQualified &&
    sourceHostAdmitted &&
    additionalHostIds.length === 2 &&
    isKnownHostRoster(additionalHostIds)
  );
}
