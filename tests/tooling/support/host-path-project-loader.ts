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
  "0e354fb17cb1bd34e1d00a5c34489ccefa4161fb00ef12774ab4a5d17e7262b7",
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

// The level vocabulary successor changes exactly this module declaration;
// the loader move's original whole-file snapshots remain frozen.
const originalModules = "pub mod diag;\npub mod dump;\npub mod fact;\n";
const analysisModules = "pub mod analysis;\npub mod diag;\npub mod dump;\n";
assert.equal(record.moved.foundation.split(originalModules).length, 2);
export const analysisHostPathLayout = Object.freeze({
  ...record.moved,
  foundation: record.moved.foundation.replace(originalModules, analysisModules),
});

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
  if (layout.foundation === analysisHostPathLayout.foundation) {
    assert.deepEqual(layout, analysisHostPathLayout, "unknown analysis-module host path layout");
    return record.reference;
  }
  assert.deepEqual(layout, record.moved, "unknown or mixed host path source layout");
  return record.reference;
}
