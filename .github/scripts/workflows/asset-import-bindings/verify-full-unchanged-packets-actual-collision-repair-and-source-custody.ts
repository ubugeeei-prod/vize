import assert from "node:assert/strict";
import fs from "node:fs";
const root = `${process.env.RUNNER_TEMP}/asset-import-bindings`;
interface NativeReport {
  sourceSha: string;
  native: { buildToken: string; sha256: string; image: string };
  packets: {
    source: string;
    options: unknown;
    result: unknown;
    assets: { url: string; varName: string }[];
    check: { status: number };
    rewritten: string;
    assembled: string;
    observedFreeValue: string;
  }[];
}
interface Profile {
  cases: {
    case: string;
    attribution_calls: number;
    timing_samples_ns: number[];
    timing_profiler_enabled: boolean;
    attribution_allocation_calls: number;
    attribution_alloc_bytes: number;
    counters: { name: string }[];
  }[];
}
const read = <T>(name: string): T => JSON.parse(fs.readFileSync(`${root}/${name}.json`, "utf8"));
const base = read<NativeReport>("base-native"),
  head = read<NativeReport>("head-native");
assert.equal(base.packets.length, 64);
assert.equal(head.packets.length, base.packets.length);
assert.equal(base.sourceSha, process.env.BASE_SHA);
assert.equal(head.sourceSha, process.env.HEAD_SHA);
assert.notEqual(base.native.buildToken, head.native.buildToken);
assert.notEqual(base.native.sha256, head.native.sha256);
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
] as const) {
  const selected = targets.find((row) => row[0] === report.sourceSha);
  assert.ok(selected);
  const [sha, target, token] = selected;
  assert.equal(sha, report.sourceSha);
  assert.equal(report.native.buildToken, token, "actual loaded image immutable identity");
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
  assert.equal(
    emissions[0].fresh,
    false,
    "actual source producer must compile in its isolated target",
  );
  const sourceRoot =
    epoch === "head"
      ? process.env.GITHUB_WORKSPACE
      : `${process.env.RUNNER_TEMP}/asset-import-bindings-base`;
  assert.equal(emissions[0].target.src_path, `${sourceRoot}/crates/vize_vitrine/src/lib.rs`);
  assert.ok(emissions[0].filenames.includes(`${target}/ci/libvize_vitrine.so`));
  assert.deepEqual(
    fs.readFileSync(`${target}/ci/libvize_vitrine.so`),
    fs.readFileSync(report.native.image),
  );
}
let repaired = 0,
  unchanged = 0;
for (let index = 0; index < base.packets.length; index++) {
  const before = base.packets[index],
    after = head.packets[index];
  assert.equal(before.source, after.source);
  assert.deepEqual(before.options, after.options);
  assert.deepEqual(before.result, after.result, "full compiler/map/resource packet");
  assert.deepEqual(
    before.assets.map((row) => row.url),
    after.assets.map((row) => row.url),
  );
  assert.equal(after.check.status, 0);
  if (before.assets.every((row, i) => row.varName === after.assets[i].varName)) {
    assert.deepEqual(before.assets, after.assets);
    assert.equal(before.rewritten, after.rewritten);
    assert.equal(before.assembled, after.assembled);
    unchanged++;
  } else repaired++;
}
assert.ok(
  base.packets.some((row) => row.check.status !== 0),
  "before must witness real syntax failures",
);
assert.ok(
  base.packets.some(
    (row: { observedFreeValue: string }) => row.observedFreeValue === "asset:./a.svg",
  ),
  "before must witness captured global",
);
assert.ok(repaired > 0 && unchanged > 0);
const profiles = [read<Profile>("base-profile"), read<Profile>("head-profile")];
for (const report of profiles) {
  assert.equal(report.cases.length, 6);
  for (const row of report.cases) {
    assert.equal(row.attribution_calls, 1);
    assert.equal(row.timing_samples_ns.length, 9);
    assert.equal(row.timing_profiler_enabled, false);
    assert.ok(row.attribution_allocation_calls > 0);
  }
}
for (const name of ["ordinary", "no-assets"]) {
  const before = profiles[0].cases.find((row: { case: string }) => row.case === name),
    after = profiles[1].cases.find((row: { case: string }) => row.case === name);
  assert.ok(before && after, name);
  assert.equal(after.attribution_allocation_calls, before.attribution_allocation_calls);
  assert.equal(after.attribution_alloc_bytes, before.attribution_alloc_bytes);
  assert.equal(
    after.counters.filter((row: { name: string }) => row.name.startsWith("sfc.asset_names."))
      .length,
    0,
  );
}
fs.writeFileSync(
  `${root}/acceptance.json`,
  JSON.stringify(
    {
      sourceBefore: base.sourceSha,
      sourceHead: head.sourceSha,
      cases: head.packets.length,
      repaired,
      unchanged,
      nativeBefore: base.native,
      nativeHead: head.native,
      scope: "correctness and isolated collector observations; no whole-tool speed ratio",
    },
    null,
    2,
  ) + "\n",
);
