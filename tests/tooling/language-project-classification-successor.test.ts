import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { gzipSync } from "node:zlib";
import {
  CLASSIFICATION_CUSTODY_ROOT as custodyRoot,
  CLASSIFICATION_PRODUCER as producer,
  CLASSIFICATION_SUCCESSOR_ID,
  historicalClassificationProducer as authenticateProducer,
} from "../../tools/support/compat/github/language-project-classification-successor.ts";

const root = path.resolve(import.meta.dirname, "../..");
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const beforeHash = "108028b9cb18a19510ac5b379885adab9e469e9c2b939842762c88a4548c8d57";
const read = (_side: "base" | "head", file: string) => {
  const source = path.join(root, file);
  return existsSync(source) ? readFileSync(source) : null;
};
const custodyPath = custodyRoot + "custody.json";
const record = JSON.parse(read("head", custodyPath)!.toString("utf8"));
const historicalRow = JSON.parse(
  read("head", "tests/_fixtures/differential/typechecker/manifest.json")!.toString(),
).languageProjectCases.find((row: any) => row.id === record.predecessor.id);
const historicalClassificationProducer = (
  reader: Parameters<typeof authenticateProducer>[0],
  hash: unknown,
) => authenticateProducer(reader, hash, historicalRow);

test("the historical source remains intact while its exact successor and complete consumers are authenticated", () => {
  assert.equal(record.id, CLASSIFICATION_SUCCESSOR_ID);
  const original = historicalClassificationProducer(read, beforeHash);
  assert.equal(digest(original), beforeHash);
  assert.notEqual(digest(read("head", producer)!), beforeHash);
  assert.match(original.toString("utf8"), /isSfc\(file\) \? \{ spans: \[\], endOfLineState: 0 \}/);
  const manifest = JSON.parse(
    read("head", "tests/_fixtures/differential/typechecker/manifest.json")!.toString(),
  );
  const old = manifest.languageProjectCases.find((row: any) => row.id === record.predecessor.id);
  assert.equal(old.tests.find((row: any) => row.source === producer).sha256, beforeHash);
  assert.throws(
    () => historicalClassificationProducer(read, "f".repeat(64)),
    /historical producer registration changed/,
  );
});

test("missing or changed custody, old gzip, input and every actual consumer are rejected", () => {
  const files = [
    producer,
    custodyPath,
    record.before.gzip.path,
    record.input.path,
    ...record.consumers.map((row: any) => row.path),
  ];
  for (const target of files)
    for (const missing of [true, false]) {
      const altered = (side: "base" | "head", file: string) => {
        const body = read(side, file);
        return file === target
          ? missing
            ? null
            : Buffer.concat([body!, Buffer.from("\n")])
          : body;
      };
      assert.throws(
        () => historicalClassificationProducer(altered, beforeHash),
        /invalid classification successor/,
        `${target}:${missing}`,
      );
    }
});

test("unknown, duplicate, repinned and future source successors cannot reuse historical authority", () => {
  for (const mutate of [
    (row: any) => {
      row.id = "typechecker/language-project/unknown";
    },
    (row: any) => {
      row.id = [CLASSIFICATION_SUCCESSOR_ID, CLASSIFICATION_SUCCESSOR_ID];
    },
    (row: any) => {
      row.predecessor.selector = "unrelatedProducer";
    },
    (row: any) => {
      row.before.path = "other.cjs";
    },
    (row: any) => {
      row.before.sha256 = "f".repeat(64);
    },
    (row: any) => {
      row.after.sha256 = "f".repeat(64);
    },
    (row: any) => {
      row.consumers.push(row.consumers[0]);
    },
    (row: any) => {
      row.consumers.pop();
    },
    (row: any) => {
      row.consumers[0].sha256 = "f".repeat(64);
    },
    (row: any) => {
      row.extra = "unregistered";
    },
  ]) {
    const changed = structuredClone(record);
    mutate(changed);
    assert.throws(
      () =>
        historicalClassificationProducer(
          (side, file) =>
            file === custodyPath ? Buffer.from(JSON.stringify(changed)) : read(side, file),
          beforeHash,
        ),
      /custody missing or changed/,
    );
  }
  const future = Buffer.concat([read("head", producer)!, Buffer.from("\n")]);
  const repinned = structuredClone(record);
  repinned.after.sha256 = digest(future);
  repinned.after.bytes = future.length;
  assert.throws(
    () =>
      historicalClassificationProducer(
        (side, file) =>
          file === producer
            ? future
            : file === custodyPath
              ? Buffer.from(JSON.stringify(repinned))
              : read(side, file),
        beforeHash,
      ),
    /unregistered current producer/,
  );
  const replacedOriginal = structuredClone(record);
  const invalidGzip = gzipSync(Buffer.from("a different historical body"));
  replacedOriginal.before.gzip.sha256 = digest(invalidGzip);
  replacedOriginal.before.gzip.bytes = invalidGzip.length;
  assert.throws(
    () =>
      historicalClassificationProducer(
        (side, file) =>
          file === record.before.gzip.path
            ? invalidGzip
            : file === custodyPath
              ? Buffer.from(JSON.stringify(replacedOriginal))
              : read(side, file),
        beforeHash,
      ),
    /custody missing or changed/,
  );
});

test("the pinned classification input replays complete genuine TypeScript packets through the current producer", () => {
  historicalClassificationProducer(read, beforeHash);
  const require = createRequire(import.meta.url);
  const {
    createMappedLanguageService,
  } = require("../../npm/typescript-plugin/language-service.cjs");
  const helper = require("../../npm/native/language-projection.js");
  const fixture = JSON.parse(read("head", record.input.path)!.toString());
  assert.equal(fixture.schema, "vize.typescriptClassificationFixture.v1");
  assert.equal(fixture.issue, 11664);
  assert.equal(fixture.typescript, "6.0.3");
  assert.deepEqual(
    fixture.cases.map((row: any) => row.id),
    ["lf-before-edit", "lf-after-edit", "crlf-before-edit", "crlf-after-edit"],
  );
  let packets = 0;
  for (const input of fixture.cases)
    for (const request of input.requests) {
      let calls = 0;
      const projection = helper.toUtf16LanguageProjection(
        input.source,
        JSON.stringify(input.projection),
      );
      const backend = {
        [request.method]: (file: string, range: unknown, format: string) => {
          calls++;
          assert.equal(file, input.file);
          assert.deepEqual(range, request.generatedRange);
          assert.equal(format, request.format);
          return request.raw;
        },
      };
      const service = createMappedLanguageService({ EndOfLineState: { None: 0 } }, backend, {
        engine: helper,
        get: () => ({ source: input.source, projection }),
      });
      assert.deepEqual(
        service[request.method](input.file, request.authoredRange, request.format),
        request.expected,
      );
      assert.equal(calls, 1);
      assert.ok(request.expected.spans.length > 0);
      packets++;
    }
  assert.equal(packets, 8);
});

test("the entire historical registration keeps its original inputs, witness, targets and producer identities", () => {
  for (const mutate of [
    (row: any) => {
      row.inputs[0].sha256 = "f".repeat(64);
    },
    (row: any) => {
      row.witness.run++;
    },
    (row: any) => {
      row.targets.reverse();
    },
    (row: any) => {
      row.tests[0].sha256 = "f".repeat(64);
    },
    (row: any) => {
      row.adapter = "unregistered";
    },
  ]) {
    const changed = structuredClone(historicalRow);
    mutate(changed);
    assert.throws(
      () => authenticateProducer(read, beforeHash, changed),
      /historical registration bytes differ/,
    );
  }
});
