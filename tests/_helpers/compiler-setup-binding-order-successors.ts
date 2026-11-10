import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import type { CompilerOutput } from "./realworld-build-errors.ts";
import {
  assertAuthoredPieceCompilerSuccessor,
  authoredPieceCompilerSuccessor,
} from "./compiler-authored-piece-successors.ts";

// #12639 lists a template-error module's setup return bindings in Vue's order.
// This layer sits on the #11698 authored-piece packet: the reviewed successor
// moves only that object's shorthand properties.
export type SetupBindingOrderCase = "vue-router-broken";
type Review = {
  id: SetupBindingOrderCase;
  sourceSha256: string;
  previousPacketSha256: string;
  previousCodeSha256: string;
  currentPacketSha256: string;
  currentCodeSha256: string;
};
const directory = new URL("../_fixtures/compiler-setup-binding-order-12639/", import.meta.url);
const capture = JSON.parse(fs.readFileSync(new URL("capture.json", directory), "utf8")) as {
  rows: Review[];
};
const hash = (value: string) => createHash("sha256").update(value).digest("hex");
const serialize = (value: CompilerOutput) => JSON.stringify(value, null, 2);
const returned = /^(\s*)return \{ ([\w$]+(?:, [\w$]+)*) \}$/u;

export function verifySetupBindingOrderPair(
  id: SetupBindingOrderCase,
  previous: CompilerOutput,
  current: CompilerOutput,
): void {
  const review = capture.rows.find((row) => row.id === id);
  assert.ok(review, `unregistered setup binding order successor ${id}`);
  assert.equal(hash(serialize(previous)), review.previousPacketSha256);
  assert.equal(hash(previous.code), review.previousCodeSha256);
  assert.equal(hash(serialize(current)), review.currentPacketSha256);
  assert.equal(hash(current.code), review.currentCodeSha256);
  assert.deepEqual(current, { ...previous, code: current.code }, "exact full forward packet");
  assert.deepEqual(previous, { ...current, code: previous.code }, "exact full inverse packet");
  const before = previous.code.split("\n");
  const after = current.code.split("\n");
  assert.equal(after.length, before.length);
  const moved = before.flatMap((line, index) => (line === after[index] ? [] : [index]));
  assert.equal(moved.length, 1, `${id}: exactly one setup return line moves`);
  const [index] = moved;
  const old = returned.exec(before[index]);
  const next = returned.exec(after[index]);
  assert.ok(old && next, `${id}: the moved line is a shorthand setup return`);
  assert.equal(next[1], old[1]);
  const names = (match: RegExpExecArray) => match[2].split(", ");
  assert.equal(new Set(names(next)).size, names(next).length, `${id}: no duplicated binding`);
  assert.deepEqual(names(next).toSorted(), names(old).toSorted(), `${id}: the same bindings`);
  assert.notDeepEqual(names(next), names(old));
}

export function setupBindingOrderSuccessor(id: SetupBindingOrderCase) {
  const previous = authoredPieceCompilerSuccessor(id).output;
  const raw = fs.readFileSync(new URL(`${id}.json`, directory), "utf8");
  const output = JSON.parse(raw) as CompilerOutput;
  assert.equal(raw, serialize(output), "retain every serialization byte");
  verifySetupBindingOrderPair(id, previous, output);
  return { previous, output, raw };
}

export function assertSetupBindingOrderSuccessor(
  id: SetupBindingOrderCase,
  actualRaw: string,
  original: Parameters<typeof assertAuthoredPieceCompilerSuccessor>[2],
): void {
  const expected = setupBindingOrderSuccessor(id);
  assertAuthoredPieceCompilerSuccessor(id, serialize(expected.previous), original);
  assert.equal(actualRaw, expected.raw, `${id}: exact whole setup binding order packet`);
}
