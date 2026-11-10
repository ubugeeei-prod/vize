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
    run: 42795331506,
    job: 129767784297,
    id: 13754879135,
    name: "instance-globals-10617-754dfff8285f920404ba958ed0c8d6594db0160e-1",
    digest: "sha256:7944f79bd6bc40b9453950e82f29c3f3d8b5b9b9bd12f5537d33ad97031bfa0e",
    bytes: 18675495,
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
      "The child usage in the LF and CRLF/Unicode slot-shadow and mixed-slot-scope packets now spells its per-usage contract aliases __VProps0, __VComp0, __VCheckProps0, __VCheckTail0 and __VCheck0 instead of __Child_<Role>_0 (#12646), so each packet's text changes only at those names and every generated coordinate after a renamed name moves by exactly the bytes it lost, while all 495 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
