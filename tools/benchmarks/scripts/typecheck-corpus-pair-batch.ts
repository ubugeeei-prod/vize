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

/** The native tsgo that the hosts' TypeScript ships, also given to `vize check`. */
export function hostCorsaPath(hostsDir: string): string {
  const platform = `typescript-${process.platform}-${process.arch}`;
  return join(hostsDir, "node_modules", "@typescript", platform, "lib", "tsc");
}

export function runBatchCheck(
  buildDir: string,
  projectDir: string,
  corsaPath: string,
): Diagnostic[] {
  const result = spawnSync(
    process.execPath,
    [
      join(buildDir, "npm", "cli", "bin", "vize"),
      "check",
      ".",
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
      env: { ...process.env, NO_COLOR: "1" },
    },
  );
  if (result.error) throw result.error;
  return (normalizeTypecheckResult(result, projectDir, "json") as { diagnostics: Diagnostic[] })
    .diagnostics;
}

export type BatchMapperDiff = {
  corpus: string;
  batch: number;
  mapper: number;
  batchOnly: Diagnostic[];
  mapperOnly: Diagnostic[];
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
    ])
    .join("\n");
}
