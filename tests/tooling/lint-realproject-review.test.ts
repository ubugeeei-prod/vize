import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import {
  loadLintSnapshotAlignments,
  verifyLintSnapshotAlignment,
} from "../_helpers/lint-snapshot-alignment.ts";
import { reviewLocalizedRequireScopedStyle } from "./support/lint-output-policy-review.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const review = path.join(root, "tests/_fixtures/_projects/lint-review/2026-10-11");
const alignments = loadLintSnapshotAlignments(root);
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

test("successor reviews retain historical hashes and all 395 sources at unchanged fixture pins", () => {
  let changedPackets = 0;
  let successors = 0;
  for (const [snapshot, alignment] of alignments) {
    if (!alignment.successor) continue;
    const { successor } = alignment;
    assert.equal(sha256(successor.previousRaw), alignment.currentSha256);
    assert.equal(successor.review.previousSha256, alignment.currentSha256);
    assert.equal(
      sha256(fs.readFileSync(path.join(root, snapshot))),
      successor.review.currentSha256,
    );
    const project = path.basename(snapshot, "-lint.snap");
    const gitlink = execFileSync("git", ["ls-tree", "HEAD", `tests/_fixtures/_git/${project}`], {
      cwd: root,
      encoding: "utf8",
    });
    assert.equal(gitlink.trim().split(/\s+/)[2], successor.fixturePin);
    changedPackets += successor.review.changedPackets;
    successors++;
  }
  assert.equal(successors, 6);
  assert.equal(changedPackets, 395);
  assert.equal(
    alignments.get("tests/snapshots/lint/__snapshots__/misskey-lint.snap")?.currentSha256,
    "f3b1001b1198613da2088667619b13f5f32d1f7c858046ad702a1f6c3eaf88a1",
    "the successor must not rewrite the original current hash",
  );
});

test("successor admission rejects unreviewed changes on either side of the historical chain", () => {
  const alignment = alignments.get("tests/snapshots/lint/__snapshots__/misskey-lint.snap")!;
  assert.ok(alignment.successor);
  const current = fs.readFileSync(path.join(root, alignment.path), "utf8");
  const corrected = JSON.parse(
    gunzipSync(
      fs.readFileSync(
        path.join(
          root,
          "tests/_fixtures/_projects/lint-review/2026-10-11-direct-client-correction/original-provider-packets.json.gz",
        ),
      ),
    ).toString(),
  ).find((entry: { path: string }) => entry.path === alignment.path);
  const historical = reviewLocalizedRequireScopedStyle(corrected.raw).raw;
  verifyLintSnapshotAlignment(historical, current, alignment);
  const badPredecessor = structuredClone(alignment);
  badPredecessor.successor!.previousRaw += "\n";
  assert.throws(() => verifyLintSnapshotAlignment(historical, current, badPredecessor));
  assert.throws(() => verifyLintSnapshotAlignment(`${historical}\n`, current, alignment));
  const { successor, ...original } = alignment;
  // Verify the successor boundary independently of the older provider packet.
  assert.throws(() =>
    verifyLintSnapshotAlignment(successor.previousRaw, `${current}\n`, successor.review),
  );
  assert.throws(() =>
    verifyLintSnapshotAlignment(`${successor.previousRaw}\n`, current, successor.review),
  );
  assert.equal(original.currentSha256, successor.review.previousSha256);
});

test("reviewed snapshots retain every whole provider packet and all 1977 ordered source identities", () => {
  const original = compressed("original-provider-packets.json.gz");
  const provenance = compressed("source-provenance.json.gz");
  let files = 0,
    changed = 0,
    reviewedMessages = 0;
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
    const reviewed = reviewLocalizedRequireScopedStyle(admitted.raw);
    reviewedMessages += reviewed.reviewedMessages;
    const bytes = fs.readFileSync(path.join(root, capture.path));
    const alignment = alignments.get(capture.path);
    assert(alignment);
    verifyLintSnapshotAlignment(reviewed.raw, bytes.toString(), alignment);
    assert.equal(
      sha256(admitted.raw),
      admitted.sha256,
      "admitted producer evidence remains intact",
    );
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
  assert.equal(reviewedMessages, 37);
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
