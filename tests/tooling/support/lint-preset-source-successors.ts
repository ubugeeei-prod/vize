// Exact source-retention inverses for the frozen lint-preset audits.
// These helpers never adapt a compiled, runtime or performance acceptance gate.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

interface Identity {
  bytes: number;
  sha256: string;
}
interface TextHunk {
  before: string;
  after: string;
}
interface OffsetHunk extends TextHunk {
  beforeOffset: number;
  afterOffset: number;
}
interface Successor<Hunk = TextHunk> {
  path: string;
  before: Identity;
  after: Identity;
  hunks: Hunk[];
}
interface CurrentSources {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  predecessor: Identity & { path: string };
  sources: Successor<OffsetHunk>[];
}
interface ReaderOptions {
  root: string;
  liveNextest: string;
  reviewedNextest: string;
  currentSources: CurrentSources;
  presetSuccessors: Successor[];
}
export const sha = (body: Uint8Array): string => createHash("sha256").update(body).digest("hex");
export const jsonBytes = (value: unknown): Buffer =>
  Buffer.from(JSON.stringify(value, null, 2) + "\n");

export function identity(body: Buffer, expected: Identity): void {
  assert.equal(body.length, expected.bytes);
  assert.equal(sha(body), expected.sha256);
}

export function replaceBytes(
  body: Buffer,
  offset: number,
  expected: Buffer,
  replacement: Buffer,
): Buffer {
  assert(Number.isSafeInteger(offset) && offset >= 0 && offset + expected.length <= body.length);
  assert.deepEqual(body.subarray(offset, offset + expected.length), expected);
  return Buffer.concat([
    body.subarray(0, offset),
    replacement,
    body.subarray(offset + expected.length),
  ]);
}

export function inverseCurrentSource(row: Successor<OffsetHunk>, current: unknown): Buffer {
  assert(Buffer.isBuffer(current), "the current source body is required");
  identity(current, row.after);
  let before = current;
  for (const hunk of [...row.hunks].reverse())
    before = replaceBytes(
      before,
      hunk.afterOffset,
      Buffer.from(hunk.after),
      Buffer.from(hunk.before),
    );
  identity(before, row.before);
  let forward = before;
  for (const hunk of [...row.hunks].reverse())
    forward = replaceBytes(
      forward,
      hunk.beforeOffset,
      Buffer.from(hunk.before),
      Buffer.from(hunk.after),
    );
  assert.deepEqual(forward, current);
  return before;
}

export function inverse(row: Successor, body: Buffer): Buffer {
  identity(body, row.after);
  let text = body.toString("utf8");
  for (const hunk of row.hunks) {
    assert.notEqual(hunk.before, hunk.after);
    assert.equal(text.split(hunk.after).length, 2, row.path);
    text = text.replace(hunk.after, hunk.before);
  }
  const before = Buffer.from(text);
  identity(before, row.before);
  return before;
}

export function canonical(body: Buffer): ReturnType<typeof JSON.parse> {
  const value = JSON.parse(body.toString());
  assert.deepEqual(body, jsonBytes(value)); // Never discard unrelated whitespace/edits.
  return value;
}

export function loadCurrentSourceSuccessors(
  root: string,
  directory: string,
  presetSuccessorBytes: Buffer,
  presetSuccessors: Successor[],
): CurrentSources {
  const currentSourceBytes = fs.readFileSync(
    path.join(root, directory, "current-source-successors-20261105.json"),
  );
  assert.equal(
    sha(currentSourceBytes),
    "88fd2253b4a5627a642c05de90faa2659b86ad25aee95dea257216489fdbf01b",
  );
  const currentSources = JSON.parse(currentSourceBytes.toString("utf8")) as CurrentSources;
  assert.equal(currentSources.schema, "vize.lint.preset-warning-current-source-successors.v1");
  assert.deepEqual(currentSources.credit, { compiled: false, native: false, performance: false });
  assert.equal(currentSources.predecessor.path, `${directory}/preset-successors.json`);
  identity(presetSuccessorBytes, currentSources.predecessor);
  assert.deepEqual(
    currentSources.sources.map((row) => row.path),
    [
      "crates/vize_patina/src/linter/script_rules/registry/rules.rs",
      "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap",
      "tests/_fixtures/patina-eslint-vue-rule-map.json",
      "crates/vize_patina/src/rule/happy_path.rs",
      "crates/vize_patina/src/rules/script/define_macros_order.rs",
    ],
  );
  for (const row of currentSources.sources)
    assert.deepEqual(
      row.before,
      presetSuccessors.filter((previous) => previous.path === row.path).at(-1)!.after,
    );
  return currentSources;
}

export function createSourceReader(options: ReaderOptions): (name: string) => Buffer {
  const { root, liveNextest, reviewedNextest, currentSources, presetSuccessors } = options;
  return (name: string): Buffer => {
    const body = fs.readFileSync(path.join(root, name === liveNextest ? reviewedNextest : name));
    const current = currentSources.sources.find((row) => row.path === name);
    return presetSuccessors.reduceRight(
      (body, row) => (row.path === name ? inverse(row, body) : body),
      current ? inverseCurrentSource(current, body) : body,
    );
  };
}

// Restore only the reviewed class-default edits for the frozen earlier audit.
export function createClassSourceInverse(
  staticClassRegistration: Successor,
  staticClassMembership: Pick<Successor, "path" | "before" | "after">,
): (name: string, body: Buffer) => Buffer {
  return (name: string, body: Buffer): Buffer => {
    if (name === staticClassRegistration.path) return inverse(staticClassRegistration, body);
    if (name !== staticClassMembership.path) return body;
    identity(body, staticClassMembership.after);
    const text = body.toString();
    const at = text.indexOf("{");
    const table = canonical(Buffer.from(text.slice(at)));
    const rule = "html/no-duplicate-class";
    for (const preset of ["happy_path", "ecosystem", "opinionated", "nuxt"]) {
      const names = table[preset];
      assert.equal(names.filter((name) => name === rule).length, 1);
      assert.equal(names[names.indexOf("html/require-datetime") + 1], rule);
      names.splice(names.indexOf(rule), 1);
      if (preset === "opinionated" || preset === "nuxt")
        names.splice(names.indexOf("html/no-dupe-style-properties"), 0, rule);
    }
    const before = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
    identity(before, staticClassMembership.before);
    return before;
  };
}
