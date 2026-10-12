import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  emptyVueStarBodyPairs,
  emptyVueStarFiles,
  emptyVueStarManifestSHA256,
  emptyVueStarMetadataPairs,
  loadEmptyVueStarManifest,
  qualifyEmptyVueStarHost,
} from "../performance/support/warm-type-backed-empty-vue-star-host.ts";

type Side = "before" | "after";
const production = [...emptyVueStarFiles];
const allPairs = [...emptyVueStarBodyPairs, ...emptyVueStarMetadataPairs];
const pairs = new Map(allPairs.map(([file, before, after]) => [file, { before, after }]));
const digest = (side: Side, file: string) => pairs.get(file)?.[side] ?? null;
const read = (file: string) => fs.readFileSync(new URL("../../" + file, import.meta.url));
const zero = "0".repeat(64);

test("the retained42/80/152 tables and exact current44/85/152 correction carry no runtime credit", () => {
  const result = qualifyEmptyVueStarHost(production, new Set(), digest);
  assert.ok(result);
  assert.equal(result.authority, "exact-reviewed-empty-vue-star-complete-source");
  assert.equal(result.manifestSHA256, emptyVueStarManifestSHA256);
  assert.equal(result.originalSource, "a93e78147b0990c813e965a7868b32dda341eb77");
  assert.equal(result.reviewedProductSource, "fe8f4fcf9dbbe750a211cf5df2588356f785f590");
  assert.equal(result.completeBodyPairs, 44);
  assert.equal(result.supplementalBodyPairs, 85);
  assert.equal(result.preservedBodyPairs, 152);
  assert.equal(result.sourceMetadataBodyPairs, 9);
  assert.equal(result.historicalCasesPreserved, 32);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
  assert.deepEqual(result.files, production);
  const loaded = loadEmptyVueStarManifest(read);
  assert.deepEqual(
    Object.values(loaded.tables).map((rows) => rows.length),
    [42, 80, 152],
  );
  assert.equal(result.predecessorReviewedProductSource, "b148ffee7526b45789ec44c16b3e3370e08a5650");
  assert.equal(allPairs.length, 290);
  assert.equal(new Set(allPairs.map(([file]) => file)).size, 290);
});

test("every complete before/after cell refuses drift, absence, partial or foreign body digest", () => {
  let checked = 0;
  for (const [file] of allPairs)
    for (const side of ["before", "after"] as const)
      for (const changed of [zero, null, "foreign", digest(side, file)?.slice(0, 63)]) {
        if ((changed ?? null) === digest(side, file)) continue;
        const candidate = (s: Side, name: string) =>
          side === s && name === file ? (changed ?? null) : digest(s, name);
        assert.equal(
          qualifyEmptyVueStarHost(production, new Set(), candidate),
          null,
          `${side}:${file}`,
        );
        checked++;
      }
  assert.ok(checked > 1_600);
});

test("an original after body cannot supply a corrected cell or a mixed current source", () => {
  const { correction } = loadEmptyVueStarManifest(read);
  for (const [file, , previous] of correction.updates) {
    const mixed = (side: Side, name: string) =>
      side === "after" && name === file ? previous : digest(side, name);
    assert.equal(qualifyEmptyVueStarHost(production, new Set(), mixed), null, file);
  }
  for (const [file] of correction.additions) {
    const missing = (side: Side, name: string) =>
      side === "after" && name === file ? null : digest(side, name);
    assert.equal(qualifyEmptyVueStarHost(production, new Set(), missing), null, file);
  }
});

test("every required production member remains mandatory and unique even when already qualified", () => {
  for (const file of production) {
    const missing = production.filter((name) => name !== file);
    assert.equal(qualifyEmptyVueStarHost(missing, new Set(), digest), null, file);
    assert.equal(qualifyEmptyVueStarHost(missing, new Set([file]), digest), null, file);
    assert.equal(qualifyEmptyVueStarHost([...production, file], new Set(), digest), null, file);
  }
  assert.equal(qualifyEmptyVueStarHost([], new Set(), digest), null);
  assert.equal(qualifyEmptyVueStarHost(production, new Set(production), digest), null);
});

test("unknown paths refuse and preserved, supplemental and metadata files cannot be inherited as production", () => {
  assert.equal(
    qualifyEmptyVueStarHost([...production, "crates/vize/src/unreviewed.rs"], new Set(), digest),
    null,
  );
  for (const [file] of allPairs.filter(([file]) => !production.includes(file))) {
    assert.equal(qualifyEmptyVueStarHost([...production, file], new Set(), digest), null, file);
    assert.equal(
      qualifyEmptyVueStarHost([...production, file], new Set([file]), digest),
      null,
      file,
    );
  }
  const inherited = new Set(["crates/vize/src/previously-qualified.rs"]);
  assert.deepEqual(
    qualifyEmptyVueStarHost([...production, ...inherited], inherited, digest)?.files,
    production,
  );
  assert.equal(qualifyEmptyVueStarHost([...inherited], inherited, digest), null);
  assert.equal(
    qualifyEmptyVueStarHost([...production, ...inherited, ...inherited], inherited, digest),
    null,
  );
});

test("all full table and manifest byte mutations fail before their rows can be trusted", () => {
  assert.equal(loadEmptyVueStarManifest(read).tables.preservedBodies.length, 152);
  for (const [file] of emptyVueStarMetadataPairs) {
    const original = read(file);
    for (const altered of [
      Buffer.concat([original, Buffer.from("\n")]),
      original.subarray(0, original.length - 1),
      Buffer.from("[]\n"),
    ])
      assert.throws(
        () => loadEmptyVueStarManifest((name) => (name === file ? altered : read(name))),
        undefined,
        file,
      );
    assert.throws(
      () =>
        loadEmptyVueStarManifest((name) => {
          if (name === file) throw new Error("missing complete metadata file");
          return read(name);
        }),
      /missing complete metadata file/u,
    );
  }
});

test("partial, duplicate, reordered or foreign table declarations refuse whole manifest custody", () => {
  const [file] = emptyVueStarMetadataPairs[0];
  const original = JSON.parse(read(file).toString("utf8"));
  const candidates = [
    { ...original, parts: original.parts.slice(1) },
    { ...original, parts: [...original.parts, original.parts[0]] },
    { ...original, parts: original.parts.toReversed() },
    { ...original, originalSource: "1".repeat(40) },
    { ...original, counts: { ...original.counts, files: 40 } },
  ];
  for (const candidate of candidates)
    assert.throws(() =>
      loadEmptyVueStarManifest((name) =>
        name === file ? Buffer.from(JSON.stringify(candidate, null, 2) + "\n") : read(name),
      ),
    );
});
