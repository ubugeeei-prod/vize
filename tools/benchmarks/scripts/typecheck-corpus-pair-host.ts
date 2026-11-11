import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { runTypecheckCommand } from "./typecheck-command.mjs";
import {
  GNU_TIME_FORMAT,
  type Diagnostic,
  type GnuTime,
  type StatEntry,
  parseGnuTime,
  parseHostRun,
} from "./typecheck-corpus-pair-parse.ts";
import type { MapperHosts } from "./typecheck-mapper-hosts.ts";

export const GNU_TIME = "/usr/bin/time";
/** tsrs 0.23, pinned beside typecheck-hosts' 0.20.1 for the target lane. */
export const TSRS_023_DIR = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "typecheck-hosts-tsrs-0.23",
);
/**
 * The pair's host lanes: tsc at its default; typecheck-hosts' tsrs 0.20.1 at
 * `--fast` (lane id `tsrs`, as every earlier pair run named it) and at its
 * default; and tsrs 0.23 at `--fast`, the release the 500 ms target names.
 */
export const PAIR_HOSTS = [
  { id: "tsc", bin: "tsc", label: "tsc", args: [] },
  { id: "tsrs", bin: "tsrs", label: "tsrs 0.20 --fast", args: ["--fast"] },
  { id: "tsrs-default", bin: "tsrs", label: "tsrs 0.20", args: [] },
  { id: "tsrs-0.23-fast", bin: "tsrs023", label: "tsrs 0.23 --fast", args: ["--fast"] },
] as const satisfies ReadonlyArray<{
  id: string;
  bin: "tsc" | "tsrs" | "tsrs023";
  label: string;
  args: string[];
}>;
export type PairHost = (typeof PAIR_HOSTS)[number];
export type PairHostBins = Record<PairHost["bin"], string>;

/** Every pair host binary; a missing install is an error. */
export function pairHostBins(hosts: MapperHosts, tsrs023Dir = TSRS_023_DIR): PairHostBins {
  const tsrs023 = join(tsrs023Dir, "node_modules", "tsrs", "bin", "tsrs");
  if (!existsSync(tsrs023)) {
    throw new Error(
      `tsrs 0.23 not found at ${tsrs023}. Run \`npm ci --prefix ${tsrs023Dir}\` first.`,
    );
  }
  return { tsc: hosts.tsc, tsrs: hosts.tsrs, tsrs023 };
}
export const PAIR_HOST_ARGS = Object.fromEntries(
  PAIR_HOSTS.map((host) => [host.id, [...host.args]]),
) as Record<PairHost["id"], string[]>;

export type HostRun = {
  wallMs: number;
  cpuMs: number;
  time: GnuTime;
  status: number | null;
  stats: StatEntry[];
  diagnostics: Diagnostic[];
  fingerprint: string;
};

export function requireGnuTime(): string {
  const probe = spawnSync(GNU_TIME, ["--version"], { encoding: "utf8" });
  const version = `${probe.stdout ?? ""}${probe.stderr ?? ""}`.trim().split("\n")[0] ?? "";
  if (probe.status !== 0 || !/GNU time/iu.test(version)) {
    throw new Error(`max RSS needs GNU time at ${GNU_TIME}; found: ${version || probe.error}`);
  }
  return version;
}

export function packageVersion(dir: string, name: string): string {
  const manifest = join(dir, "node_modules", name, "package.json");
  return (JSON.parse(readFileSync(manifest, "utf8")) as { version: string }).version;
}

export const hostArgs = (
  host: { id: PairHost["id"] },
  cwd: string,
  tsconfig: string,
  extraArgs: string[] = [],
): string[] => [
  "--noEmit",
  ...PAIR_HOST_ARGS[host.id],
  "--runExternalCode",
  "--extendedDiagnostics",
  "--pretty",
  "false",
  ...extraArgs,
  "-p",
  join(cwd, tsconfig),
];

export function hostRun(
  command: string[],
  cwd: string,
  tsconfig: string,
  timeFile: string,
  env: Record<string, string> = {},
): HostRun {
  const result = runTypecheckCommand(
    GNU_TIME,
    ["-q", "-f", GNU_TIME_FORMAT, "-o", timeFile, ...command],
    { cwd, env },
  ) as {
    ms: number;
    cpuMs: number | null;
    status: number | null;
    stdout: string;
    stderr: string;
  };
  if (result.cpuMs == null) throw new Error("process-tree CPU needs Linux /proc/self/stat");
  const time = parseGnuTime(readFileSync(timeFile, "utf8"));
  if (time.status !== result.status) {
    throw new Error(`GNU time saw status ${time.status}, the runner saw ${result.status}`);
  }
  try {
    return {
      wallMs: result.ms,
      cpuMs: result.cpuMs,
      time,
      status: result.status,
      ...parseHostRun(result, cwd),
    };
  } catch (error) {
    throw new Error(
      `${tsconfig} in ${cwd}: ${(error as Error).message}\nstderr:\n${result.stderr.slice(-4000)}`,
      {
        cause: error,
      },
    );
  }
}
