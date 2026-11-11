/**
 * Batch path against mapper path: the head build's `vize check` and its
 * content mapper under tsc see the same corpus with the same TypeScript, so a
 * diagnostic only one of them reports is a projection or filtering defect.
 * Diagnostics are compared by file, authored line:col and code; messages are
 * reported but do not take part in the match, because the two paths flatten
 * elaborations differently.
 */

import { spawnSync } from "node:child_process";
import { join } from "node:path";

import { normalizeTypecheckResult } from "./typecheck-command.mjs";
import type { Diagnostic } from "./typecheck-corpus-pair-parse.ts";
import { TYPECHECK_HOSTS_DIR } from "./typecheck-mapper-hosts.ts";

/** The native tsgo that the hosts' TypeScript ships, also given to `vize check`. */
export function hostCorsaPath(hostsDir: string): string {
  const platform = `typescript-${process.platform}-${process.arch}`;
  return join(hostsDir, "node_modules", "@typescript", platform, "lib", "tsc");
}

/** The independently pinned tsrs host, normalized by Vize to its native image. */
export function hostTsrsPath(hostsDir: string): string {
  return join(hostsDir, "node_modules", "tsrs", "bin", "tsrs");
}

export function runBatchCheck(
  buildDir: string,
  projectDir: string,
  corsaPath: string,
  env: Record<string, string> = {},
): Diagnostic[] {
  const result = spawnSync(
    process.execPath,
    [
      join(buildDir, "npm", "cli", "bin", "vize"),
      // No positional pattern: `.` would add files the tsconfig excludes (tests, stories).
      "check",
      "--quiet",
      "--format",
      "json",
      "--tsconfig",
      join(projectDir, "tsconfig.json"),
      "--corsa-path",
      corsaPath,
    ],
    {
      cwd: projectDir,
      encoding: "utf8",
      maxBuffer: 1 << 30,
      env: { ...process.env, NO_COLOR: "1", ...env },
    },
  );
  if (result.error) throw result.error;
  // The normalizer is untyped JS; its rows are the 6-tuples `Diagnostic` names.
  const { diagnostics } = normalizeTypecheckResult(result, projectDir, "json");
  return diagnostics as unknown as Diagnostic[];
}

export type BatchMapperDiff = {
  corpus: string;
  batch: number;
  mapper: number;
  batchOnly: Diagnostic[];
  mapperOnly: Diagnostic[];
  /** `vize check` at a wider Corsa checker width: what it reports differently from one checker. */
  wide?: { checkers: number; added: Diagnostic[]; removed: Diagnostic[] };
  /** Materialized batch diagnostics under qualified tsrs fast mode versus tsgo. */
  tsrs?: {
    diagnostics: number;
    only: Diagnostic[];
    missing: Diagnostic[];
    wide: { checkers: number; added: Diagnostic[]; removed: Diagnostic[] };
  };
};

const key = ([file, , line, column, code]: Diagnostic) => `${file}:${line}:${column} ${code}`;

/** Multiset difference both ways by (file, line, col, code). */
export function diffBatchMapper(
  corpus: string,
  batch: Diagnostic[],
  mapper: Diagnostic[],
): BatchMapperDiff {
  const only = (from: Diagnostic[], against: Diagnostic[]) => {
    const remaining = new Map<string, number>();
    for (const diagnostic of against) {
      remaining.set(key(diagnostic), (remaining.get(key(diagnostic)) ?? 0) + 1);
    }
    return from.filter((diagnostic) => {
      const left = remaining.get(key(diagnostic)) ?? 0;
      if (left > 0) remaining.set(key(diagnostic), left - 1);
      return left === 0;
    });
  };
  return {
    corpus,
    batch: batch.length,
    mapper: mapper.length,
    batchOnly: only(batch, mapper),
    mapperOnly: only(mapper, batch),
  };
}

