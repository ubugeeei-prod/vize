import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeWithDefaultsUndefined } from "./with-defaults-undefined-transition.ts";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
type FileTransition = { path: string; before: Identity; after: Identity; hunks: Hunk[] };
type Record = {
  schema: string;
  issue: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: FileTransition[];
};

/** Inverse of the #13113 authored component binding; mirrors payload_update/authored_component_infer.rs. */
const recordBytes = readFileSync(
  new URL(
    "../../../../tests/_fixtures/tooling/authored-component-infer-13113.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "0698d1a514fab517c63ec516202a2dffbe33467107b8b0062437a8f3f64a795f",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.authoredComponentInferSource.v1");
assert.equal(record.issue, 13113);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

function readSourceRecord(name: string, hash: string, schema: string, issue: number): Record {
  const bytes = readFileSync(
    new URL(`../../../../tests/_fixtures/tooling/${name}.json`, import.meta.url),
  );
  assert.equal(createHash("sha256").update(bytes).digest("hex"), hash);
  const parsed = JSON.parse(bytes.toString("utf8")) as Record;
  assert.equal(parsed.schema, schema);
  assert.equal(parsed.issue, issue);
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  return parsed;
}

const metadata = readSourceRecord(
  "component-metadata-13430",
  "24efda8a8e69229347612becd4f5503d1158d824a755f2f5b9d3b5eff5f23b95",
  "vize.componentMetadataSource.v1",
  13430,
);
const defaultExpression = readSourceRecord(
  "default-export-expression-mapping-12985",
  "c7747859a9100742ac100f965c4e981005381e65c3bf12ed1986a699e459e8d2",
  "vize.defaultExportExpressionMappingSource.v1",
  12985,
);

const scriptConsumedSlots = readSourceRecord(
  "script-consumed-slots-13578",
  "87473c12da88cdefdadb4d3fac8e2ac97c61495e45896502783c2298e7a15314",
  "vize.scriptConsumedSlotsSource.v1",
  13578,
);

const typedUseSlots = readSourceRecord(
  "typed-use-slots-13578",
  "6454233b984533bce96260ee97fd94dd989449749f4fb1b484a10582becd9978",
  "vize.typedUseSlotsSource.v1",
  13578,
);

/** Remove only the authenticated #13578 follow-up that keeps typed `useSlots()` exact. */
export function beforeTypedUseSlots(file: string, current: string): string {
  return restoreProducer(typedUseSlots, file, current);
}

/** Remove only the authenticated #13578 producer changes before historical custody. */
export function beforeScriptConsumedSlots(file: string, current: string): string {
  return restoreProducer(scriptConsumedSlots, file, current);
}

export function beforeComponentMetadata(file: string, current: string): string {
  current = beforeScriptConsumedSlots(file, beforeTypedUseSlots(file, current));
  return restoreProducer(metadata, file, restoreProducer(defaultExpression, file, current));
}

/** The pre-#13113 bytes of a recorded producer; other images pass through. */
export function beforeAuthoredComponentInfer(file: string, current: string): string {
  return restoreProducer(
    record,
    file,
    beforeWithDefaultsUndefined(file, beforeComponentMetadata(file, current)),
  );
}

function restoreProducer(record: Record, file: string, current: string): string {
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
