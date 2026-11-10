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
    run: 42793104632,
    job: 129754946757,
    id: 13753414880,
    name: "instance-globals-10617-a15555ded2f2d44d5ccbfc68cda51d9198175be4-1",
    digest: "sha256:24f8584cf9a65a6fbf1a5ef02b909b12eaab9f8fe36355309d8a685009571b24",
    bytes: 18718659,
  });
  // Every whole packet changed (the generated comment cut, #12655); every other outcome passed.
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 499,
    passedOutcomes: 419,
    gapCount: 80,
    status: "unqualified",
    fullPacketQualification: false,
    hostMatrixQualification: false,
    gapIds: labels.map((label) => `whole-packet/${label}`),
  });
  assert.deepEqual(provenance.review, {
    completePackets: 80,
    allNonWholePacketOutcomesPassed: true,
    qualificationCredit: false,
    nativeAcceptanceCredit: false,
    publicPackageAcceptanceCredit: false,
    performanceCredit: false,
    reason:
      "Every whole packet lost the debug-only `@vize-map: expr|prop|handler|component` lines and the reasons after its generated `@ts-ignore` directives (the generated comment cut); every mapping, directive and semantic link moved only by those removed bytes, while all 419 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
