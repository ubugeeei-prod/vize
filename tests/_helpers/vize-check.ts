import assert from "node:assert/strict";
import {
  spawnSync,
  type SpawnSyncOptionsWithStringEncoding,
  type SpawnSyncReturns,
} from "node:child_process";

import { CORSA_BIN, VIZE_BIN } from "./apps.ts";

export type VizeCheckFileJson = {
  diagnostics: string[];
  file: string;
  virtualTs?: string;
};

export type VizeCheckJson = {
  errorCount: number;
  fileCount: number;
  files: VizeCheckFileJson[];
  programs: Array<{ files: string[]; root: string; tsconfig?: string }>;
  warningCount: number;
};

type RunVizeCheckJsonOptions = {
  allowExitCodes?: readonly number[];
  corsaPath?: string;
  maxBufferBytes?: number;
  showVirtualTs?: boolean;
  spawnSync?: VizeCheckSpawnSync;
  timeoutMs?: number;
  tsconfig?: string;
};

const DEFAULT_ALLOWED_EXIT_CODES = [0, 1] as const;

type VizeCheckSpawnSync = (
  command: string,
  args: readonly string[],
  options: SpawnSyncOptionsWithStringEncoding,
) => SpawnSyncReturns<string>;

export function runVizeCheckJson(
  cwd: string,
  patterns: readonly string[],
  options: RunVizeCheckJsonOptions = {},
): VizeCheckJson {
  const args = buildVizeCheckArgs(patterns, options);
  console.log(`Running: ${formatCommand(VIZE_BIN, args)}`);

  const spawn = options.spawnSync ?? spawnSync;
  const result = spawn(VIZE_BIN, args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: options.maxBufferBytes ?? 128 * 1024 * 1024,
    timeout: options.timeoutMs ?? 120_000,
  });
  if (result.error != null) {
    throw result.error;
  }
  const status = result.status ?? -1;
  const allowedExitCodes = options.allowExitCodes ?? DEFAULT_ALLOWED_EXIT_CODES;
  if (!allowedExitCodes.includes(status)) {
    throw new Error(
      `vize check crashed (exit code ${status}, signal ${result.signal ?? "none"}):\n` +
        result.stderr,
    );
  }
  assert.ok(
    result.stdout.trim().length > 0,
    `vize check should print JSON on stdout; stderr was:\n${result.stderr}`,
  );
  return JSON.parse(result.stdout) as VizeCheckJson;
}

export function buildVizeCheckArgs(
  patterns: readonly string[],
  options: RunVizeCheckJsonOptions = {},
): string[] {
  const args = ["check", ...patterns, "--format", "json", "--quiet"];
  if (options.showVirtualTs) {
    args.push("--show-virtual-ts");
  }
  if (options.tsconfig != null) {
    args.push("--tsconfig", options.tsconfig);
  }
  args.push("--corsa-path", options.corsaPath ?? CORSA_BIN);
  return args;
}

function formatCommand(command: string, args: readonly string[]): string {
  return [command, ...args].map(shellQuote).join(" ");
}

function shellQuote(value: string): string {
  if (/^[A-Za-z0-9_./:=@+-]+$/.test(value)) {
    return value;
  }
  return `'${value.replaceAll("'", "'\\''")}'`;
}
