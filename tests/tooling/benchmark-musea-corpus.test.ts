import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  MUSEA_CORPUS_FILE_COUNT,
  MUSEA_CORPUS_SUPPORT_FILES,
  createArtFileSource,
  museaCorpusVariantCount,
  writeMuseaCorpus,
} from "../../tools/benchmarks/scripts/musea-corpus.mjs";
import { museaWorkDir, runMuseaStages } from "../../tools/benchmarks/scripts/musea.mjs";

function withTempRoot(fn: (root: string) => void): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-musea-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("the corpus is pinned, so two runs measure the same workload", () => {
  assert.equal(MUSEA_CORPUS_FILE_COUNT, 240);
  assert.equal(museaCorpusVariantCount(MUSEA_CORPUS_FILE_COUNT), 720);
  assert.equal(museaCorpusVariantCount(3), 9);
  assert.deepEqual(MUSEA_CORPUS_SUPPORT_FILES, ["bench-tokens.ts", "styles/bench-tokens.css"]);

  // Same index, same bytes — the property the whole lane's reproducibility
  // rests on. A generator that reached for Math.random or a timestamp would
  // still produce a plausible-looking median over two different workloads.
  assert.equal(createArtFileSource(7), createArtFileSource(7));
  assert.notEqual(createArtFileSource(7), createArtFileSource(8));
});

test("the corpus writes the same bytes into any directory it is given", () => {
  withTempRoot((first) => {
    withTempRoot((second) => {
      const firstRun = writeMuseaCorpus(path.join(first, "corpus"), 6);
      const secondRun = writeMuseaCorpus(path.join(second, "corpus"), 6);

      assert.equal(firstRun.bytes, secondRun.bytes);
      assert.deepEqual(
        firstRun.files.map((file) => path.basename(file)),
        secondRun.files.map((file) => path.basename(file)),
      );
      assert.deepEqual(
        firstRun.files.map((file) => path.basename(file)),
        [
          "BenchComponent0.art.vue",
          "BenchComponent1.art.vue",
          "BenchComponent2.art.vue",
          "BenchComponent3.art.vue",
          "BenchComponent4.art.vue",
          "BenchComponent5.art.vue",
        ],
      );
      for (const [index, file] of firstRun.files.entries()) {
        assert.equal(fs.readFileSync(file, "utf8"), createArtFileSource(index));
        assert.equal(fs.readFileSync(secondRun.files[index], "utf8"), createArtFileSource(index));
      }
      assert.deepEqual(fs.readdirSync(path.join(second, "corpus", "styles")), ["bench-tokens.css"]);
    });
  });
});

