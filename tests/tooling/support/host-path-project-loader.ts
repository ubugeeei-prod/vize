import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export interface HostPathLayout {
  foundation: string;
  host: string;
  lowerPathExists: boolean;
  lower?: string;
}

const bytes = readFileSync(
  new URL("../../_fixtures/tooling/host-path-project-loader-11794.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(bytes).digest("hex"),
  "eb4d055d4db4379713f966ff7d6614846b90a3ce667490c7376f386c48101b91",
);
const record = JSON.parse(bytes.toString("utf8")) as {
  schema: string;
  reference: HostPathLayout;
  moved: HostPathLayout;
};
assert.equal(record.schema, "vize.host-path-project-loader-source-successor.v1");
assert.equal(record.reference.host.split("vize_l0::micro_count!").length, 3);
assert.equal(
  record.reference.host.replaceAll("vize_l0::micro_count!", "crate::micro_count!"),
  record.moved.lower,
  "all function bodies and Windows tests remain byte identical through the mechanical move",
);

export const originalHostPathLayout = Object.freeze(record.reference);
export const movedHostPathLayout = Object.freeze(record.moved);

/** Retain the original caller/host law through this one authenticated move. */
export function hostPathLayoutForOriginalLaw(layout: HostPathLayout): HostPathLayout {
  if (
    layout.foundation === record.reference.foundation &&
    layout.host === record.reference.host &&
    layout.lowerPathExists === false
  ) {
    assert.deepEqual(layout, record.reference, "unknown original host path layout");
    return layout;
  }
  assert.deepEqual(layout, record.moved, "unknown or mixed host path source layout");
  return record.reference;
}
