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
    run: 42680895978,
    job: 129375771855,
    id: 13704185312,
    name: "instance-globals-10617-a8146816896489c638b4c88c9f1ef31775d5f5a0-1",
    digest: "sha256:08e204445821ba9d397c9bac3c00c5607e7923125b648ad14aa77a8bd8c5cb69",
    bytes: 18675726,
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
      "The component-props pass no longer replays a v-slot scope whose body checks no component, so the LF and CRLF/Unicode slot-shadow and mixed-slot-scope packets each lost exactly that 214-byte replay and its one mapped $ghost binding, the two unmapped-gap ignore rows around that binding merged into one, and every later generated coordinate moved by 214, while all 495 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
