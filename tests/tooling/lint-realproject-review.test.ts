import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

const root = fileURLToPath(new URL("../../", import.meta.url));
const review = path.join(root, "tests/_fixtures/_projects/lint-review/2026-10-11");
const compressed = (name: string) =>
  JSON.parse(gunzipSync(fs.readFileSync(path.join(review, name))).toString());
const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.entries(value)
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
      .map(([key, entry]) => `${JSON.stringify(key)}:${canonical(entry)}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

test("the Oct 11 review retains every whole provider packet and all 1977 ordered source identities", () => {
  const original = compressed("original-provider-packets.json.gz");
  const provenance = compressed("source-provenance.json.gz");
  let files = 0,
    changed = 0;
  for (const capture of original) {
    const corrected = JSON.parse(
      gunzipSync(
        fs.readFileSync(
          path.join(
            root,
            "tests/_fixtures/_projects/lint-review/2026-10-11-direct-client-correction/original-provider-packets.json.gz",
          ),
        ),
      ).toString(),
    ).find((entry: { path: string }) => entry.path === capture.path);
    const admitted = corrected ?? capture;
    // Preserve the historical admission and correction, independently of
    // current App lint goldens that incorporate later reviewed rule changes.
    assert.equal(sha256(admitted.raw), admitted.sha256, "reviewed whole packet bytes stay intact");
    assert.equal(sha256(capture.raw), capture.sha256, "original producer evidence remains intact");
    const packets = JSON.parse(capture.raw);
    const source = provenance.find(
      (entry: { snapshot: string }) => entry.snapshot === capture.path,
    );
    assert.equal(source.original_fixture_pin, source.actual_fixture_pin);
    assert.equal(packets.length, source.files.length);
    for (const [index, packet] of packets.entries()) {
      const identity = source.files[index];
      assert.equal(packet.file, identity.file, "no fixture may be dropped or reordered");
      // The original receipt uses Python's sorted-key, ASCII JSON encoding.
      const asciiPacket = canonical(packet).replace(
        /[\u0080-\uffff]/g,
        (character) => `\\u${character.charCodeAt(0).toString(16).padStart(4, "0")}`,
      );
      assert.equal(sha256(asciiPacket), identity.actual_packet_sha256);
      assert.match(identity.source_blob_oid, /^[a-f0-9]{40}$/);
      changed += Number(!identity.unchanged_diagnostic_packet);
      files++;
    }
  }
  assert.equal(files, 1977);
  assert.equal(changed, 316);
});

test("every added and removed whole diagnostic has a reviewed source policy", () => {
  const deltas = compressed("packet-deltas.json.gz");
  const policies = compressed("policy-review.json.gz");
  const actual: string[] = [];
  for (const project of deltas) {
    for (const kind of ["removed", "added"]) {
      for (const whole of project[kind]) {
        actual.push(canonical({ snapshot: project.snapshot, kind, whole_packet: whole }));
      }
    }
  }
  assert.equal(actual.length, 395);
  const expected = policies.map(
    (policy: { snapshot: string; kind: string; whole_packet: unknown; review: string }) => {
      assert.ok(policy.review.length > 0);
      return canonical({
        snapshot: policy.snapshot,
        kind: policy.kind,
        whole_packet: policy.whole_packet,
      });
    },
  );
  assert.deepEqual(
    actual.sort(),
    expected.sort(),
    "no message field or policy witness may be omitted",
  );
});

test("the official comparison retains all 51 explicit rules and complete 316 provider captures", () => {
  const oracle = compressed("official-oracle.json.gz");
  assert.equal(oracle.loadError, undefined);
  assert.equal(Object.keys(oracle.configurations[0].rules).length, 51);
  assert.equal(oracle.captures.length, 316);
  for (const capture of oracle.captures) {
    assert.equal(capture.error, undefined);
    assert.equal(capture.packets.length, 1);
    assert.match(capture.sourceSha256, /^[a-f0-9]{64}$/);
  }
  const raw = gunzipSync(fs.readFileSync(path.join(review, "official-oracle.ndjson.gz")));
  const summary = JSON.parse(
    fs.readFileSync(path.join(review, "official-oracle-summary.json"), "utf8"),
  );
  assert.equal(sha256(raw), summary.rawSha256);
});
