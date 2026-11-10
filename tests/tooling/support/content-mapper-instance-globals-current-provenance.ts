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
    run: 42780733364,
    job: 129710794572,
    id: 13746003658,
    name: "instance-globals-tooling-9-42780733364-1",
    digest: "sha256:916f2e98c2f2b3e7ab3162ebeb81244f506f38b447c0ed20f50fad83d816bd2d",
    bytes: 394106,
  });
  const changed = [
    "lf/slot-shadow",
    "lf/mixed-slot-scope",
    "crlf-unicode/slot-shadow",
    "crlf-unicode/mixed-slot-scope",
  ];
  assert.ok(changed.every((label) => labels.includes(label)));
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 398,
    passedOutcomes: 394,
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
      "Only the LF and CRLF/Unicode slot-shadow and mixed-slot-scope packets respell their per-usage contract aliases as __V<Role><N> (#12646). Their text changes only at those names, every generated coordinate after a renamed name moves by exactly the bytes it lost, and every diagnostic and source coordinate is unchanged; the other 76 packets are byte-identical, and all 394 non-whole-packet semantic, protocol, mapping-geometry, reported-beta and official-nightly controls passed. The pinned source-built TypeScript host was unavailable, so this record retains the observed postimages without assigning host-matrix, public-package or performance acceptance credit.",
  });
}
