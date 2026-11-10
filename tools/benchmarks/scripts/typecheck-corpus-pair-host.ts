import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { runTypecheckCommand } from "./typecheck-command.mjs";
import {
  GNU_TIME_FORMAT,
  type Diagnostic,
  type GnuTime,
  type StatEntry,
  parseGnuTime,
  parseHostRun,
} from "./typecheck-corpus-pair-parse.ts";
import type { MAPPER_HOSTS } from "./typecheck-mapper-hosts.ts";

export const GNU_TIME = "/usr/bin/time";
type Host = (typeof MAPPER_HOSTS)[number];
/** Each host at the setting the pair compares: tsc's default, tsrs's `--fast`. */
export const PAIR_HOST_ARGS: Record<Host["id"], string[]> = { tsc: [], tsrs: ["--fast"] };

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
  host: Host,
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
  ) as { ms: number; cpuMs: number | null; status: number | null; stdout: string; stderr: string };
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
