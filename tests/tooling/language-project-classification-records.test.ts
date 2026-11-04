import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { gzipSync } from "node:zlib";
import {
  CLASSIFICATION_CUSTODY_ROOT,
  historicalClassificationProducer,
  validateClassificationSuccessorRecord,
} from "../../tools/support/compat/github/language-project-classification-successor.ts";
import {
  CLASSIFICATION_RUNTIME_ROOT,
  authenticateClassificationRuntime,
  validateClassificationRuntimeRecord,
} from "../../tools/support/compat/github/language-project-classification-runtime.ts";

type ReadFile = (side: "base" | "head", file: string) => Buffer | null;
const root = path.resolve(import.meta.dirname, "../..");
const read: ReadFile = (_side, file) => {
  const source = path.join(root, file);
  return fs.existsSync(source) ? fs.readFileSync(source) : null;
};
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const firstPath = CLASSIFICATION_CUSTODY_ROOT + "custody.json";
const runtimePath = CLASSIFICATION_RUNTIME_ROOT + "custody.json";
const first = JSON.parse(read("head", firstPath)!.toString());
const runtime = JSON.parse(read("head", runtimePath)!.toString());
const historicalRow = JSON.parse(
  read("head", "tests/_fixtures/differential/typechecker/manifest.json")!.toString(),
).languageProjectCases.find((row: any) => row.id === first.predecessor.id);
const override =
  (file: string, body: Buffer | null): ReadFile =>
  (side, source) =>
    side === "head" && source === file ? body : read(side, source);

test("post-integrity validators consume the genuine complete records without weakening authenticated intake", () => {
  assert.deepEqual(
    validateClassificationSuccessorRecord(read, first, historicalRow),
    historicalClassificationProducer(read, first.before.sha256, historicalRow),
  );
  assert.deepEqual(
    validateClassificationRuntimeRecord(read, runtime),
    authenticateClassificationRuntime(read),
  );
  // Re-serialization is deliberately invalid at the authenticated boundary,
  // even when the parsed record has exactly the same meaning.
  assert.throws(
    () =>
      historicalClassificationProducer(
        override(firstPath, Buffer.from(JSON.stringify(first))),
        first.before.sha256,
        historicalRow,
      ),
    { message: "invalid classification successor: custody missing or changed" },
  );
  assert.throws(
    () =>
      authenticateClassificationRuntime(
        override(runtimePath, Buffer.from(JSON.stringify(runtime))),
      ),
    { message: "invalid classification successor runtime: custody missing or changed" },
  );
});

test("first successor mutations reach each specific post-hash record validation branch", () => {
  const cases: Array<[string, (row: any) => void, string]> = [
    [
      "closed root fields",
      (row) => {
        row.extra = true;
      },
      "closed fields differ",
    ],
    [
      "schema",
      (row) => {
        row.schema = "future";
      },
      "unregistered successor",
    ],
    [
      "unknown identity",
      (row) => {
        row.id = "unknown";
      },
      "unregistered successor",
    ],
    [
      "duplicate identity",
      (row) => {
        row.id = [row.id, row.id];
      },
      "unregistered successor",
    ],
    [
      "predecessor fields",
      (row) => {
        row.predecessor.extra = true;
      },
      "closed fields differ",
    ],
    [
      "predecessor identity",
      (row) => {
        row.predecessor.id = "unknown";
      },
      "historical predecessor differs",
    ],
    [
      "predecessor selector",
      (row) => {
        row.predecessor.selector = "unrelated";
      },
      "historical predecessor differs",
    ],
    [
      "original registration",
      (row) => {
        row.predecessor.registrationSha256 = "f".repeat(64);
      },
      "historical registration bytes differ",
    ],
    [
      "historical path",
      (row) => {
        row.before.path = "other.cjs";
      },
      "historical source identity differs",
    ],
    [
      "historical hash",
      (row) => {
        row.before.sha256 = "f".repeat(64);
      },
      "historical source identity differs",
    ],
    [
      "historical length",
      (row) => {
        row.before.bytes = 0;
      },
      "historical source identity differs",
    ],
    [
      "historical body length",
      (row) => {
        row.before.bytes++;
      },
      "historical source bytes differ",
    ],
    [
      "gzip path",
      (row) => {
        row.before.gzip.path = "other.gz";
      },
      "source or input path differs",
    ],
    [
      "gzip hash",
      (row) => {
        row.before.gzip.sha256 = "f".repeat(64);
      },
      `missing or changed body: ${first.before.gzip.path}`,
    ],
    [
      "current producer identity",
      (row) => {
        row.after.sha256 = "f".repeat(64);
      },
      "current producer registration changed",
    ],
    [
      "current producer path",
      (row) => {
        row.after.path = "other.cjs";
      },
      "source or input path differs",
    ],
    [
      "current producer bytes",
      (row) => {
        row.after.bytes++;
      },
      `missing or changed body: ${first.after.path}`,
    ],
    [
      "input path",
      (row) => {
        row.input.path = "other.json";
      },
      "source or input path differs",
    ],
    [
      "input hash",
      (row) => {
        row.input.sha256 = "f".repeat(64);
      },
      `missing or changed body: ${first.input.path}`,
    ],
    [
      "missing consumer",
      (row) => {
        row.consumers.pop();
      },
      "complete current consumer closure required",
    ],
    [
      "duplicate consumer",
      (row) => {
        row.consumers.push(row.consumers[0]);
      },
      "complete current consumer closure required",
    ],
    [
      "consumer order",
      (row) => {
        row.consumers.reverse();
      },
      "source or input path differs",
    ],
    [
      "consumer hash",
      (row) => {
        row.consumers[0].sha256 = "f".repeat(64);
      },
      `missing or changed body: ${first.consumers[0].path}`,
    ],
  ];
  for (const [label, mutate, message] of cases) {
    const record = structuredClone(first);
    mutate(record);
    assert.throws(
      () => validateClassificationSuccessorRecord(read, record, historicalRow),
      { message: `invalid classification successor: ${message}` },
      label,
    );
  }
});

