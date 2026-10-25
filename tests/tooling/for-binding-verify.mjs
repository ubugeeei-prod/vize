import assert from "node:assert/strict";
import fs from "node:fs";
const [root] = process.argv.slice(2);
const read = (name) => JSON.parse(fs.readFileSync(`${root}/${name}.json`));
const base = read("base-native"),
  head = read("head-native");
assert.equal(base.sourceSha, process.env.BASE_SHA);
assert.equal(head.sourceSha, process.env.HEAD_SHA);
assert.equal(base.records.length, 78);
assert.equal(head.records.length, 78);
assert.notEqual(base.native.sha256, head.native.sha256);
assert.notEqual(base.native.token, head.native.token);
const targets = fs
  .readFileSync(`${root}/source-target-token.txt`, "utf8")
  .trim()
  .split("\n")
  .map((line) => line.split(" "));
assert.equal(targets.length, 2);
assert.notEqual(targets[0][1], targets[1][1]);
for (const [epoch, report] of [
  ["base", base],
  ["head", head],
]) {
  const [sha, target, token] = targets.find((row) => row[0] === report.sourceSha);
  assert.equal(sha, report.sourceSha);
  assert.equal(report.native.token, token);
  const messages = fs
    .readFileSync(`${root}/${epoch}-native-build.jsonl`, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const emissions = messages.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === "vize_vitrine" &&
      row.target.kind.includes("cdylib"),
  );
  assert.equal(emissions.length, 1);
  assert.equal(emissions[0].fresh, false);
  const source =
    epoch === "head"
      ? process.env.GITHUB_WORKSPACE
      : `${process.env.RUNNER_TEMP}/for-bindings-base`;
  assert.equal(emissions[0].target.src_path, `${source}/crates/vize_vitrine/src/lib.rs`);
  assert.ok(emissions[0].filenames.includes(`${target}/ci/libvize_vitrine.so`));
  assert.deepEqual(
    fs.readFileSync(`${target}/ci/libvize_vitrine.so`),
    fs.readFileSync(report.native.image),
  );
}
let unchanged = 0,
  changed = 0;
for (let index = 0; index < head.records.length; index++) {
  const before = base.records[index],
    after = head.records[index];
  assert.equal(before.source, after.source);
  assert.equal(before.runtimeInputText, after.runtimeInputText);
  assert.deepEqual(before.options, after.options);
  assert.equal(before.id, after.id);
  assert.equal(before.backend, after.backend);
  assert.equal(before.newline, after.newline);
  assert.deepEqual(after.result.errors, []);
  assert.ok(after.result.map && after.mapSegments.length > 0);
  assert.equal(after.observed.length, 2);
  if (after.backend !== "ssr") assert.equal(after.updated.length, 3);
  if (after.unchanged) {
    assert.deepEqual(after.result, before.result, "entire ordinary compiler/maps/resources packet");
    assert.deepEqual(after.mapSegments, before.mapSegments);
    for (const key of ["serverHtml", "initialHtml", "updatedHtml", "observed", "updated"])
      assert.deepEqual(after[key], before[key]);
    unchanged++;
  } else if (after.result.code !== before.result.code) changed++;
}
assert.equal(unchanged, 12);
assert.ok(changed > 0);
assert.ok(
  base.records.some(
    (row) =>
      JSON.stringify(row.observed) !==
      JSON.stringify(
        head.records.find(
          (after) =>
            after.id === row.id && after.newline === row.newline && after.backend === row.backend,
        ).observed,
      ),
  ),
  "actual before runtime must witness the old scope defect",
);
const beforeProfile = read("base-profile"),
  afterProfile = read("head-profile");
assert.equal(beforeProfile.rows.length, 27);
assert.equal(afterProfile.rows.length, 27);
for (const row of afterProfile.rows) {
  const before = beforeProfile.rows.find(
    (candidate) => candidate.case === row.case && candidate.backend === row.backend,
  );
  assert.equal(row.source, before.source);
  if (row.case === "ordinary-key-literal") {
    const parserCounters = (record) =>
      new Map(
        record.counters
          .filter((counter) => counter.name.endsWith(".parses"))
          .map((counter) => [counter.name, [counter.samples, counter.total]]),
      );
    assert.deepEqual(
      parserCounters(row),
      parserCounters(before),
      "the actual ordinary callback path must add no parser calls",
    );
  }
  assert.equal(row.calls, 1);
  assert.equal(row.success, true);
  if (row.case.startsWith("ordinary"))
    assert.equal(
      row.counters.filter(
        (counter) =>
          counter.name.startsWith("atelier.for_binding.") ||
          counter.name === "davinci.for_binding.parses",
      ).length,
      0,
    );
  else
    assert.ok(
      row.counters.some(
        (counter) =>
          (counter.name === "atelier.for_binding.parses" ||
            counter.name === "davinci.for_binding.parses") &&
          counter.total > 0,
      ),
    );
}
fs.writeFileSync(
  `${root}/acceptance.json`,
  JSON.stringify(
    {
      sourceBefore: base.sourceSha,
      sourceHead: head.sourceSha,
      cases: 78,
      unchanged,
      changed,
      nativeBefore: base.native,
      nativeHead: head.native,
      runtimes: head.runtimes,
      scope:
        "DOM and Vapor actual mount/update in pinned real Vue runtime under Happy DOM; genuine installed Vue SSR; full source/maps/raw HTML packets retained; no browser-pixel or throughput claim",
    },
    null,
    2,
  ) + "\n",
);
