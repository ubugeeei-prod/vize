import assert from "node:assert/strict";

type CurrentPacketProvenance = {
  artifact: {
    repository: string;
    run: number;
    job: number;
    id: number;
    name: string;
    digest: string;
    bytes: number;
  };
  capturedQualification: {
    totalOutcomes: number;
    passedOutcomes: number;
    gapCount: number;
    status: string;
    fullPacketQualification: boolean;
    hostMatrixQualification: boolean;
    gapIds: string[];
  };
  review: {
    completePackets: number;
    allNonWholePacketOutcomesPassed: boolean;
    qualificationCredit: boolean;
    nativeAcceptanceCredit: boolean;
    publicPackageAcceptanceCredit: boolean;
    performanceCredit: boolean;
    reason: string;
  };
};

/** Packets whose component type has no fallthrough surface stay byte-identical. */
const unchangedCases = [
  "v-for-source",
  "loop-shadow",
  "expect-owned-interpolation",
  "ignore-owned-interpolation",
  "expect-owned-foreign-sibling",
  "ignore-owned-foreign-sibling",
  "mixed-loop-scope",
  "mixed-slot-scope",
];

export function currentPacketChanges(labels: readonly string[]): string[] {
  return labels.filter((label) => !unchangedCases.includes(label.slice(label.indexOf("/") + 1)));
}

export function assertCurrentPacketProvenance(
  provenance: CurrentPacketProvenance,
  labels: readonly string[],
): void {
  assert.deepEqual(provenance.artifact, {
    repository: "ubugeeei-prod/vize",
    run: 43251466440,
    job: 131299205802,
    id: 13959294344,
    name: "instance-globals-10617-558eaa1dcc10886035238dba76ee13f9d677ff78-1",
    digest: "sha256:a456d056a496d04877221faea72084ad817add6e3a28554f683e41194957a46b",
    bytes: 18699200,
  });
  const changed = currentPacketChanges(labels);
  assert.equal(labels.length, 80);
  assert.equal(changed.length, 64);
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 499,
    passedOutcomes: 435,
    gapCount: 64,
    status: "unqualified",
    fullPacketQualification: false,
    hostMatrixQualification: false,
    gapIds: changed.map((label) => `whole-packet/${label}`),
  });
  assert.deepEqual(provenance.review, {
    completePackets: 80,
    allNonWholePacketOutcomesPassed: true,
    qualificationCredit: false,
    nativeAcceptanceCredit: false,
    publicPackageAcceptanceCredit: false,
    performanceCredit: false,
    reason:
      "64 LF and CRLF/Unicode whole packets gained exactly one raw `readonly __vizeFallthroughSource?: <fallthrough source>;` member ahead of __vizeHasFallthroughProps in their component type (#12863), with every later generated coordinate moved by that member's bytes; the two string-comment-role packets also dropped their one ignore directive over a single space (#12848). The other 16 complete results were byte-identical and all 435 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
