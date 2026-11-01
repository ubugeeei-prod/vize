/** Whole input seals for one finite helper-asset change; no path or diagnostic rewriting. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";

type Row = Record<string, unknown>;
export interface CandidateInputSeal {
  roots: string[];
  rows: Row[];
  sha256: string;
  files: number;
  bytes: number;
}
const hash = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const inside = (root: string, file: string) => file === root || file.startsWith(root + path.sep);

/** output.rs prints SHARED plus one LF; materialize_stubs adds only this optional owned prefix. */
export function materializedCandidateHelpers(
  debug: { before: Buffer; after: Buffer },
  observedBefore: { bytes: number; sha256: string },
) {
  const shared = (body: Buffer) => {
    assert(body.length > 1 && body.at(-1) === 10, "exact eprintln wrapper LF required");
    return body.subarray(0, -1); // Derive a separate body; original raw buffers remain unchanged.
  };
  const before = shared(debug.before),
    after = shared(debug.after);
  const wrappers = [Buffer.alloc(0), Buffer.from('/// <reference types="vue/jsx" />\n')];
  const matches = wrappers
    .map((prefix) => ({
      before: Buffer.concat([prefix, before]),
      after: Buffer.concat([prefix, after]),
      prefix,
    }))
    .filter(
      (row) =>
        row.before.length === observedBefore.bytes && hash(row.before) === observedBefore.sha256,
    );
  assert.equal(
    matches.length,
    1,
    "native helper must match one complete source-defined body wrapper",
  );
  return {
    before: matches[0].before,
    after: matches[0].after,
    wrapper: matches[0].prefix.length === 0 ? "plain-shared-preamble" : "exact-vue-jsx-reference",
    sourceLaw:
      "one printer LF is outside the materialized body; optional exact owned JSX reference only; no raw packet bytes rewritten",
  };
}

/** The two bodies are exact materialized bodies, with source-defined wrappers separately verified. */
export function verifyCandidateInputSeals(
  before: CandidateInputSeal,
  after: CandidateInputSeal,
  materializedRoot: string | null,
  helpers: { before: Buffer; after: Buffer } | null,
  expectApplied: boolean,
  additionalRuntimeRoot?: string,
) {
  assert.deepEqual(after.roots, before.roots, "whole potential-input roots changed");
  assert.equal(before.roots.length, materializedRoot === null ? 2 : additionalRuntimeRoot ? 4 : 3);
  assert(before.roots.every((root) => path.isAbsolute(root) && path.normalize(root) === root));
  assert.equal(new Set(before.roots).size, before.roots.length);
  if (materializedRoot !== null) assert.equal(before.roots[2], materializedRoot);
  if (additionalRuntimeRoot) assert.equal(before.roots[3], additionalRuntimeRoot);
  const recreatedRoots = [materializedRoot, additionalRuntimeRoot].filter(
    (root): root is string => root !== null && root !== undefined,
  );
  assert(!expectApplied || (materializedRoot !== null && helpers !== null));
  const rows = (seal: CandidateInputSeal) => {
    assert.deepEqual(Object.keys(seal).sort(), ["bytes", "files", "roots", "rows", "sha256"]);
    assert.equal(hash(JSON.stringify(seal.rows)), seal.sha256, "whole raw seal digest");
    const seen = new Set<string>();
    let files = 0,
      bytes = 0;
    const result = seal.rows.map((row) => {
      assert.equal(typeof row.path, "string");
      const file = row.path as string;
      assert(path.isAbsolute(file) && path.normalize(file) === file);
      assert(seal.roots.some((root) => inside(root, file)));
      assert(!seen.has(file), "duplicate potential input row");
      seen.add(file);
      if (row.kind !== "file") return row;
      assert(Number.isSafeInteger(row.bytes) && Number(row.bytes) >= 0);
      assert.match(String(row.sha256), /^[a-f0-9]{64}$/u);
      assert.equal(typeof row.mtimeNs, "string");
      files += 1;
      bytes += Number(row.bytes);
      if (!recreatedRoots.some((root) => inside(root, file))) return row;
      const { mtimeNs: _recreatedFileTime, ...rest } = row;
      return rest;
    });
    assert.equal(seal.files, files);
    assert.equal(seal.bytes, bytes);
    return result;
  };
  const left = rows(before),
    right = rows(after);
  const helperIndexes =
    materializedRoot === null
      ? []
      : left.flatMap((row, index) =>
          inside(materializedRoot, row.path as string) &&
          path.basename(row.path as string) === "__vize_helpers.d.ts"
            ? [index]
            : [],
        );
  assert(helperIndexes.length <= 1, "ambiguous materialized helper target");
  assert(!expectApplied || helperIndexes.length === 1, "candidate not applied to native program");
  let helperPath: string | null = null;
  const expected = [...left];
  if (helpers !== null && helperIndexes.length === 1) {
    const index = helperIndexes[0],
      old = left[index];
    assert.equal(old.kind, "file");
    assert.equal(old.bytes, helpers.before.length);
    assert.equal(old.sha256, hash(helpers.before), "debug/helper native whole body join");
    assert(!helpers.before.equals(helpers.after));
    expected[index] = { ...old, bytes: helpers.after.length, sha256: hash(helpers.after) };
    helperPath = old.path as string;
  }
  assert.deepEqual(right, expected, "unclassified whole native input/path/link/mode/body change");
  return {
    rows: left.length,
    beforeSealSha256: before.sha256,
    afterSealSha256: after.sha256,
    changedHelperPath: helperPath,
    scope:
      "all sealed rows; only recreated materialized file mtimes and one bound whole helper body differ; loaded-file vector unobserved",
  };
}
