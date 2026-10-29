/** Load the unchanged, pinned public admission generator; validate all runtime return values. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

export type ScaleOptions = {
  sfcs: number;
  sfcBytes: number;
  graphNodes: number;
  shape: "chain" | "diamond" | "scc" | "crosspkg";
  typeWidth: number;
  typeDepth: number;
  stage: "valid" | "invalid" | "poisoned";
  allowLarge?: boolean;
};
export type ScaleProbe = {
  file: string;
  before: string;
  after: string;
  token: string;
  code: number;
};
export type Expected = { file: string; line: number; column: number; code: number };
type Entry = { file: string; bytes: number; sha256: string; lines: number };
type Sfc = { file: string; rows: number; bytes: number; lineEndings: string };
export type Corpus = Record<string, unknown> & {
  options: ScaleOptions;
  sfcs: Sfc[];
  expected: Expected[];
  manifest: Entry[];
  sourceBytes: number;
  cold: string;
};
type GeneratedSfc = {
  file: string;
  source: string;
  probes: ScaleProbe[];
  rows: number;
  lineEndings: string;
};
type GeneratedGraph = { files: Record<string, string>; probes: ScaleProbe[] };
type Mutation = Omit<ScaleProbe, "file"> & { name: string };
type Mutated = { source: string; line: number; column: number; code: number };
export type AdmissionAPI = {
  prepareAdmissionScale: (project: string, provider: string, options: ScaleOptions) => Corpus;
  scaleSfc: (index: number, options: ScaleOptions) => GeneratedSfc;
  scaleGraph: (options: ScaleOptions) => GeneratedGraph;
  mutatedAdmission: (source: string, mutation: Mutation) => Mutated;
};
type RecordValue = Record<string, unknown>;
function record(value: unknown): value is RecordValue {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function text(value: unknown): value is string {
  return typeof value === "string";
}
function integer(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}
function probe(value: unknown): value is ScaleProbe {
  return (
    record(value) &&
    ["file", "before", "after", "token"].every((key) => text(value[key])) &&
    integer(value.code)
  );
}
function expected(value: unknown): value is Expected {
  return (
    record(value) &&
    text(value.file) &&
    integer(value.line) &&
    value.line > 0 &&
    integer(value.column) &&
    value.column > 0 &&
    integer(value.code)
  );
}
function entry(value: unknown): value is Entry {
  return (
    record(value) &&
    text(value.file) &&
    integer(value.bytes) &&
    text(value.sha256) &&
    /^[a-f0-9]{64}$/u.test(value.sha256) &&
    integer(value.lines)
  );
}
function sfc(value: unknown): value is Sfc {
  return (
    record(value) &&
    text(value.file) &&
    integer(value.bytes) &&
    integer(value.rows) &&
    text(value.lineEndings)
  );
}
function options(value: unknown): value is ScaleOptions {
  return (
    record(value) &&
    ["sfcs", "sfcBytes", "graphNodes", "typeWidth", "typeDepth"].every((key) =>
      integer(value[key]),
    ) &&
    text(value.shape) &&
    ["chain", "diamond", "scc", "crosspkg"].includes(value.shape) &&
    text(value.stage) &&
    ["valid", "invalid", "poisoned"].includes(value.stage) &&
    (value.allowLarge === undefined || typeof value.allowLarge === "boolean")
  );
}
function list<T>(value: unknown, check: (item: unknown) => item is T): value is T[] {
  return Array.isArray(value) && value.every(check);
}
function stringMap(value: unknown): value is Record<string, string> {
  return record(value) && Object.values(value).every(text);
}
function corpus(value: unknown): value is Corpus {
  return (
    record(value) &&
    options(value.options) &&
    list(value.sfcs, sfc) &&
    list(value.expected, expected) &&
    list(value.manifest, entry) &&
    integer(value.sourceBytes) &&
    text(value.cold)
  );
}
function generatedSfc(value: unknown): value is GeneratedSfc {
  return (
    record(value) &&
    text(value.file) &&
    text(value.source) &&
    list(value.probes, probe) &&
    integer(value.rows) &&
    text(value.lineEndings)
  );
}
function generatedGraph(value: unknown): value is GeneratedGraph {
  return record(value) && stringMap(value.files) && list(value.probes, probe);
}
function mutated(value: unknown): value is Mutated {
  return (
    record(value) &&
    text(value.source) &&
    integer(value.line) &&
    value.line > 0 &&
    integer(value.column) &&
    value.column > 0 &&
    integer(value.code)
  );
}
function invoke(owner: unknown, name: string, args: unknown[]): unknown {
  assert.ok(record(owner));
  const fn = owner[name];
  assert.equal(typeof fn, "function");
  assert.ok(typeof fn === "function");
  const value: unknown = Reflect.apply(fn, owner, args);
  return value;
}
const files: Record<string, string> = {
  "tools/benchmarks/scripts/type-admission-scale.ts": "8e1f01bd22aed6b3de52a43314752e66c853842a",
  "tools/benchmarks/scripts/type-admission-scale-types.ts":
    "df321ad8246ff3f08a09639485a9b91f9653d585",
  "tools/benchmarks/scripts/type-admission-scale-sfc.ts":
    "f3616b6e37bdae2814530d31999a6d9fad475f46",
  "tools/benchmarks/scripts/type-admission-scale-graph.ts":
    "516e077baa7f68b299cca81057d557929cabafef",
  "tests/tooling/support/type-admission-fixtures.ts": "8a6b1b2aa19bda0be1b89cd0cf38d4d09c5bacd9",
  "tests/tooling/support/type-admission-mutation.ts": "2832996fd33942538c36d7de2438b345d9ebed6a",
};
export async function loadAdmissionAPI(
  driver: string,
): Promise<{ api: AdmissionAPI; identities: { file: string; blob: string; bytes: number }[] }> {
  const identities = [];
  for (const [file, expectedBlob] of Object.entries(files)) {
    const bytes = fs.readFileSync(path.join(driver, file));
    const blob = createHash("sha1")
      .update("blob " + bytes.length + "\0")
      .update(bytes)
      .digest("hex");
    assert.equal(blob, expectedBlob, "the independent generator remains frozen");
    identities.push({ file, blob, bytes: bytes.length });
  }
  const load = async (file: string): Promise<unknown> => {
    const value: unknown = await import(pathToFileURL(path.join(driver, file)).href);
    return value;
  };
  const prepare = await load("tools/benchmarks/scripts/type-admission-scale.ts");
  const makeSfc = await load("tools/benchmarks/scripts/type-admission-scale-sfc.ts");
  const makeGraph = await load("tools/benchmarks/scripts/type-admission-scale-graph.ts");
  const mutate = await load("tests/tooling/support/type-admission-mutation.ts");
  const api: AdmissionAPI = {
    prepareAdmissionScale: (...args) => {
      const value = invoke(prepare, "prepareAdmissionScale", args);
      assert.ok(corpus(value));
      return value;
    },
    scaleSfc: (...args) => {
      const value = invoke(makeSfc, "scaleSfc", args);
      assert.ok(generatedSfc(value));
      return value;
    },
    scaleGraph: (...args) => {
      const value = invoke(makeGraph, "scaleGraph", args);
      assert.ok(generatedGraph(value));
      return value;
    },
    mutatedAdmission: (...args) => {
      const value = invoke(mutate, "mutatedAdmission", args);
      assert.ok(mutated(value));
      return value;
    },
  };
  return { api, identities };
}
