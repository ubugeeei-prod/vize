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

export function assertCurrentPacketProvenance(provenance: CurrentPacketProvenance): void {
  assert.deepEqual(provenance.artifact, {
    repository: "ubugeeei-prod/vize",
    run: 42646667936,
    job: 129264778908,
    id: 13690100504,
    name: "instance-globals-10617-8827c3de910036ad616a4ebbf441a2b4f14a7684-1",
    digest: "sha256:e350807b134e8c1b2a4d440170e6b29ab969cb83d58bb7f548adaf9d1c333cbf",
    bytes: 18676246,
  });
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 499,
    passedOutcomes: 495,
    gapCount: 4,
    status: "unqualified",
    fullPacketQualification: false,
    hostMatrixQualification: false,
    gapIds: [
      "whole-packet/lf/slot-shadow",
      "whole-packet/lf/mixed-slot-scope",
      "whole-packet/crlf-unicode/slot-shadow",
      "whole-packet/crlf-unicode/mixed-slot-scope",
    ],
  });
  assert.deepEqual(provenance.review, {
    completePackets: 80,
    allNonWholePacketOutcomesPassed: true,
    qualificationCredit: false,
    nativeAcceptanceCredit: false,
    publicPackageAcceptanceCredit: false,
    performanceCredit: false,
    reason:
      "The component contract now passes never as the required-key set when every fallthrough member admits {}, so the four slot-shadow and mixed-slot-scope packets carry the gated $RequiredFallthroughProps reference on their one __Child_CheckProps_0 line and every later generated coordinate moved by that growth while all 495 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
