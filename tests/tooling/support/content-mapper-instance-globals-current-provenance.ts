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
    run: 42628370735,
    job: 129198720061,
    id: 13681080320,
    name: "instance-globals-10617-11e4add58056f6dabc81a32ce94db94db1cdbce0-1",
    digest: "sha256:ec4071b01f99551e15eed50ecbd46a5f7783563af188ffbe8fe3ced0a00b68b4",
    bytes: 18672687,
  });
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 499,
    passedOutcomes: 497,
    gapCount: 2,
    status: "unqualified",
    fullPacketQualification: false,
    hostMatrixQualification: false,
    gapIds: ["whole-packet/lf/css-module", "whole-packet/crlf-unicode/css-module"],
  });
  assert.deepEqual(provenance.review, {
    completePackets: 80,
    allNonWholePacketOutcomesPassed: true,
    qualificationCredit: false,
    nativeAcceptanceCredit: false,
    publicPackageAcceptanceCredit: false,
    performanceCredit: false,
    reason:
      "The template context no longer declares $GlobalContextIsAny, $GlobalContextProperty and __Global when every template global carries its own type, so the two CSS-module packets lost exactly those 293 bytes and every later generated coordinate moved by 293 while all 497 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
