import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

interface DocumentSuccession {
  path: string;
  originalSha256: string;
  successorSha256: string;
  patches: { offset: number; before: string; after: string }[];
}

const fixture = JSON.parse(
  readFileSync(
    resolve(import.meta.dirname, "../../tests/_fixtures/docs-config-editorial-succession.json"),
    "utf8",
  ),
) as {
  documents: DocumentSuccession[];
  witnesses: { case: string; previous: string; after: string }[];
};
const sha = (source: string) => createHash("sha256").update(source).digest("hex");

/** Undo only reviewed configuration edits; the original editorial corpus remains immutable. */
export function originalEditorialSource(path: string, source: string) {
  const entry = fixture.documents.find((document) => document.path === path);
  if (!entry) return source;
  assert.equal(sha(source), entry.successorSha256, `${path}: complete configuration successor`);
  let original = source;
  for (const patch of [...entry.patches].reverse()) {
    assert.equal(
      original.slice(patch.offset, patch.offset + patch.after.length),
      patch.after,
      `${path}: complete reviewed edit`,
    );
    original =
      original.slice(0, patch.offset) +
      patch.before +
      original.slice(patch.offset + patch.after.length);
  }
  assert.equal(sha(original), entry.originalSha256, `${path}: every original editorial byte`);
  return original;
}

export function currentEditorialWitness<T extends { case: string; after: string }>(witness: T): T {
  const successor = fixture.witnesses.find((entry) => entry.case === witness.case);
  if (!successor) return witness;
  assert.equal(witness.after, successor.previous, `${witness.case}: original whole witness`);
  return { ...witness, after: successor.after };
}
