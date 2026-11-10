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
    run: 42774589490,
    job: 129691996696,
    id: 13743648133,
    name: "instance-globals-tooling-14-42774589490-1",
    digest: "sha256:45256d8b1bf1fd62d15d3da613e14ad68f28d345cbf0ffdcc11f147f6e137316",
    bytes: 392307,
  });
  const slotPackets = ["slot-shadow", "mixed-slot-scope"];
  // Check mode omits only the four slot-packet navigation links; every other outcome passed.
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 398,
    passedOutcomes: 394,
    gapCount: 4,
    status: "unqualified",
    fullPacketQualification: false,
    hostMatrixQualification: false,
    gapIds: labels
      .filter((label) => slotPackets.some((packet) => label.endsWith(`/${packet}`)))
      .map((label) => `whole-packet/${label}`),
  });
  assert.deepEqual(provenance.review, {
    completePackets: 80,
    allNonWholePacketOutcomesPassed: true,
    qualificationCredit: false,
    nativeAcceptanceCredit: false,
    publicPackageAcceptanceCredit: false,
    performanceCredit: false,
    reason:
      "Only the LF and CRLF/Unicode slot-shadow and mixed-slot-scope whole packets dropped their wire-only vueSetupTemplateRefUnwrap semanticLinks in the diagnostics-only CLI projection (#12604); their generated text, mappings, diagnostic directives, diagnostics, and all other fields stayed exact, the other 76 complete results were byte-identical, and all 394 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