test("first successor gzip declarations can pass their body pin and still fail the whole historical source check", () => {
  const compressed = gzipSync(Buffer.from("a different historical producer"));
  const record = structuredClone(first);
  record.before.gzip.sha256 = digest(compressed);
  record.before.gzip.bytes = compressed.length;
  assert.throws(
    () =>
      validateClassificationSuccessorRecord(
        override(record.before.gzip.path, compressed),
        record,
        historicalRow,
      ),
    { message: "invalid classification successor: historical source bytes differ" },
  );
});

test("runtime mutations reach specific schema, transition, predecessor, input and consumer branches after the custody gate", () => {
  const changedBody = (file: string) =>
    `unreviewed producer registration (missing or changed closure body): ${file}`;
  const cases: Array<[string, (row: any) => void, string]> = [
    [
      "closed fields",
      (row) => {
        row.extra = true;
      },
      "closed fields differ",
    ],
    [
      "schema",
      (row) => {
        row.schema = "future";
      },
      "unregistered successor",
    ],
    [
      "identity",
      (row) => {
        row.id = "unknown";
      },
      "unregistered successor",
    ],
    [
      "predecessor path",
      (row) => {
        row.predecessor.path = "other.json";
      },
      "closure path differs",
    ],
    [
      "predecessor pin",
      (row) => {
        row.predecessor.sha256 = "f".repeat(64);
      },
      changedBody(runtime.predecessor.path),
    ],
    [
      "missing transition",
      (row) => {
        row.transitions.pop();
      },
      "complete transition closure required",
    ],
    [
      "duplicate transition",
      (row) => {
        row.transitions.push(row.transitions[0]);
      },
      "complete transition closure required",
    ],
    [
      "transition fields",
      (row) => {
        row.transitions[0].extra = true;
      },
      "closed fields differ",
    ],
    [
      "historical path",
      (row) => {
        row.transitions[0].before.path = "other.ts";
      },
      "historical path differs",
    ],
    [
      "historical length",
      (row) => {
        row.transitions[0].before.bytes = 0;
      },
      "historical path differs",
    ],
    [
      "historical hash",
      (row) => {
        row.transitions[0].before.sha256 = "f".repeat(64);
      },
      "historical bytes differ",
    ],
    [
      "gzip path",
      (row) => {
        row.transitions[0].gzip.path = "other.gz";
      },
      "closure path differs",
    ],
    [
      "gzip hash",
      (row) => {
        row.transitions[0].gzip.sha256 = "f".repeat(64);
      },
      changedBody(runtime.transitions[0].gzip.path),
    ],
    [
      "current path",
      (row) => {
        row.transitions[0].after.path = "other.ts";
      },
      "closure path differs",
    ],
    [
      "current pin",
      (row) => {
        row.transitions[0].after.sha256 = "f".repeat(64);
      },
      changedBody(runtime.transitions[0].after.path),
    ],
    [
      "input path",
      (row) => {
        row.input.path = "other.json";
      },
      "closure path differs",
    ],
    [
      "input pin",
      (row) => {
        row.input.sha256 = "f".repeat(64);
      },
      changedBody(runtime.input.path),
    ],
    [
      "missing consumer",
      (row) => {
        row.consumers.pop();
      },
      "complete current consumer closure required",
    ],
    [
      "duplicate consumer",
      (row) => {
        row.consumers.push(row.consumers[0]);
      },
      "complete current consumer closure required",
    ],
    [
      "consumer order",
      (row) => {
        row.consumers.reverse();
      },
      "closure path differs",
    ],
    [
      "consumer pin",
      (row) => {
        row.consumers[0].sha256 = "f".repeat(64);
      },
      changedBody(runtime.consumers[0].path),
    ],
  ];
  for (const [label, mutate, message] of cases) {
    const record = structuredClone(runtime);
    mutate(record);
    assert.throws(
      () => validateClassificationRuntimeRecord(read, record),
      { message: `invalid classification successor runtime: ${message}` },
      label,
    );
  }
});

test("runtime predecessor and gzip body pins are distinct from their immutable historical identities", () => {
  const original = Buffer.from("a different predecessor");
  const predecessor = structuredClone(runtime);
  predecessor.predecessor.sha256 = digest(original);
  predecessor.predecessor.bytes = original.length;
  assert.throws(
    () =>
      validateClassificationRuntimeRecord(
        override(predecessor.predecessor.path, original),
        predecessor,
      ),
    { message: "invalid classification successor runtime: predecessor changed" },
  );
  const compressed = gzipSync(Buffer.from("a different original consumer"));
  const gzip = structuredClone(runtime);
  gzip.transitions[0].gzip.sha256 = digest(compressed);
  gzip.transitions[0].gzip.bytes = compressed.length;
  assert.throws(
    () =>
      validateClassificationRuntimeRecord(
        override(gzip.transitions[0].gzip.path, compressed),
        gzip,
      ),
    { message: "invalid classification successor runtime: historical bytes differ" },
  );
});
