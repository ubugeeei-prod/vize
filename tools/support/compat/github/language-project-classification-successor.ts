import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";

import { historicalClassificationConsumers } from "./language-project-classification-runtime.ts";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const CLASSIFICATION_PRODUCER = "npm/typescript-plugin/language-service.cjs";
export const CLASSIFICATION_SUCCESSOR_ID = "typechecker/language-project/classifications-11664";
export const CLASSIFICATION_CUSTODY_ROOT =
  "tests/_fixtures/tooling/typescript-classification-successor-11664/";
const beforeHash = "108028b9cb18a19510ac5b379885adab9e469e9c2b939842762c88a4548c8d57";
const afterHash = "b2ae3cd04fb0d71ab6f1f1ec05490f058f3e620c9f5e647f95654bfa407966af";
const custodyHash = "04628e3b129e38c7ee0cc721848a428fa34518993a4646ea7bef806827c68005";
const consumers = [
  "tests/typescript-plugin/classifications.test.cjs",
  "tests/typescript-plugin/mapped-language-service.test.cjs",
  "tests/tooling/language-project-classification-successor.test.ts",
  "npm/native/language-projection.js",
  ".github/workflows/language-project-fixtures.yml",
];
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
function requireCustody(value: unknown, message: string): asserts value {
  if (!value) throw new Error(`invalid classification successor: ${message}`);
}
function object(value: unknown, keys: string[]): Record<string, unknown> {
  requireCustody(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "object required",
  );
  requireCustody(
    JSON.stringify(Object.keys(value).sort()) === JSON.stringify(keys.sort()),
    "closed fields differ",
  );
  return value as Record<string, unknown>;
}
function pinned(read: ReadFile, value: unknown, path: string): Buffer {
  const row = object(value, ["path", "sha256", "bytes"]);
  requireCustody(row.path === path, "source or input path differs");
  const body = read("head", path);
  requireCustody(
    body && body.length === row.bytes && digest(body) === row.sha256,
    `missing or changed body: ${path}`,
  );
  return body;
}

/** Preserve the historical10740 producer and authenticate its one exact current successor. */
export function historicalClassificationProducer(
  rawRead: ReadFile,
  expectedHash: unknown,
  historicalRow: unknown,
): Buffer {
  const read = historicalClassificationConsumers(rawRead);
  requireCustody(expectedHash === beforeHash, "historical producer registration changed");
  const current = read("head", CLASSIFICATION_PRODUCER);
  requireCustody(current, "current producer missing");
  if (digest(current) === beforeHash) return current;
  requireCustody(digest(current) === afterHash, "unregistered current producer");

  const custody = read("head", CLASSIFICATION_CUSTODY_ROOT + "custody.json");
  requireCustody(custody && digest(custody) === custodyHash, "custody missing or changed");
  const record = object(JSON.parse(custody.toString("utf8")), [
    "schema",
    "id",
    "predecessor",
    "before",
    "after",
    "input",
    "consumers",
  ]);
  requireCustody(
    record.schema === "vize.languageProjectClassificationSuccessor.v1" &&
      record.id === CLASSIFICATION_SUCCESSOR_ID,
    "unregistered successor",
  );
  const predecessor = object(record.predecessor, ["id", "selector", "registrationSha256"]);
  requireCustody(
    predecessor.id === "typechecker/language-project/inherited-configuration-10740" &&
      predecessor.selector === "createMappedLanguageService",
    "historical predecessor differs",
  );
  requireCustody(
    digest(Buffer.from(JSON.stringify(historicalRow))) === predecessor.registrationSha256,
    "historical registration bytes differ",
  );
  const before = object(record.before, ["path", "sha256", "bytes", "gzip"]);
  requireCustody(
    before.path === CLASSIFICATION_PRODUCER &&
      before.sha256 === beforeHash &&
      Number.isSafeInteger(before.bytes) &&
      (before.bytes as number) > 0,
    "historical source identity differs",
  );
  const compressed = pinned(
    read,
    before.gzip,
    CLASSIFICATION_CUSTODY_ROOT + "language-service.before.cjs.gz",
  );
  const original = gunzipSync(compressed, { maxOutputLength: before.bytes as number });
  requireCustody(
    original.length === before.bytes && digest(original) === beforeHash,
    "historical source bytes differ",
  );
  const after = object(record.after, ["path", "sha256", "bytes"]);
  requireCustody(after.sha256 === afterHash, "current producer registration changed");
  pinned(read, after, CLASSIFICATION_PRODUCER);
  pinned(read, record.input, CLASSIFICATION_CUSTODY_ROOT + "input.json");
  requireCustody(
    Array.isArray(record.consumers) && record.consumers.length === consumers.length,
    "complete current consumer closure required",
  );
  for (const [index, consumer] of record.consumers.entries())
    pinned(read, consumer, consumers[index]!);
  return original;
}
