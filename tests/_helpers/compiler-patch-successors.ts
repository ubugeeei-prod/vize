// Complete reviewed successors retain the old patch-oracle bytes and their
// original hashes. Runtime/product parity remains outside this source contract.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import type { CompilerOutput } from "./realworld-build-errors.ts";

export type CompilerPatchCase =
  | "create-vue-clean"
  | "create-vue-broken"
  | "vue-router-clean"
  | "vue-router-broken";
interface Review {
  id: CompilerPatchCase;
  input: string;
  inputSha256: string;
  previous: string;
  previousSha256: string;
  previousCodeSha256: string;
  current: string;
  currentSha256: string;
  currentCodeSha256: string;
  changedFacets: string[];
}
const directory = new URL("../_fixtures/compiler-patch-successors-10923/", import.meta.url);
const capture = JSON.parse(fs.readFileSync(new URL("capture.json", directory), "utf8")) as {
  rows: Review[];
};
const hash = (value: string | Uint8Array) => createHash("sha256").update(value).digest("hex");
const fields = ["filename", "code", "css", "errors", "warnings", "script_lang", "macro_artifacts"];

function replaceOnce(code: string, previous: string, current: string): string {
  assert.equal(code.split(previous).length - 1, 1, "the reviewed source edit has one exact anchor");
  return code.replace(previous, current);
}

export function verifyCompilerPatchPair(
  id: CompilerPatchCase,
  previous: CompilerOutput,
  current: CompilerOutput,
): void {
  assert.deepEqual(Object.keys(previous).sort(), [...fields].sort());
  assert.deepEqual(Object.keys(current).sort(), [...fields].sort());
  let code = previous.code;
  if (id === "vue-router-clean") {
    code = replaceOnce(
      code,
      "(...args) => navigate && navigate(...args)",
      "(...args) => _unref(navigate) && _unref(navigate)(...args)",
    );
  } else if (id === "create-vue-broken") {
    code = replaceOnce(
      code,
      "setup(__props) {}",
      "setup(__props, { expose: __expose }) {\n    __expose();\n  }",
    );
  } else if (id === "vue-router-broken") {
    code = replaceOnce(
      code,
      "setup(__props) {",
      "setup(__props, { expose: __expose }) {\n    __expose();",
    );
  } else {
    assert.equal(id, "create-vue-clean");
  }
  assert.deepEqual(
    current,
    { ...previous, code },
    `${id}: every other complete packet byte/field is preserved`,
  );
}

export function compilerPatchSuccessor(id: CompilerPatchCase): {
  review: Review;
  previous: CompilerOutput;
  current: CompilerOutput;
  currentRaw: string;
} {
  const review = capture.rows.find((row) => row.id === id);
  assert.ok(review, `unregistered compiler patch successor ${id}`);
  assert.equal(hash(fs.readFileSync(new URL(review.input, directory))), review.inputSha256);
  const previousRaw = fs.readFileSync(new URL(review.previous, directory), "utf8");
  const currentRaw = fs.readFileSync(new URL(review.current, directory), "utf8");
  assert.equal(hash(previousRaw), review.previousSha256);
  assert.equal(hash(currentRaw), review.currentSha256);
  const previous = JSON.parse(previousRaw) as CompilerOutput;
  const current = JSON.parse(currentRaw) as CompilerOutput;
  assert.equal(hash(previous.code), review.previousCodeSha256);
  assert.equal(hash(current.code), review.currentCodeSha256);
  verifyCompilerPatchPair(id, previous, current);
  return { review, previous, current, currentRaw };
}

export function assertCompilerPatchSuccessor(
  id: CompilerPatchCase,
  actualRaw: string,
  original: { packetSha256?: string; codeSha256?: string; previousPacket?: CompilerOutput },
): void {
  const packet = compilerPatchSuccessor(id);
  if (original.packetSha256 !== undefined)
    assert.equal(
      packet.review.previousSha256,
      original.packetSha256,
      "the original immutable whole-packet pin stays exact",
    );
  if (original.codeSha256 !== undefined)
    assert.equal(
      packet.review.previousCodeSha256,
      original.codeSha256,
      "the original immutable code pin stays exact",
    );
  if (original.previousPacket !== undefined)
    assert.deepEqual(
      packet.previous,
      original.previousPacket,
      "the original full expected packet stays exact",
    );
  assert.equal(actualRaw, packet.currentRaw, `${id}: exact complete current compiler output`);
}
