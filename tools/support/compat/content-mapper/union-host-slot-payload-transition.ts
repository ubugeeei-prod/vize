import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

/**
 * The slot payload before it distributed over a union host: one parameter was
 * inferred from the union of the members' slot functions, which intersects
 * their payloads. The current line is read from its source-owned asset.
 */
export const intersectingNamedSlotPayload =
  "  type $NamedSlotPayload<S, N> = { [K in keyof S & N]-?: NonNullable<S[K]> extends (props: infer P, ...args: any[]) => any ? P : never }[keyof S & N] extends infer P ? ([P] extends [never] ? any : P) : any;";

const sharedPayloadPath = new URL(
  "../../../../crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-payload.d.ts.txt",
  import.meta.url,
);

export function unionHostNamedSlotPayload(): string {
  const lines = readFileSync(sharedPayloadPath, "utf8")
    .split("\n")
    .filter((line) => line.startsWith("  type $NamedSlotPayload<"));
  assert.equal(lines.length, 1, "one shared $NamedSlotPayload declaration");
  const line = lines[0]!;
  assert.ok(line.includes("(S extends unknown ? {"), "the payload distributes over the host");
  return line;
}

/** Advance a composed shared preamble across the union-host slot payload. */
export function applyUnionHostSlotPayloadIn(previous: string): string {
  assert.equal(
    previous.split(intersectingNamedSlotPayload).length,
    2,
    "intersecting slot payload predecessor line",
  );
  return previous.replace(intersectingNamedSlotPayload, () => unionHostNamedSlotPayload());
}

/** Inverse of {@link applyUnionHostSlotPayloadIn}. */
export function beforeUnionHostSlotPayloadIn(current: string): string {
  const line = unionHostNamedSlotPayload();
  assert.equal(current.split(line).length, 2, "union-host slot payload current line");
  return current.replace(line, () => intersectingNamedSlotPayload);
}
