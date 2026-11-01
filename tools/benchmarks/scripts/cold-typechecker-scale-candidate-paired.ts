/** Finite two-candidate frontdoor, used only by a frozen existing paired-comparison controller ref. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { verifyColdScaleBuildPair } from "./cold-typechecker-scale-build.ts";
import { prepareStrictOutput } from "./cold-typechecker-scale-strict-pair.ts";
import {
  verifyCandidateSources,
  type CandidateSources,
} from "./cold-typechecker-scale-candidate-source.ts";
import {
  candidateNormalCommands,
  type CandidateBuild,
} from "./cold-typechecker-scale-candidate-normal.ts";
import { candidateProfileCommands } from "./cold-typechecker-scale-candidate-profile.ts";
import { verifyCandidateCommandPopulation } from "./cold-typechecker-scale-candidate-law.ts";

const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const read = (file: string) => JSON.parse(fs.readFileSync(file, "utf8"));
type Role = "base" | "head";

export async function pairedCandidateComparison(options: {
  provider: string;
  out: string;
  sourceManifest: string;
  normal: Record<Role, string>;
  allocation: Record<Role, string>;
  custody: Record<Role, string>;
}) {
  const { provider, out } = options;
  prepareStrictOutput(out);
  const ids: string[] = [];
  const write = (name: string, value: unknown, fresh = true) =>
    fs.writeFileSync(path.join(out, name), JSON.stringify(value, null, 2) + "\n", {
      flag: fresh ? "wx" : "w",
    });
  const enter = (id: string) => {
    ids.push(id);
    const population = verifyCandidateCommandPopulation(ids, false);
    write(
      "command-population.json",
      {
        ids,
        ...population,
        classification: "logical target stages; OS/native descendant counts are separate",
      },
      false,
    );
  };
  try {
    assert.equal(process.platform, "linux");
    assert.equal(process.arch, "x64");
    fs.copyFileSync(options.sourceManifest, path.join(out, "source-manifest.json"));
    const frozen = read(path.join(out, "source-manifest.json")) as CandidateSources;
    assert.equal(
      hash(path.join(provider, "package-lock.json")),
      "9c2974672b5f5358e643276e6ec98c6084681ef609e8f25922ba4aa7adb0f0a6",
    );
    assert.equal(
      hash(path.join(provider, "package.json")),
      "bb62ff5452cbfd7866d232c6e1a78d1d7b6d82ec5f6cbf67e08aa6deff9c324e",
    );
    for (const key of [
      "VIZE_COLD_NATIVE",
      "VIZE_COLD_CAPTURE",
      "VIZE_COLD_PROJECT",
      "VIZE_COLD_PROVIDER",
      "VIZE_CHILD_ALLOCATION_OBSERVATION",
      "VIZE_ROUTE_MEMORY_OBSERVATION",
      "VIZE_OWNED_READ_BUFFER_TRACE",
    ])
      assert.equal(
        process.env[key],
        undefined,
        "observers must not leak into a primary environment",
      );
    const normal = {} as Record<Role, CandidateBuild & { receipt: string }>;
    const custody = {} as Record<Role, CandidateBuild & { receipt: string }>;
    const allocation = {} as Record<Role, { directory: string; build: CandidateBuild }>;
    for (const role of ["base", "head"] as const) {
      normal[role] = { ...read(options.normal[role]), receipt: options.normal[role] };
      custody[role] = { ...read(options.custody[role]), receipt: options.custody[role] };
      allocation[role] = {
        directory: path.dirname(options.allocation[role]),
        build: read(options.allocation[role]),
      };
      assert.equal(normal[role].sourceSha, frozen[role].sha);
      assert.equal(normal[role].treeSha, frozen[role].tree);
      assert.equal(normal[role].allocation, false);
      assert.deepEqual(normal[role].cargoArtifact.features.toSorted(), [
        "default",
        "glyph",
        "maestro",
      ]);
      assert.equal(hash(normal[role].binary), normal[role].binarySha256);
    }
    verifyColdScaleBuildPair(options.normal.base, options.normal.head, process.cwd());
    const source = verifyCandidateSources(
      { base: normal.base.sourceRoot, head: normal.head.sourceRoot },
      provider,
      frozen,
    );
    // Source assets are retained as source bytes; executable images are metadata-only.
    write("source-closure.json", source);
    const first = await candidateNormalCommands({
      provider,
      out,
      frozen,
      normal,
      allocation,
      assets: source.assets,
      enter,
    });
    if (!first.eligible) {
      write("ineligible.json", {
        ...first,
        population: verifyCandidateCommandPopulation(ids, false),
        comparisonQualified: false,
        primaryTimings: 0,
      });
      return { comparisonQualified: false, reason: first.reason };
    }
    const native = candidateProfileCommands({
      provider,
      out,
      common: first.common,
      corpus: first.corpus,
      frozen,
      normal,
      custody,
      helpers: first.helperTransition,
      primaryRaw: first.primaryRaw,
      enter,
    });
    for (const role of ["base", "head"] as const) {
      assert.equal(hash(normal[role].binary), normal[role].binarySha256);
      assert.equal(hash(custody[role].binary), custody[role].binarySha256);
      assert.equal(hash(allocation[role].build.binary), allocation[role].build.binarySha256);
    }
    assert.equal(
      hash(path.join(provider, "package-lock.json")),
      "9c2974672b5f5358e643276e6ec98c6084681ef609e8f25922ba4aa7adb0f0a6",
    );
    const summary = {
      source,
      native,
      allocation: first.allocationReports,
      samples: first.samples,
      statistics: first.statistics,
      population: verifyCandidateCommandPopulation(ids, true),
      order: ["AB", "BA", "BA", "AB"],
      comparisonQualified: true,
      osPageCache: "uncontrolled; no flush; fixture/build/setup warm pages",
      ownedAppCacheNamespace: "absent before each primary spawn",
      sampleScope:
        "one corpus; eight normal primary commands/four descriptive pairs; no precision/significance/universal-scale claim",
      cpuScope:
        "GNU reported command user+system CPU at10ms resolution, not root-vs-native ownership",
      rssScopes:
        "GNU child HWM, sampled actual CLI root and concurrent live descendant windows are distinct; never sum PID peaks",
      goMemoryScope:
        "alloc_objects/alloc_space/inuse_objects/inuse_space sampled weights, not RSS/live heap/forcedGC",
      nativeLoadedFiles: "unobserved",
      noDiagnosticNormalization: true,
      oldBufferExperimentRepeated: false,
    };
    write("paired-comparison.json", summary);
    return summary;
  } catch (error) {
    write("controller-failure.json", {
      error: String(error),
      stack: error instanceof Error ? error.stack : null,
      population: ids.length
        ? verifyCandidateCommandPopulation(ids, false)
        : { outerTargetStages: 0 },
      comparisonQualified: false,
      scope:
        "all partial raw observations retained; no favourable unchanged retry or completed-comparison claim",
    });
    throw error;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: Object.fromEntries(
      [
        "provider",
        "out",
        "case",
        "base",
        "head",
        "allocation-base",
        "allocation-head",
        "custody-base",
        "custody-head",
      ].map((key) => [key, { type: "string" as const }]),
    ),
  });
  for (const key of [
    "provider",
    "out",
    "case",
    "base",
    "head",
    "allocation-base",
    "allocation-head",
    "custody-base",
    "custody-head",
  ])
    assert(typeof values[key] === "string");
  const absolute = (key: string) => path.resolve(values[key] as string);
  await pairedCandidateComparison({
    provider: absolute("provider"),
    out: absolute("out"),
    sourceManifest: absolute("case"),
    normal: { base: absolute("base"), head: absolute("head") },
    allocation: { base: absolute("allocation-base"), head: absolute("allocation-head") },
    custody: { base: absolute("custody-base"), head: absolute("custody-head") },
  });
}