export function renderBatchDiff(diffs: BatchMapperDiff[]): string {
  const row = ([file, , line, column, code, message]: Diagnostic) =>
    `| \`${file}\` | ${line}:${column} | ${code} | ${message.split("\n")[0]!.replaceAll("|", "\\|")} |`;
  return diffs
    .flatMap((diff) => [
      `### Batch vs mapper: ${diff.corpus}`,
      "",
      `\`vize check\` reports ${diff.batch} diagnostics, tsc with the Vize mapper ${diff.mapper}; ${diff.batchOnly.length} only in batch, ${diff.mapperOnly.length} only through the mapper.`,
      "",
      ...(diff.wide
        ? [
            `With \`VIZE_CHECKERS=${diff.wide.checkers}\`, vize check adds ${diff.wide.added.length} and drops ${diff.wide.removed.length} diagnostics against one checker.`,
            "",
          ]
        : []),
      ...(diff.tsrs
        ? [
            `The materialized tsrs fast host reports ${diff.tsrs.diagnostics} diagnostics; ${diff.tsrs.only.length} only under tsrs and ${diff.tsrs.missing.length} only under tsgo.`,
            "",
            `At ${diff.tsrs.wide.checkers} checkers, tsrs adds ${diff.tsrs.wide.added.length} and drops ${diff.tsrs.wide.removed.length} diagnostics against its one-checker result.`,
            "",
          ]
        : []),
      ...(["batchOnly", "mapperOnly"] as const).flatMap((side) =>
        diff[side].length === 0
          ? []
          : [
              `#### ${side === "batchOnly" ? "Only in vize check" : "Only through the mapper"}`,
              "",
              "| File | Line:col | Code | Message |",
              "| --- | --- | --- | --- |",
              ...diff[side].map(row),
              "",
            ],
      ),
      ...(["only", "missing"] as const).flatMap((side) =>
        !diff.tsrs || diff.tsrs[side].length === 0
          ? []
          : [
              `#### ${side === "only" ? "Only under tsrs batch" : "Only under tsgo batch"}`,
              "",
              "| File | Line:col | Code | Message |",
              "| --- | --- | --- | --- |",
              ...diff.tsrs[side].map(row),
              "",
            ],
      ),
    ])
    .join("\n");
}

/** The head build's `vize check` on both batch hosts against its tsc mapper diagnostics. */
export function batchVsMapper(
  corpus: string,
  headBuild: string,
  project: string,
  mapper: Diagnostic[],
) {
  const corsa = hostCorsaPath(TYPECHECK_HOSTS_DIR);
  const tsrs = hostTsrsPath(TYPECHECK_HOSTS_DIR);
  const one = runBatchCheck(headBuild, project, corsa, { VIZE_CHECKERS: "1" });
  // Qualify both batch hosts at the one-checker oracle and measured default width.
  const checkers = 8;
  const wide = runBatchCheck(headBuild, project, corsa, { VIZE_CHECKERS: String(checkers) });
  const widened = diffBatchMapper(corpus, wide, one);
  const tsrsOne = runBatchCheck(headBuild, project, tsrs, { VIZE_CHECKERS: "1" });
  const tsrsWide = runBatchCheck(headBuild, project, tsrs, { VIZE_CHECKERS: String(checkers) });
  const tsrsVsTsgo = diffBatchMapper(corpus, tsrsOne, one);
  const tsrsWidened = diffBatchMapper(corpus, tsrsWide, tsrsOne);
  return {
    ...diffBatchMapper(corpus, one, mapper),
    wide: { checkers, added: widened.batchOnly, removed: widened.mapperOnly },
    tsrs: {
      diagnostics: tsrsOne.length,
      only: tsrsVsTsgo.batchOnly,
      missing: tsrsVsTsgo.mapperOnly,
      wide: { checkers, added: tsrsWidened.batchOnly, removed: tsrsWidened.mapperOnly },
    },
  };
}
