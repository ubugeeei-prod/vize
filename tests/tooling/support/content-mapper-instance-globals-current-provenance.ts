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

const WHOLE_PACKET_CASES = [
  "original",
  "valid",
  "undeclared-only",
  "repaired",
  "repeated",
  "optional",
  "call",
  "v-for-source",
  "intentional-any",
  "declared-unknown",
  "plugin-valid",
  "plugin-invalid",
  "setup-shadow",
  "loop-shadow",
  "slot-shadow",
  "css-module",
  "own-props",
  "options-own-method",
  "expect-error-opening-child",
  "ignore-opening-child",
  "expect-owned-interpolation",
  "ignore-owned-interpolation",
  "expect-owned-attribute",
  "ignore-owned-attribute",
  "expect-owned-foreign-sibling",
  "ignore-owned-foreign-sibling",
  "shorthand-free",
  "typeof-value-free",
  "object-key",
  "member-key",
  "string-comment-role",
  "template-literal-raw",
  "template-literal-value",
  "callback-shadow",
  "callback-mixed",
  "callback-free",
  "computed-key-free",
  "mixed-loop-scope",
  "mixed-slot-scope",
  "destructured-callback-shadow",
];

export function assertCurrentPacketProvenance(provenance: CurrentPacketProvenance): void {
  assert.deepEqual(provenance.artifact, {
    repository: "ubugeeei-prod/vize",
    run: 42663572688,
    job: 129325555943,
    id: 13695998759,
    name: "instance-globals-10617-faaa9d3bb43c3bcb6096d6b4a47cab4d361313f3-1",
    digest: "sha256:0a95624682cb3d35d3f6b7fab206bbe29c89f3b7990318a6449a9092b8c49cf4",
    bytes: 18714937,
  });
  assert.deepEqual(provenance.capturedQualification, {
    totalOutcomes: 499,
    passedOutcomes: 419,
    gapCount: 80,
    status: "unqualified",
    fullPacketQualification: false,
    hostMatrixQualification: false,
    // Every whole packet changed; no other outcome failed.
    gapIds: ["lf", "crlf-unicode"].flatMap((variant) =>
      WHOLE_PACKET_CASES.map((name) => `whole-packet/${variant}/${name}`),
    ),
  });
  assert.deepEqual(provenance.review, {
    completePackets: 80,
    allNonWholePacketOutcomesPassed: true,
    qualificationCredit: false,
    nativeAcceptanceCredit: false,
    publicPackageAcceptanceCredit: false,
    performanceCredit: false,
    reason:
      "Each component constructor picks the public instance inline and keeps the bound instance behind NoInfer, so every packet's constructor return changed by the same two exact replacements while every generated coordinate, mapping, directive and semantic link moved with the text and all 419 semantic, protocol, mapping-geometry, and real-host controls passed; this record retains the observed postimages without assigning acceptance credit to the stale-golden run.",
  });
}
