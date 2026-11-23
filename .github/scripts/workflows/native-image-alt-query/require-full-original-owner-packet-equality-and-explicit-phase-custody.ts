import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const directory = `${process.env.RUNNER_TEMP}/native-image-alt`;
const read = (name: string) => JSON.parse(readFileSync(`${directory}/${name}.json`, "utf8"));
const base = read("base"),
  head = read("head"),
  canonical = read("canonical");
assert.equal(base.source, head.source);
assert.equal(base.representation, head.representation);
assert.equal(base.phase, "patina.native.original.img_alt.observer");
assert.equal(head.phase, base.phase);
assert.deepEqual(head.packets, base.packets, "complete original-header diagnostic packets");
assert.equal(head.packets.length, 16);
assert.equal(canonical.length, 3);
assert.deepEqual(
  canonical.map((row: { representation: string }) => row.representation),
  [
    "Vue NativeSfc canonical File",
    "JSX JsxMarkupFile same-Program canonical File",
    "TSX JsxMarkupFile same-Program canonical File",
  ],
);
for (const row of [base, head, ...canonical]) {
  assert.equal(row.packets.length, 16);
  assert.equal(row.samples.length, 9);
  for (const sample of row.samples) {
    assert.equal(sample.calls, 500);
    assert.ok(sample.average_ns > 0);
  }
}
for (const row of canonical) assert.equal(row.phase, "patina.native.canonical.img_alt");
const hash = (name: string) =>
  createHash("sha256")
    .update(readFileSync(`${directory}/${name}`))
    .digest("hex");
const targets = readFileSync(`${directory}/isolated-targets.txt`, "utf8").trim().split("\n");
assert.equal(targets.length, 2);
assert.notEqual(targets[0], targets[1]);
const compiled = ["base", "head"].map((epoch, index) => {
  const messages = readFileSync(`${directory}/${epoch}-build.jsonl`, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const library = messages.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === "vize_patina" &&
      row.target.kind.includes("lib"),
  );
  assert.equal(library.length, 1);
  assert.equal(library[0].fresh, false, `${epoch} Patina must compile from its own source`);
  assert.ok(library[0].filenames.every((file: string) => file.startsWith(`${targets[index]}/`)));
  const root =
    epoch === "base"
      ? `${process.env.RUNNER_TEMP}/native-image-alt-base`
      : process.env.GITHUB_WORKSPACE;
  assert.equal(library[0].target.src_path, `${root}/crates/vize_patina/src/lib.rs`);
  assert.ok(library[0].package_id.startsWith(`path+file://${root}/crates/vize_patina#`));
  return {
    epoch,
    packageId: library[0].package_id,
    source: library[0].target.src_path,
    filenames: library[0].filenames,
    fresh: library[0].fresh,
  };
});
const receipt = {
  base: process.env.BASE_SHA,
  head: process.env.HEAD_SHA,
  originalPacketEquality: true,
  baseCaptureSha256: hash("base.json"),
  baseObserverOrigin: readFileSync(`${directory}/base-observer-origin.txt`, "utf8").trim(),
  headCaptureSha256: hash("head.json"),
  canonicalCaptureSha256: hash("canonical.json"),
  originalOwner: head.representation,
  canonicalOwners: canonical.map((row: { representation: string }) => row.representation),
  isolatedCompiledTargets: targets,
  compiledLibraries: compiled,
  baseProductionSha256: hash("base-production.sha256"),
  headProductionSha256: hash("head-production.sha256"),
  profile: "ci-opt: release, thin LTO, 16 codegen units",
  crossRepresentationRatioAsserted: false,
  defaultDriverMigrationAsserted: false,
  wholeProductPerformanceAsserted: false,
};
writeFileSync(`${directory}/receipt.json`, JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify(receipt));
