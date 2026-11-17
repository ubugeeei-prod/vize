import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeGenericHoistedAlias } from "./generic-hoisted-alias-transition.ts";
import { previousGenericSlotPayload } from "./generic-slot-payload-transition.ts";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
type FileTransition = { path: string; before: Identity; after: Identity; hunks: Hunk[] };
type UnionHostRecord = {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: FileTransition[];
};

/** Inverse of the union-host slot payload; mirrors payload_update/union_host_slot_payload.rs. */
const recordBytes = readFileSync(
  new URL("../../../../tests/_fixtures/tooling/union-host-slot-payload.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "b5fc83b7d0cdfbf82f45dac0bf04674c88becb6050e16107dc26adc9eefed085",
);
const record = JSON.parse(recordBytes.toString("utf8")) as UnionHostRecord;
assert.equal(record.schema, "vize.unionHostSlotPayloadSource.v1");
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The intersecting-payload bytes of a recorded helper asset; other images pass through. */
export function beforeUnionHostSlotPayloadSource(file: string, current: string): string {
  current = beforeGenericHoistedAlias(file, current);
  const transition = record.files.find((candidate) => candidate.path === file);
  if (!transition) return current;
  const now = identity(current);
  if (now.bytes !== transition.after.bytes || now.sha256 !== transition.after.sha256)
    return current;
  let previous = current;
  for (const hunk of transition.hunks) {
    assert.equal(previous.split(hunk.after).length - 1, 1);
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(previous), transition.before);
  return previous;
}

/**
 * The slot payload before it distributed over a union host: one parameter was
 * inferred from the union of the members' slot functions, which intersects
 * their payloads. The current line is read from its source-owned asset.
 */
export const intersectingNamedSlotPayload =
  "  type $NamedSlotPayload<S, N> = { [K in keyof S & N]-?: NonNullable<S[K]> extends (props: infer P, ...args: any[]) => any ? P : never }[keyof S & N] extends infer P ? ([P] extends [never] ? any : P) : any;";

/**
 * The union-host payload line. #13239 replaced it in the source-owned asset,
 * so its recorded predecessor is authoritative here.
 */
export function unionHostNamedSlotPayload(): string {
  const line = previousGenericSlotPayload;
  assert.equal(line.split("\n").length, 1, "one shared $NamedSlotPayload declaration");
  assert.ok(line.startsWith("  type $NamedSlotPayload<"), "the union-host payload line");
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
