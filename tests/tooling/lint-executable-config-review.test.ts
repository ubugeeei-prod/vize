import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

const root = fileURLToPath(new URL("../../", import.meta.url));
const review = path.join(
  root,
  "tests/_fixtures/_projects/lint-review/2026-10-11-executable-configs",
);
const compressed = (name: string) =>
  JSON.parse(gunzipSync(fs.readFileSync(path.join(review, name))).toString());
const compare = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.entries(value)
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
      .map(([key, entry]) => `${JSON.stringify(key)}:${canonical(entry)}`)
      .join(",")}}`;
  }
  return JSON.stringify(value).replace(
    /[\u0080-\uffff]/g,
    (character) => `\\u${character.charCodeAt(0).toString(16).padStart(4, "0")}`,
  );
}

test("the genuinely red provider run retains its complete ZIP, runtime closures and all 803 source packets", () => {
  const inventory = JSON.parse(fs.readFileSync(path.join(review, "inventory.json"), "utf8"));
  for (const entry of inventory) {
    const bytes = fs.readFileSync(path.join(review, entry.path));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(hash(bytes), entry.sha256, entry.path);
  }
  const binding = JSON.parse(fs.readFileSync(path.join(review, "binding.json"), "utf8"));
  assert.equal(binding.runConclusion, "failure");
  assert.equal(binding.lintCasesPassed, 13);
  assert.equal(binding.lintCasesRequired, 15);
  assert.equal(
    hash(fs.readFileSync(path.join(review, "original-provider-artifact.zip"))),
    binding.artifactSha256,
  );
  assert.equal(
    hash(
      gunzipSync(fs.readFileSync(path.join(review, "8589-corrected-lint-all-114400465871.log.gz"))),
    ),
    binding.jobRawLogSha256,
  );
  const runtime = compressed("runtime-fixture-receipts.json.gz");
  assert.deepEqual(Object.keys(runtime).sort(compare), [
    "misskey",
    "naive-ui",
    "npmx.dev",
    "voicevox",
  ]);
  assert.equal(runtime["npmx.dev"].originalFrozenClosure, false);
  assert.equal(
    Object.values(runtime).reduce((total: number, value: any) => total + value.originalFiles, 0),
    10341,
  );
  const provenance = [
    compressed("misskey-all-583-file-packet-source-provenance.json.gz"),
    compressed("npmx-all-220-file-packet-source-provenance.json.gz"),
  ];
  const captures = compressed("original-provider-packets.json.gz");
  let total = 0,
    changed = 0;
  for (const [index, capture] of captures.entries()) {
    assert.equal(hash(capture.raw), capture.sha256);
    const packets = JSON.parse(capture.raw);
    const identities = provenance[index].files;
    assert.equal(packets.length, identities.length);
    for (const [position, packet] of packets.entries()) {
      const identity = identities[position];
      assert.equal(packet.file, identity.file);
      assert.equal(hash(canonical(packet)), identity.actualWholePacketSha256);
      assert.match(identity.gitBlob, /^[a-f0-9]{40}$/);
      changed += Number(identity.expectedWholePacketSha256 !== identity.actualWholePacketSha256);
      total++;
    }
  }
  assert.equal(total, 803);
  assert.equal(changed, 216);
});

test("all 435 whole message deltas are reviewed and direct client-guard defects remain explicitly red", () => {
  const delta = compressed("packet-deltas.json.gz");
  const misskey = compressed("misskey-new-whole-policy-review.json.gz");
  const npmx = compressed("npmx-new-whole-policy-review.json.gz");
  const observed: string[] = [];
  for (const project of delta) {
    for (const kind of ["removed", "added"]) {
      for (const whole of project[kind])
        observed.push(canonical({ project: project.project, kind, whole }));
    }
  }
  const reviewed = [
    ...misskey.entries.map((entry: any) =>
      canonical({ project: "misskey", kind: entry.kind, whole: entry.whole_packet }),
    ),
    ...npmx.map((entry: any) =>
      canonical({ project: "npmx.dev", kind: entry.kind, whole: entry.whole_packet }),
    ),
  ];
  assert.equal(observed.length, 435);
  assert.deepEqual(
    observed.sort(compare),
    reviewed.sort(compare),
    "no message field or rule may be stripped",
  );
  assert.deepEqual(misskey.unjustifiedChangedMessagesFound, []);
  const defects = npmx.filter((entry: any) => entry.status === "requires_source_correction");
  assert.deepEqual(
    defects.map((entry: any) => entry.whole_packet.file),
    ["app/app.vue", "app/components/Diff/MobileSidebarDrawer.vue"],
  );
  assert.equal(
    npmx.filter((entry: any) => entry.status === "documented_unproven_indirect_boundary").length,
    1,
  );
});

test("the original 51-rule projection and explicit typed-default comparison retain complete official packets", () => {
  const misskey = compressed("misskey-official-all51-capture.json.gz");
  const npmx = compressed("official-51-npmx-new-capture.json.gz");
  const typed = compressed("misskey-official-52-capture.json.gz");
  assert.equal(misskey.projectionSha256, npmx.projectionSha256);
  assert.equal(typed.originalProjectionSha256, misskey.projectionSha256);
  assert.equal(Object.keys(misskey.configurations[0].rules).length, 51);
  assert.deepEqual(misskey.configurations[0].rules, npmx.configurations[0].rules);
  for (const [identity, options] of Object.entries(misskey.configurations[0].rules)) {
    assert.deepEqual(typed.rules[identity], options);
  }
  assert.equal(Object.keys(typed.rules).length, 52);
  assert.equal(misskey.captures.length, 169);
  assert.equal(npmx.captures.length, 47);
  assert.equal(typed.captures.length, 169);
  for (const capture of [...misskey.captures, ...npmx.captures, ...typed.captures]) {
    assert.equal(capture.error, undefined);
    assert.match(capture.sourceSha256, /^[a-f0-9]{64}$/);
    for (const packet of capture.packets) assert.equal(packet.fatalErrorCount, 0);
  }
  const findings = typed.captures
    .flatMap((capture: any) => capture.packets.flatMap((packet: any) => packet.messages))
    .filter((message: any) => message.ruleId === "vue/no-required-prop-with-default");
  assert.equal(findings.length, 5);
  assert.ok(
    findings.every((message: any) => message.severity === 2 && message.suggestions.length > 0),
  );
});

test("the Oct 11 corrected captures retract exactly three positive client-guard warnings and preserve every other field", () => {
  const directory = path.join(
    root,
    "tests/_fixtures/_projects/lint-review/2026-10-11-direct-client-correction",
  );
  const read = (name: string) =>
    JSON.parse(gunzipSync(fs.readFileSync(path.join(directory, name))).toString());
  const inventory = JSON.parse(fs.readFileSync(path.join(directory, "inventory.json"), "utf8"));
  for (const entry of inventory) {
    const bytes = fs.readFileSync(path.join(directory, entry.path));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(hash(bytes), entry.sha256);
  }
  const binding = JSON.parse(fs.readFileSync(path.join(directory, "binding.json"), "utf8"));
  assert.equal(binding.runConclusion, "failure");
  assert.equal(binding.lintCasesPassed, 12);
  assert.equal(
    hash(fs.readFileSync(path.join(directory, "original-provider-artifact.zip"))),
    binding.artifactSha256,
  );
  assert.equal(
    hash(gunzipSync(fs.readFileSync(path.join(directory, "original-job.log.gz")))),
    binding.jobRawLogSha256,
  );
  const captures = read("original-provider-packets.json.gz");
  const reviews = read("whole-correction-review.json.gz");
  const witnesses = read("original-source-witnesses.json.gz");
  const observed: unknown[] = [];
  let files = 0;
  for (const capture of captures) {
    // These immutable captures retain the Oct 11 correction. Current App lint
    // goldens evolve with separately reviewed rule and diagnostic changes.
    assert.equal(hash(capture.raw), capture.sha256);
    assert.equal(hash(capture.baselineRaw), capture.baselineSha256);
    const actual = JSON.parse(capture.raw),
      previous = JSON.parse(capture.baselineRaw);
    assert.deepEqual(
      actual.map((packet: any) => packet.file),
      previous.map((packet: any) => packet.file),
    );
    files += actual.length;
    for (const [index, packet] of actual.entries()) {
      const before = previous[index];
      if (canonical(before) === canonical(packet)) continue;
      const review = reviews.find(
        (entry: any) => entry.project === capture.project && entry.file === packet.file,
      );
      assert.deepEqual(before, review.expected);
      assert.deepEqual(packet, review.actual);
      assert.deepEqual(
        before.messages.filter(
          (message: any) => canonical(message) === canonical(review.removedWholeMessage),
        ),
        [review.removedWholeMessage],
      );
      assert.equal(review.removedWholeMessage.ruleId, "ssr/no-browser-globals-in-ssr");
      const remaining = before.messages.filter(
        (message: any) => canonical(message) !== canonical(review.removedWholeMessage),
      );
      assert.deepEqual(packet, {
        ...before,
        messages: remaining,
        warningCount: before.warningCount - 1,
      });
      const source = witnesses.find(
        (entry: any) => entry.project === capture.project && entry.file === packet.file,
      );
      const sourceBytes = Buffer.from(source.source);
      assert.equal(hash(sourceBytes), source.sourceSha256);
      assert.equal(
        createHash("sha1").update(`blob ${sourceBytes.length}\0`).update(sourceBytes).digest("hex"),
        source.gitBlob,
      );
      assert.equal(source.clientGuard, "positive direct import.meta.client");
      observed.push([capture.project, packet.file]);
    }
  }
  assert.equal(files, 874);
  assert.equal(reviews.length, 3);
  assert.deepEqual(observed, [
    ["npmx.dev", "app/app.vue"],
    ["npmx.dev", "app/components/Diff/MobileSidebarDrawer.vue"],
    ["vuefes-2025", "app/pages/timetable/index.vue"],
  ]);
  const npmx = JSON.parse(captures.find((entry: any) => entry.project === "npmx.dev").raw);
  const indirect = npmx.find(
    (packet: any) => packet.file === "app/pages/package/[[org]]/[name].vue",
  );
  assert.ok(
    indirect.messages.some(
      (message: any) => message.ruleId === "ssr/no-browser-globals-in-ssr" && message.line === 258,
    ),
  );
});
