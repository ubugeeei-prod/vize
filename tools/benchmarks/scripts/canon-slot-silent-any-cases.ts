/** Freeze framework-derived neighbours before choosing a slot fallback. */
import assert from "node:assert/strict";
import { slotCases, type SlotCase } from "./canon-slot-constructor-cases.ts";
export type Probe = SlotCase & { family: string; stage: string };
export function silentAnyCases(): Probe[] {
  const zero = slotCases().find((row) => row.id === "optional");
  assert.ok(zero);
  const base = zero;
  const signature = "{ new(props?: { value: number }): Instance }";
  assert.equal(zero.component.split(signature).length, 2);
  const rows: Probe[] = [];
  function add(family: string, component: string, source = base.source): void {
    rows.push({
      id: family + "-probe",
      family,
      stage: "probe",
      component,
      source,
      requiresAuthoredError: false,
    });
    rows.push({
      id: family + "-repaired",
      family,
      stage: "repaired",
      component: base.component,
      source: base.source,
      requiresAuthoredError: false,
    });
  }
  for (const type of ["unknown", "any", "never", "object", "{}", "unknown | { new(): Instance }"])
    add(
      "provider-" + type.replaceAll(/[^a-z]+/gu, "-").replace(/^-|-$/gu, ""),
      zero.component.replace(signature, type),
    );
  add(
    "unresolved-tag",
    zero.component,
    zero.source.replace("import WitnessChild from '../component';\n", ""),
  );
  for (const type of ["any", "unknown", "never", "string"])
    add("slot-" + type, zero.component.replace("item: number", "item: " + type));
  for (const shape of [
    "generic-default-zero",
    "generic-props",
    "overload-last-zero",
    "overload-first-zero",
  ]) {
    const row = slotCases().find((item) => item.id === shape);
    assert.ok(row);
    rows.push({ ...row, family: shape, stage: "control" });
  }
  assert.equal(new Set(rows.map((row) => row.id)).size, rows.length);
  return rows;
}
