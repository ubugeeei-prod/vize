import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type BodyDigest = (side: "before" | "after", file: string) => string | null;
type Pair = readonly [string, string | null, string];
type Section = "files" | "supplementalBodies" | "preservedBodies";
type Part = {
  file: string;
  section: Section;
  start: number;
  entries: number;
  mode: "100644";
  bytes: number;
  sha256: string;
  blob: string;
};
type Stored = {
  version: number;
  originalSource: string;
  reviewedProductSource: string;
  originalReviewedProductSource: string;
  correctiveSourceIndexSHA256: string;
  reviewedSourceIndexSHA256: string;
  integrationSourceProofSHA256: string;
  qualification: string;
  counts: Record<Section, number>;
  historicalCases: number;
  historicalPackBodies: number;
  historicalDistinctBodies: number;
  parts: Part[];
  correction: Omit<Part, "section" | "start" | "entries">;
};
const prefix = "tests/performance/support/warm-type-backed-empty-vue-star-";
const manifestFile = prefix + "manifest.json";
export const emptyVueStarManifestSHA256 =
  "bac92f00182be34b4a7000323f4dec7b7437839891515621c772ee57598c8ef7";
const hash = (kind: "sha256" | "sha1", body: Buffer | string) =>
  createHash(kind).update(body).digest("hex");
const blob = (body: Buffer) =>
  hash("sha1", Buffer.concat([Buffer.from(`blob ${body.length}\0`), body]));
const sha = (body: Buffer | string) => hash("sha256", body);

/** Authenticate complete, explicitly formatted finite tables before decoding rows. */
export function loadEmptyVueStarManifest(read: (file: string) => Buffer) {
  const body = read(manifestFile);
  assert.equal(sha(body), emptyVueStarManifestSHA256);
  const stored = JSON.parse(body.toString("utf8")) as Stored;
  assert.equal(stored.version, 1);
  assert.equal(stored.originalSource, "a93e78147b0990c813e965a7868b32dda341eb77");
  assert.equal(stored.reviewedProductSource, "b148ffee7526b45789ec44c16b3e3370e08a5650");
  assert.equal(stored.originalReviewedProductSource, "5968ebfff1fde8d0503d83a414640e18d52e95b2");
  assert.equal(
    stored.correctiveSourceIndexSHA256,
    "4f7d67ecb2bda9ead266c7af35885f4091f8cbfb904094b6e2c688986fb3db63",
  );
  assert.deepEqual(stored.counts, { files: 42, supplementalBodies: 80, preservedBodies: 152 });
  assert.equal(stored.historicalCases, 32);
  assert.equal(stored.historicalPackBodies, 15);
  assert.equal(stored.historicalDistinctBodies, 111);
  assert.equal(
    stored.integrationSourceProofSHA256,
    "dfa6fbbf2eee04a1ab6556035d4429912799814214456833d3e3ab535f34c6b6",
  );
  const tables: Record<Section, Pair[]> = {
    files: [],
    supplementalBodies: [],
    preservedBodies: [],
  };
  assert.equal(stored.parts.length, 7);
  for (const [index, part] of stored.parts.entries()) {
    assert.equal(part.file, prefix + `part-${String(index).padStart(2, "0")}.json`);
    assert.ok(Object.hasOwn(tables, part.section));
    assert.equal(part.start, tables[part.section].length);
    assert.equal(part.mode, "100644");
    const bytes = read(part.file);
    assert.equal(bytes.length, part.bytes);
    assert.equal(sha(bytes), part.sha256);
    assert.equal(blob(bytes), part.blob);
    const rows = JSON.parse(bytes.toString("utf8")) as Pair[];
    assert.ok(Array.isArray(rows));
    assert.equal(rows.length, part.entries);
    for (const row of rows) {
      assert.ok(Array.isArray(row) && row.length === 3);
      const [file, before, after] = row;
      assert.equal(typeof file, "string");
      assert.match(file, /^(?:crates|tests)\/(?:[A-Za-z0-9_.-]+\/)*[A-Za-z0-9_.-]+$/u);
      if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
      assert.match(after, /^[a-f0-9]{64}$/u);
      if (part.section === "preservedBodies") assert.equal(before, after);
      tables[part.section].push(Object.freeze(row));
    }
  }
  for (const section of Object.keys(tables) as Section[]) {
    assert.equal(tables[section].length, stored.counts[section]);
    assert.deepEqual(
      tables[section].map(([file]) => file),
      tables[section].map(([file]) => file).toSorted(),
    );
    Object.freeze(tables[section]);
  }
  assert.equal(
    new Set(
      Object.values(tables)
        .flat()
        .map(([file]) => file),
    ).size,
    122 + 152,
  );
  const part = stored.correction;
  assert.equal(part.file, prefix + "corrective.json");
  assert.equal(part.mode, "100644");
  const bytes = read(part.file);
  assert.equal(bytes.length, part.bytes);
  assert.equal(sha(bytes), part.sha256);
  assert.equal(blob(bytes), part.blob);
  const correction = JSON.parse(bytes.toString("utf8")) as {
    version: number;
    predecessorSource: string;
    reviewedSource: string;
    sourceIndexSHA256: string;
    updates: [string, string | null, string, string][];
    additions: Pair[];
    productionAdditions: string[];
  };
  assert.equal(correction.version, 1);
  assert.equal(correction.predecessorSource, "dd36599600f2c1e622cd05a4d6f73dc6649a8a18");
  assert.equal(correction.reviewedSource, "fe8f4fcf9dbbe750a211cf5df2588356f785f590");
  assert.equal(
    correction.sourceIndexSHA256,
    "680708e5b876e665d6c0dcb4f33f3b3e8db021c17eb4eda2550510a02dad2f23",
  );
  assert.equal(correction.updates.length, 6);
  assert.equal(correction.additions.length, 7);
  const originals = new Map(
    Object.values(tables)
      .flat()
      .map((row) => [row[0], row]),
  );
  const replacements = new Map<string, Pair>();
  for (const [file, before, previous, after] of correction.updates) {
    assert.deepEqual(originals.get(file), [file, before, previous]);
    assert.match(after, /^[a-f0-9]{64}$/u);
    assert.notEqual(previous, after);
    assert(!replacements.has(file));
    replacements.set(file, Object.freeze([file, before, after]));
  }
  assert.deepEqual(correction.productionAdditions, [
    "crates/vize_canon/src/batch/virtual_project/vue_codegen/malformed_setup_tests.rs",
    "crates/vize_canon/tests/empty_vue_star_exports/membership.rs",
  ]);
  const additions = correction.additions.map(([file, before, after]) => {
    assert(!originals.has(file) && !replacements.has(file));
    assert.match(file, /^(?:crates|tests)\/(?:[A-Za-z0-9_.-]+\/)*[A-Za-z0-9_.-]+$/u);
    if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
    assert.match(after, /^[a-f0-9]{64}$/u);
    return Object.freeze([file, before, after] as const);
  });
  assert.equal(new Set(additions.map(([file]) => file)).size, 7);
  const order = (a: Pair, b: Pair) => {
    if (a[0] === b[0]) return 0;
    return a[0] < b[0] ? -1 : 1;
  };
  const current = {
    files: [
      ...tables.files.map((row) => replacements.get(row[0]) ?? row),
      ...additions.filter(([file]) => correction.productionAdditions.includes(file)),
    ].toSorted(order),
    supplementalBodies: [
      ...tables.supplementalBodies.map((row) => replacements.get(row[0]) ?? row),
      ...additions.filter(([file]) => !correction.productionAdditions.includes(file)),
    ].toSorted(order),
    preservedBodies: tables.preservedBodies,
  };
  assert.equal(current.files.length, 44);
  assert.equal(current.supplementalBodies.length, 85);
  assert.equal(current.preservedBodies.length, 152);
  correction.updates.forEach(Object.freeze);
  correction.additions.forEach(Object.freeze);
  Object.freeze(correction.updates);
  Object.freeze(correction.additions);
  Object.freeze(correction.productionAdditions);
  Object.values(current).forEach(Object.freeze);
  const metadata: Pair[] = [
    [manifestFile, null, emptyVueStarManifestSHA256],
    [part.file, null, part.sha256],
    ...stored.parts.map((part): Pair => [part.file, null, part.sha256]),
  ];
  assert.equal(new Set(metadata.map(([file]) => file)).size, 1 + 1 + 7);
  metadata.forEach(Object.freeze);
  Object.freeze(metadata);
  return {
    stored: Object.freeze(stored),
    tables: Object.freeze(tables),
    current: Object.freeze(current),
    correction: Object.freeze(correction),
    metadata,
  };
}

