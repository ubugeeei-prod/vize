#!/usr/bin/env node
/**
 * Replay the MIT-licensed pikax/vue-benchmarks typecheck confirmation corpus
 * against Vize plus the external TypeScript/Vue checker rows published by the
 * upstream benchmark (#3283, #3984).
 *
 * Every upstream case is scored against its own meta.json expectations, then
 * compared with this repo's pinned expectation table. Any drift fails the
 * replay: a newly failing case is a regression, and a case that starts
 * passing while the table still expects "fail" is a stale suppression that
 * must be removed. Missing vize or tsgo is an explicit error, never a pass.
 */

import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

import {
  UPSTREAM,
  diffExpectations,
  ensureUpstream,
  parseArgs,
  parseToolList,
  prepareCase,
  requireBinary,
  resolveVuePackageDir,
  rootDir,
  scoreCase,
  workspaceBinCandidates,
} from "./vue-benchmarks-replay-core.mjs";
import { createReplayTools } from "./vue-benchmarks-replay-tools.mjs";

export {
  DEFAULT_REPLAY_TOOLS,
  UPSTREAM,
  diffExpectations,
  scoreCase,
} from "./vue-benchmarks-replay-core.mjs";

export async function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const workRoot = resolve(args["work-root"] ?? join(rootDir, "target", "vue-benchmarks-replay"));
  mkdirSync(workRoot, { recursive: true });

  const selectedTools = parseToolList(args.tools);
  const needsVize = selectedTools.includes("vize");
  const needsVerter = selectedTools.includes("verter-tsc");
  const needsGolar =
    selectedTools.includes("golar-typecheck") || selectedTools.includes("golar-default");
  const needsTsgo = needsVize || needsVerter;

  const vize = needsVize
    ? requireBinary("vize binary", args["vize-bin"], [join(rootDir, "target", "release", "vize")])
    : null;
  const tsgo = needsTsgo
    ? requireBinary("TypeScript 7/Corsa runtime", process.env.VIZE_REPLAY_TSGO, [
        join(
          rootDir,
          "node_modules",
          "@typescript",
          `typescript-${process.platform}-${process.arch}`,
          "lib",
          `tsc${process.platform === "win32" ? ".exe" : ""}`,
        ),
        join(rootDir, "node_modules", ".bin", "tsgo"),
        join(rootDir, "tests", "node_modules", ".bin", "tsgo"),
      ])
    : null;
  const verterTsc = needsVerter
    ? requireBinary("verter-tsc binary", args["verter-bin"], workspaceBinCandidates("verter-tsc"))
    : null;
  const golar = needsGolar
    ? requireBinary("golar binary", args["golar-bin"], workspaceBinCandidates("golar"))
    : null;
  const replayTools = createReplayTools({
    selectedTools,
    vize,
    tsgo,
    verterTsc,
    golar,
  });
  const vuePackageDir = resolveVuePackageDir();
  const upstreamDir = ensureUpstream(args.upstream, workRoot);
  const casesRoot = join(upstreamDir, "tests/confirm/fixtures/typecheck/cases");
  const caseIds = readdirSync(casesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
  if (caseIds.length === 0) throw new Error(`replay: no cases found under ${casesRoot}`);

  const preparedCases = caseIds.map((caseId) => ({
    caseId,
    ...prepareCase(upstreamDir, caseId, workRoot, vuePackageDir),
  }));
  const toolResults = replayTools.map((tool) => {
    const results = preparedCases.map(({ caseId, dest, meta }) => {
      const check = tool.run(dest);
      const combined = `${check.stdout}\n${check.stderr}`;
      return {
        caseId,
        ...scoreCase(meta, check.status, check.report, combined),
      };
    });
    return {
      toolId: tool.id,
      label: tool.label,
      version: tool.version,
      caseCount: results.length,
      results,
    };
  });
  const vizeResults = toolResults.find((tool) => tool.toolId === "vize")?.results ?? [];

  const data = {
    schemaVersion: 1,
    kind: "vue-benchmarks-replay",
    generatedAt: new Date().toISOString(),
    upstream: UPSTREAM,
    versions: {
      vize: vize?.version ?? null,
      tsgo: tsgo?.version ?? null,
      verterTsc: verterTsc?.version ?? null,
      golar: golar?.version ?? null,
    },
    tools: replayTools.map((tool) => ({
      id: tool.id,
      label: tool.label,
      version: tool.version,
    })),
    caseCount: caseIds.length,
    results: vizeResults,
    toolResults,
  };
  if (args.json) writeFileSync(resolve(args.json), `${JSON.stringify(data, null, 2)}\n`);
  for (const toolResult of toolResults) {
    for (const result of toolResult.results) {
      console.log(
        `${result.outcome.padEnd(5)} ${toolResult.toolId.padEnd(15)} ${result.caseId} — ${result.detail}`,
      );
    }
  }

  if (args.expect) {
    const expectations = JSON.parse(readFileSync(resolve(args.expect), "utf8"));
    const problems = diffExpectations(vizeResults, expectations);
    if (problems.length > 0) {
      throw new Error(`replay: expectation drift:\n- ${problems.join("\n- ")}`);
    }
    console.log(`replay: ${vizeResults.length} Vize cases match ${args.expect}`);
  } else {
    console.log("replay: no --expect table given; report-only run");
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
