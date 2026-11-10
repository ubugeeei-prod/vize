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
    run: 42736126056,
    job: 129566448378,
    id: 13726709260,
    name: "instance-globals-10617-8f1d67abf8491620cc642e26537fbf9be74020ab-1",
    digest: "sha256:77d86be670c33cdfbf53852cab06902dad03a5a8ce8ba9af165ff844841688d6",
    bytes: 18718518,
  });
  // Every whole packet changed (the canon train, #12572); every other outcome passed.
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
      "Every whole packet lost the two-space indentation of its authored setup lines and its setup map comment moved by the same bytes (#12550), every component constructor return gained the inline public-instance pick and NoInfer wrapper (#12523), and the slot-shadow and mixed-slot-scope packets dropped their per-file __U/__UN declarations and name the shared unwrap aliases (#12470); every mapping, directive and semantic link moved only by those edits, while all 419 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