const manifest = loadEmptyVueStarManifest((file) =>
  fs.readFileSync(new URL("../../../" + file, import.meta.url)),
);
export const emptyVueStarFiles: readonly string[] = Object.freeze(
  manifest.current.files.map(([file]) => file),
);
export const emptyVueStarBodyPairs: readonly Pair[] = Object.freeze(
  Object.values(manifest.current).flat(),
);
export const emptyVueStarMetadataPairs: readonly Pair[] = manifest.metadata;
const admitted = new Set(emptyVueStarFiles);
const guarded = new Set(
  [
    ...manifest.current.supplementalBodies,
    ...manifest.current.preservedBodies,
    ...manifest.metadata,
  ].map(([file]) => file),
);

/** Only this complete reviewed source slice; no runtime or performance verdict. */
export function qualifyEmptyVueStarHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: BodyDigest,
) {
  if (new Set(production).size !== production.length) return null;
  if (production.every((file) => alreadyQualified.has(file))) return null;
  if (production.some((file) => guarded.has(file))) return null;
  if (production.some((file) => !admitted.has(file) && !alreadyQualified.has(file))) return null;
  const footprint = production.filter((file) => admitted.has(file)).toSorted();
  if (JSON.stringify(footprint) !== JSON.stringify(emptyVueStarFiles)) return null;
  for (const [file, before, after] of [...emptyVueStarBodyPairs, ...emptyVueStarMetadataPairs])
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "exact-reviewed-empty-vue-star-complete-source",
    manifestSHA256: emptyVueStarManifestSHA256,
    originalSource: manifest.stored.originalSource,
    reviewedProductSource: manifest.correction.reviewedSource,
    predecessorReviewedProductSource: manifest.stored.reviewedProductSource,
    correctivePacketSHA256: manifest.stored.correction.sha256,
    originalReviewedProductSource: manifest.stored.originalReviewedProductSource,
    correctiveSourceIndexSHA256: manifest.correction.sourceIndexSHA256,
    predecessorCorrectiveSourceIndexSHA256: manifest.stored.correctiveSourceIndexSHA256,
    reviewedSourceIndexSHA256: manifest.stored.reviewedSourceIndexSHA256,
    integrationSourceProofSHA256: manifest.stored.integrationSourceProofSHA256,
    completeBodyPairs: 44,
    supplementalBodyPairs: 85,
    preservedBodyPairs: 152,
    sourceMetadataBodyPairs: emptyVueStarMetadataPairs.length,
    historicalCasesPreserved: 32,
    runtimeCredit: false,
    performanceCredit: false,
    files: [...emptyVueStarFiles],
  };
}
