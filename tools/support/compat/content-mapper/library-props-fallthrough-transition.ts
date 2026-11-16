import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { certifiedTransition } from "./canonical-helper-transition.ts";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
type FileTransition = { path: string; before: Identity; after: Identity; hunks: Hunk[] };
type Record = {
  schema: string;
  issue: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: FileTransition[];
  compiledPreamble: { hunks: Hunk[] };
};

/** Inverse of the #13021 library fallthrough tail; mirrors payload_update/library_props_fallthrough.rs. */
const recordBytes = readFileSync(
  new URL(
    "../../../../tests/_fixtures/tooling/library-props-fallthrough-13021.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "2f7eececab23cb5dd4db6640993ec4b130396abd31f203a3d08f1240542ca30f",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.libraryPropsFallthroughSource.v1");
assert.equal(record.issue, 13021);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The pre-#13021 bytes of a recorded asset; other images pass through. */
export function beforeLibraryPropsFallthrough(file: string, current: string): string {
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

/** The one compiled-preamble hunk: the library branch of `$PropChecker`. */
function preambleHunk(): Hunk {
  const [hunk, ...rest] = record.compiledPreamble.hunks;
  assert.ok(hunk && rest.length === 0, "one library fallthrough preamble hunk");
  return hunk;
}

/** Advance a composed shared preamble across the #13021 library fallthrough tail. */
export function applyLibraryPropsFallthroughIn(previous: string): string {
  const { before, after } = preambleHunk();
  assert.equal(previous.split(before).length, 2, "closed library checker predecessor line");
  return previous.replace(before, () => after);
}

/** Inverse of {@link applyLibraryPropsFallthroughIn}. */
export function beforeLibraryPropsFallthroughIn(current: string): string {
  const { before, after } = preambleHunk();
  assert.equal(current.split(after).length, 2, "open library checker current line");
  return current.replace(after, () => before);
}

/**
 * The embedded `$PropChecker` library branch before and after #13021. The
 * fragments stop at the next `(props: ` so the content mapper's public helper
 * names (`ComponentPropsForTemplate`) cannot hide them.
 */
export const previousLibraryPropChecker =
  "=> void : $IsLibraryGenericExport<C> extends true ? C : (props: ";
export const currentLibraryPropChecker =
  "=> void : $IsLibraryGenericExport<C> extends true ? [keyof { [K in keyof T as string extends K ? never : K]: 0 }] extends [never] ? C | ((props: T) => void) : C : (props: ";

/** The embedded library-branch transition, when a packet crosses it. */
export function certifyLibraryPropChecker(before: Buffer, after: Buffer) {
  return certifiedTransition(
    before,
    after,
    previousLibraryPropChecker,
    currentLibraryPropChecker,
    "embedded library-generic fallthrough tail",
  );
}
