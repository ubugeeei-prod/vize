#!/usr/bin/env node
/**
 * Re-render a recorded tool-comparison artifact with the CURRENT generator.
 *
 * This never measures anything: it reads a results JSON, re-derives only the
 * fields the generator computes from the recorded medians (engine-class
 * classification, primary speedup), and writes the JSON and Markdown back. It
 * exists so a change to how results are *reported* — such as #3283 withdrawing
 * the cross-engine type-check ratio — can be applied to the committed snapshot
 * without inventing replacement timings and without waiting for a fresh
 * Blacksmith run.
 *
 *   node bench/render-results.mjs \
 *     --json bench/results/tool-benchmark-latest.json \
 *     --doc docs/content/architecture/performance-blacksmith.md
 */

import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { createSurface, ENGINE_CLASSES_BY_SURFACE } from "./compare-tools-report.mjs";
import { renderDocument, renderMarkdown } from "./compare-tools.mjs";

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    if (!argv[i].startsWith("--")) continue;
    const key = argv[i].slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith("--")) {
      args[key] = "true";
    } else {
      args[key] = next;
      i++;
    }
  }
  return args;
}

/** Recorded timings are copied verbatim; only derived fields are recomputed. */
export function rerenderData(data) {
  return {
    ...data,
    surfaces: data.surfaces.map((surface) => {
      const engineClasses = ENGINE_CLASSES_BY_SURFACE[surface.id] ?? surface.engineClasses;
      // Drop the fields createSurface derives so a stale value from an older
      // generator cannot survive into the re-rendered artifact.
      const recorded = { ...surface };
      delete recorded.primarySpeedup;
      delete recorded.speedupBaselineId;
      delete recorded.speedupStatus;
      delete recorded.engineClassRanking;
      return createSurface(engineClasses ? { ...recorded, engineClasses } : recorded);
    }),
  };
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  if (!args.json) {
    throw new Error("render-results: --json <results.json> is required");
  }
  const jsonPath = resolve(args.json);
  const data = rerenderData(JSON.parse(readFileSync(jsonPath, "utf8")));

  writeFileSync(jsonPath, `${JSON.stringify(data, null, 2)}\n`);
  if (args.doc) writeFileSync(resolve(args.doc), renderDocument(data));
  if (args.out) writeFileSync(resolve(args.out), renderMarkdown(data));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
