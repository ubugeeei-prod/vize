import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const hash7502 = (bytes: string | Buffer) =>
  createHash("sha256").update(bytes).digest("hex");
export const fixtureUrl7502 = new URL(
  "../../../crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/original_inputs.json",
  import.meta.url,
);
export const outputUrl7502 = new URL("reviewed_output.json", fixtureUrl7502);
export const fixtureSha7502 = "130e3b247f1333f66ed1a529b7618e8de28cd926b33e6a57e2a12de640645b8f";
export const ledgerSha7502 = "a7f0d7ea54e8d687057e36a778d8d7fab8bfc2f5a9fd5e31a3c226b3bc5928f8";
export const targets7502 = [
  { target: "dom", runtimeVersion: "3.5.35" },
  { target: "ssr", runtimeVersion: "3.5.35" },
  { target: "vapor", runtimeVersion: "3.6.0-rc.9" },
];
export const modes7502 = [
  { linkMode: "Recorded", sourceMap: true },
  { linkMode: "NoLinks", sourceMap: false },
];
export function exactKeys7502(value: any, expected: string[]) {
  assert(value && typeof value === "object" && !Array.isArray(value));
  assert.deepEqual(Object.keys(value).sort(), expected.toSorted());
}
export function loadInputs7502(bytes = readFileSync(fixtureUrl7502)) {
  assert.equal(hash7502(bytes), fixtureSha7502, "original input pack drift");
  const pack = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
  exactKeys7502(pack, [
    "schema",
    "version",
    "ledgerSha256",
    "originalFix",
    "originalParent",
    "testBlob",
    "reporterBlob",
    "authority",
    "fixtures",
  ]);
  assert.equal(pack.schema, "vize.native-attribute-values-7502.inputs");
  assert.equal(pack.version, 1);
  assert.equal(pack.ledgerSha256, ledgerSha7502);
  assert.equal(pack.originalFix, "578f770216b7b3b5b8c168698e664b85addfd6e0");
  assert.equal(pack.originalParent, "a2fc9c9ec4fb2dd913dfce146691caa8009b3976");
  assert.equal(pack.testBlob, "70872e2d54bc06021c4e791bb61a99538071f909");
  assert.equal(pack.reporterBlob, "36819dc73df85b56b389bda9e4d10bfb9511d05b");
  assert.deepEqual(
    pack.fixtures.map((row: any) => row.id),
    [
      "reporter-7502",
      ...Array.from({ length: 13 }, (_, index) => `original-regression-${index + 1}`),
    ],
  );
  for (const row of pack.fixtures) {
    exactKeys7502(row, [
      "id",
      "source",
      "sourceSha256",
      "existingExpectedTemplateHtml",
      "expectedTemplateHtmlSha256",
      "existingOptions",
      "disposition",
    ]);
    assert.equal(row.sourceSha256, hash7502(row.source));
    assert.equal(row.expectedTemplateHtmlSha256, hash7502(row.existingExpectedTemplateHtml));
    assert.equal(row.existingOptions, "compile_vapor Default::default()");
    assert.equal(
      row.disposition,
      ["original-regression-11", "original-regression-12"].includes(row.id)
        ? "lower-refusal"
        : "positive",
    );
  }
  return pack;
}
export function requireReviewed7502(actual: any, frozen: any) {
  exactKeys7502(frozen, ["schema", "version", "state", "capture"]);
  assert.equal(frozen.schema, "vize.native-attribute-values-7502.reviewed-output");
  assert.equal(frozen.version, 1);
  assert.equal(frozen.state, "reviewed", "whole native code/maps remain unfrozen");
  assert(
    frozen.capture && typeof frozen.capture === "object",
    "complete reviewed capture required",
  );
  assert.deepEqual(
    actual,
    frozen.capture,
    "complete current native capture must equal frozen output",
  );
}
