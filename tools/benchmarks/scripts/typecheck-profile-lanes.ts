/**
 * The profiled lanes: how each runs one corpus project for one build, and how
 * its output yields the diagnostic count and the host's own phase statistics.
 */

import { spawnSync } from "node:child_process";
import { join } from "node:path";

import { normalizeTypecheckResult } from "./typecheck-command.mjs";
import { parseHostRun } from "./typecheck-corpus-pair-parse.ts";
import { type MapperHosts, TYPECHECK_HOSTS_DIR } from "./typecheck-mapper-hosts.ts";
import type { LaneShape } from "./typecheck-profile-rules.ts";

export type RunResult = { status: number | null; stdout: string; stderr: string; wallMs: number };

const HOST_FLAGS = ["--noEmit", "--runExternalCode", "--extendedDiagnostics", "--pretty", "false"];
const corsaPath = () =>
  join(
    TYPECHECK_HOSTS_DIR,
    "node_modules",
    "@typescript",
    `typescript-${process.platform}-${process.arch}`,
    "lib",
    "tsc",
  );

export type LaneContext = { build: string; project: string; hosts: MapperHosts };
export type Lane = {
  label: string;
  shape: LaneShape;
  callGraph: "fp" | "dwarf";
  /** Also write tsgo's own CPU profile from the untimed run. */
  pprof?: boolean;
  command: (context: LaneContext, extra: string[]) => string[];
  diagnostics: (result: RunResult, project: string) => number;
  stats?: (result: RunResult, project: string) => Array<[string, string]>;
};

const hostDiagnostics = (result: RunResult, project: string) =>
  parseHostRun(result, project).diagnostics.length;
const hostStats = (result: RunResult, project: string) => parseHostRun(result, project).stats;
const vizeMapper = (project: string) => join(project, "tsconfig.vize-mapper.json");
const tsrsLane = (callGraph: Lane["callGraph"], label: string): Lane => ({
  label,
  shape: "host",
  callGraph,
  command: ({ hosts, project }, extra) => [
    process.execPath,
    hosts.tsrs,
    ...HOST_FLAGS,
    "--fast",
    ...extra,
    "-p",
    vizeMapper(project),
  ],
  diagnostics: hostDiagnostics,
  stats: hostStats,
});

/** Lanes by id; a lane added here is selectable with `--lanes`. */
export const LANES: Record<string, Lane> = {
  tsc: {
    label: "tsc (tsgo) + Vize mapper",
    shape: "host",
    callGraph: "fp",
    pprof: true,
    command: ({ hosts, project }, extra) => [
      process.execPath,
      hosts.tsc,
      ...HOST_FLAGS,
      ...extra,
      "-p",
      vizeMapper(project),
    ],
    diagnostics: hostDiagnostics,
    stats: hostStats,
  },
  // tsrs ships stripped, BOLT-laid-out and without frame pointers. Frame
  // pointers still name every leaf (self time per function) and keep the
  // mapper's stacks whole; dwarf is the variant that tries to unwind tsrs.
  tsrs: tsrsLane("fp", "tsrs --fast + Vize mapper"),
  "tsrs-dwarf": tsrsLane("dwarf", "tsrs --fast + Vize mapper (dwarf unwinding)"),
  check: {
    label: "vize check (Corsa/tsgo)",
    shape: "check",
    callGraph: "fp",
    command: ({ build, project }) => [
      process.execPath,
      join(build, "npm", "cli", "bin", "vize"),
      "check",
      "--quiet",
      "--format",
      "json",
      "--tsconfig",
      join(project, "tsconfig.json"),
      "--corsa-path",
      corsaPath(),
    ],
    diagnostics: (result, project) =>
      (normalizeTypecheckResult(result, project, "json") as { diagnostics: unknown[] }).diagnostics
        .length,
  },
};

/** One synchronous run with its wall time; the caller judges the status. */
export function run(command: string[], cwd: string, env: Record<string, string> = {}): RunResult {
  const start = performance.now();
  const result = spawnSync(command[0]!, command.slice(1), {
    cwd,
    encoding: "utf8",
    maxBuffer: 1 << 30,
    env: { ...process.env, NO_COLOR: "1", ...env },
  });
  if (result.error) throw result.error;
  return {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
    wallMs: performance.now() - start,
  };
}