test("the corpus exercises both metadata shapes and every generated variant", () => {
  // The plugin reads metadata from the `defineArt` macro and from legacy
  // `<art>` attributes through different code, and only `<art>` interaction
  // attributes reach `extractCustomArtMetadata`. A corpus that dropped any of
  // these would quietly stop measuring the branch it was written to cover.
  const describe = (index: number) => {
    const source = createArtFileSource(index);
    return [
      /^defineArt\("\.\/BenchComponent\d+\.vue", \{$/m.test(source) ? "defineArt" : "legacy-attrs",
      /<art[^>]*\baction-events="[^"]+"/.test(source) ? "interactive" : "static",
      source.match(/<variant\b/g)?.length ?? 0,
      source.match(/<style\b[^>]*>/g)?.length ?? 0,
    ];
  };

  // The full schedule for one period of every axis, so a generator change that
  // drops a branch shows up as a diff rather than a still-passing count.
  assert.deepEqual(
    Array.from({ length: 15 }, (_, index) => describe(index)),
    [
      ["defineArt", "interactive", 2, 2],
      ["defineArt", "static", 3, 2],
      ["defineArt", "static", 4, 2],
      ["defineArt", "interactive", 2, 2],
      ["legacy-attrs", "static", 3, 2],
      ["defineArt", "static", 4, 2],
      ["defineArt", "interactive", 2, 2],
      ["defineArt", "static", 3, 2],
      ["defineArt", "static", 4, 2],
      ["legacy-attrs", "interactive", 2, 2],
      ["defineArt", "static", 3, 2],
      ["defineArt", "static", 4, 2],
      ["defineArt", "interactive", 2, 2],
      ["defineArt", "static", 3, 2],
      ["legacy-attrs", "static", 4, 2],
    ],
  );
  assert.equal(
    Array.from({ length: 20 }, (_, index) => describe(index)[2] as number).reduce(
      (total, count) => total + count,
      0,
    ),
    museaCorpusVariantCount(20),
  );
});

test("the lane's work directory is a fixed path under target, never a temporary one", () => {
  // The plugin derives virtual module ids from absolute file names, so a
  // mkdtemp root changes the generated modules between runs and makes the
  // output check below impossible. os.tmpdir() is not stable under
  // `nix develop`, which sets a fresh TMPDIR per invocation.
  assert.equal(museaWorkDir("/repo"), path.join("/repo", "target", "musea-benchmark", "corpus"));
  assert.equal(museaWorkDir("/repo").startsWith(os.tmpdir()), false);
});

/** The statistic the lane claims to publish, recomputed independently. */
function medianOf(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

test("the lane reports medians, warms up untimed, and alternates stage order", async () => {
  const order: string[] = [];
  const prepared: string[] = [];
  const stages = ["first", "second"].map((id) => ({
    id,
    label: `stage ${id}`,
    units: 4,
    unitLabel: "art files",
    prepare: async () => {
      prepared.push(id);
    },
    run: async () => {
      order.push(id);
      return `${id}-digest`;
    },
  }));

  const results = await runMuseaStages(stages, { runs: 3, warmups: 1 });

  // One warmup pass in declaration order, then three measured passes whose
  // order alternates, so no stage is systematically measured on a cold heap.
  assert.deepEqual(order, [
    "first",
    "second",
    "first",
    "second",
    "second",
    "first",
    "first",
    "second",
  ]);
  // Every pass re-prepares, including the warmup: a `load` stage that reused
  // the previous pass's plugin instance would measure a warm art-file map.
  assert.deepEqual(prepared, order);
  assert.deepEqual(
    results.map((result) => result.id),
    ["first", "second"],
  );
  assert.deepEqual(
    results.map((result) => result.label),
    ["stage first", "stage second"],
  );
  assert.deepEqual(
    results.map((result) => result.digest),
    ["first-digest", "second-digest"],
  );
  assert.deepEqual(
    results.map((result) => result.units),
    [4, 4],
  );
  assert.deepEqual(
    results.map((result) => result.unitLabel),
    ["art files", "art files"],
  );
  for (const result of results) {
    // The warmup pass must not appear in the published samples.
    assert.equal(result.runs.length, 3);
    // Median, not mean and not the last sample.
    assert.equal(result.medianMs, Number(medianOf(result.runs).toFixed(3)));
    assert.equal(result.msPerUnit, Number((result.medianMs / 4).toFixed(6)));
  }
});

test("the lane refuses to publish a median over two different workloads", async () => {
  let pass = 0;
  const stages = [
    {
      id: "drifting",
      label: "stage that stops being reproducible",
      units: 1,
      unitLabel: "art files",
      prepare: async () => {},
      run: async () => `digest-${pass++}`,
    },
  ];

  await assert.rejects(
    () => runMuseaStages(stages, { runs: 2, warmups: 0 }),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.equal(
        error.message,
        "tools/benchmarks/scripts/musea.mjs: drifting produced different output between passes (digest-0 -> digest-1); refusing to report a median over two workloads",
      );
      return true;
    },
  );
});

test("the first measured pass must match every warmup output", async () => {
  let pass = 0;
  const stages = [
    {
      id: "warmup-drift",
      label: "stage that changes after warmup",
      units: 1,
      unitLabel: "build hooks",
      prepare: async () => {},
      run: async () => (pass++ === 0 ? "warmup-output" : "measured-output"),
    },
  ];

  await assert.rejects(
    () => runMuseaStages(stages, { runs: 2, warmups: 1 }),
    /warmup-drift produced different output between passes \(warmup-output -> measured-output\)/,
  );
});

test("an untimed buildStart observable is equivalence-checked", async () => {
  let observation = 0;
  const stages = [
    {
      id: "build-start-observer",
      label: "buildStart with observable graph",
      units: 1,
      unitLabel: "art files",
      prepare: async () => {},
      run: async () => undefined,
      observe: async () => `graph-digest-${observation++}`,
    },
  ];

  await assert.rejects(
    () => runMuseaStages(stages, { runs: 2, warmups: 0 }),
    /build-start-observer produced different output between passes \(graph-digest-0 -> graph-digest-1\)/,
  );
});

test("every measured stage must provide a deterministic digest", async () => {
  const stages = [
    {
      id: "unchecked",
      label: "stage with no observable",
      units: 1,
      unitLabel: "build hooks",
      prepare: async () => {},
      run: async () => undefined,
    },
  ];

  await assert.rejects(
    () => runMuseaStages(stages, { runs: 1, warmups: 0 }),
    /unchecked produced no deterministic digest/,
  );
});
