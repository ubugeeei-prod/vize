/** Forward and invert the member-inferred emit factory family in preamble text. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

type Identity = { bytes: number; sha256: string };
type Edit = { before: string; after: string };
type Entry = { path: string; before: Identity; after: Identity; edits: Edit[] };

const recordUrl = new URL(
  "../../../../tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/emit-member-infer-update-20261108.json",
  import.meta.url,
);
const recordSha256 = "4b155209430258da65b41f472c84e3583d1ae5f4e551d01640f624fe20473424";
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

function record(): { files: Entry[]; preamble: Entry } {
  const raw = readFileSync(recordUrl);
  assert.equal(sha256(raw), recordSha256, "member-inferred emit factory source record");
  return JSON.parse(raw.toString("utf8")) as { files: Entry[]; preamble: Entry };
}

function replaceEach(text: string, edits: Edit[], direction: "forward" | "inverse"): string {
  for (const edit of edits) {
    const [from, to] =
      direction === "forward" ? [edit.before, edit.after] : [edit.after, edit.before];
    assert.equal(text.split(from).length, 2, `member-inferred emit factory edit is unique`);
    assert.equal(text.includes(to), false, `member-inferred emit factory edit is absent`);
    text = text.replace(from, to);
  }
  return text;
}

/** Restore the single-call family in a preamble that carries the union gate. */
export function beforeEmitMemberInferIn(current: string): string {
  return replaceEach(current, record().preamble.edits, "inverse");
}

/** Add the union gate to a preamble that carries the single-call family. */
export function applyEmitMemberInferIn(previous: string): string {
  return replaceEach(previous, record().preamble.edits, "forward");
}

/** The recorded predecessor of a canonical asset; unrecorded files pass through. */
export function emitMemberInferAssetBefore(file: string, current: string): string {
  const entry = record().files.find((entry) => entry.path === file);
  if (!entry) return current;
  const counts = entry.edits.map((edit) => current.split(edit.after).length - 1);
  if (counts.every((count) => count === 0)) return current;
  assert.ok(counts.every((count) => count === 1));
  return replaceEach(current, entry.edits, "inverse");
}
