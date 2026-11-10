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

export function assertCurrentPacketProvenance(
  provenance: CurrentPacketProvenance,
  labels: readonly string[],
): void {
  assert.deepEqual(provenance.artifact, {
    repository: "ubugeeei-prod/vize",
    run: 42781388461,
    job: 129712771074,
    id: 13746472560,
    name: "instance-globals-10617-0c3ae8dc16cec2a240124f550d7ecf7fb159946a-1",
    digest: "sha256:a36320d46224ffa25515e66dd680456799ac0a3db396bc7635938edf27c5104b",
    bytes: 18677366,
  });
  const changed = [
    "lf/slot-shadow",
    "lf/mixed-slot-scope",
    "crlf-unicode/slot-shadow",
    "crlf-unicode/mixed-slot-scope",
  ];
  assert.ok(changed.every((label) => labels.includes(label)));
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 499,
    passedOutcomes: 495,
    gapCount: 4,
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
      "The child usage in the LF and CRLF/Unicode slot-shadow and mixed-slot-scope packets now names the shared $UsageProps and $UsageRequiredFallthrough helpers instead of declaring their inline conditionals on its __Child_Props_0 and __Child_CheckProps_0 lines, so each packet shrank by exactly those two alias bodies and every later generated coordinate moved by the same bytes, while all 495 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
