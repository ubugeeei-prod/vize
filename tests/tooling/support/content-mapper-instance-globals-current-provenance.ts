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
    run: 42772339922,
    job: 129684248644,
    id: 13742224501,
    name: "instance-globals-tooling-15-42772339922-1",
    digest: "sha256:562e1bc06c236903cb89ce8fd49881919870d76846c2599e6539b725f2deeeb7",
    bytes: 393773,
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
      "Only the LF and CRLF/Unicode slot-shadow and mixed-slot-scope packets add the exact disjoint opening and closing slot-pattern gap mappings required by #12511. Their generated text, diagnostics, semantic links and all other opaque fields remain exact, the other 76 packets are byte-identical, and all 394 non-whole-packet semantic, protocol, mapping-geometry, reported-beta and official-nightly controls passed. The pinned source-built TypeScript host was unavailable, so this record retains the observed postimages without assigning host-matrix, public-package or performance acceptance credit.",
  });
}
