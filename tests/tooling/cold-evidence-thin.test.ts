import assert from "node:assert/strict";
import { Readable } from "node:stream";
import { gzipSync } from "node:zlib";
import { test } from "node:test";
import { writeTar, writeTarGz } from "../../tools/support/archive/tar.ts";
import { evidenceTarStream } from "../../tools/benchmarks/scripts/cold-evidence-tar.ts";
import {
  coldEvidenceCollector,
  selectedEvidence,
} from "../../tools/benchmarks/scripts/cold-evidence-thin.ts";
import {
  parseEvidenceJson,
  recordEvidence,
  stringifyEvidenceJson,
} from "../../tools/benchmarks/scripts/cold-evidence-json.ts";

void test("metadata JSON preserves u64/i64 lexemes without converting strings", () => {
  const source =
    '{"max":18446744073709551615,"min":-9223372036854775808,"string":"18446744073709551615","small":4323}';
  const value = recordEvidence(parseEvidenceJson(source));
  assert.equal(value.max, 18446744073709551615n);
  assert.equal(value.min, -9223372036854775808n);
  assert.equal(value.string, "18446744073709551615");
  assert.equal(stringifyEvidenceJson(value), source);
});
void test("streamed metadata preserves exact bytes and rejects links/duplicates/missing evidence", async () => {
  const body = Buffer.from('{"unchanged":true}\n'),
    written = new Map<string, Buffer>();
  const collector = coldEvidenceCollector((p, b) => written.set(p, b), ["base.sha"]);
  const archive = writeTarGz([
    { name: "cold-scale/base-1/measurement.json", data: body },
    { name: "cold-scale/binaries/normal-head/vize", data: Buffer.alloc(1024 * 1024, 7) },
    { name: "cold-scale/base-1/measurement.json", data: Buffer.from("changed") },
    { name: "cold-scale/builds/link.json", typeflag: "2", linkname: "/private" },
  ]);
  for await (const member of evidenceTarStream(Readable.from([archive]), collector.retain))
    collector.consume(member);
  const result = collector.result();
  assert.deepEqual([...written.keys()], ["base-1/measurement.json"]);
  assert.deepEqual(written.get("base-1/measurement.json"), body);
  assert.equal(result.errors.length, 2);
  assert.deepEqual(result.requiredMissing, ["base.sha"]);
  assert.equal(result.totalBytes, body.length);
  for (const path of [
    "../cold-scale/base.sha",
    "/cold-scale/base.sha",
    "cold-scale/record-head/perf.data",
    "cold-scale/record-head/project/source.json",
    "cold-scale/x/../base.sha",
    "cold-scale/record-head/tmp/a.txt",
  ])
    assert.equal(selectedEvidence(path), null);
});
void test("GNU long-name metadata and strict checksum/truncation are retained", async () => {
  const name = "cold-scale/builds/" + "x".repeat(120) + ".json";
  const archive = writeTarGz([
    { name: "././@LongLink", typeflag: "L", data: Buffer.from(name + "\0") },
    { name: "short", data: Buffer.from("{}") },
  ]);
  const rows = [];
  for await (const row of evidenceTarStream(Readable.from([archive]), () => true)) rows.push(row);
  assert.equal(rows.length, 1);
  assert.equal(rows[0]?.name, name);
  assert.equal(rows[0]?.data?.toString(), "{}");
  const raw = writeTar([{ name: "cold-scale/base.sha", data: Buffer.from("abc") }]);
  const bad = Buffer.from(raw);
  bad[0] = (bad[0] ?? 0) ^ 1;
  const consume = async (body: Buffer) => {
    for await (const row of evidenceTarStream(Readable.from([gzipSync(body)]), () => true))
      void row;
  };
  await assert.rejects(consume(bad), /checksum/u);
  await assert.rejects(consume(raw.subarray(0, 514)), /truncated/u);
});
