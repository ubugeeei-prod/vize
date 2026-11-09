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
    run: 42480778738,
    job: 128734903108,
    id: 13617534399,
    name: "instance-globals-10617-dbc3ab84a7799535d6489d715ac252c7453324bc-1",
    digest: "sha256:5f44a68c1627cef54f381bafaf632a745c72dc461309e9aba597c714d69b2fc2",
    bytes: 18669937,
  });
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 499,
    passedOutcomes: 497,
    gapCount: 2,
    status: "unqualified",
    fullPacketQualification: false,
    hostMatrixQualification: false,
    gapIds: ["whole-packet/lf/options-own-method", "whole-packet/crlf-unicode/options-own-method"],
  });
  assert.deepEqual(provenance.review, {
    completePackets: 80,
    allNonWholePacketOutcomesPassed: true,
    qualificationCredit: false,
    nativeAcceptanceCredit: false,
    publicPackageAcceptanceCredit: false,
    performanceCredit: false,
    reason:
      "The exact #11499 constructor base shifted the two Options API packets while all 497 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
