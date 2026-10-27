import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import type { SpawnSyncReturns } from "node:child_process";

export function arg(index: number): string {
  const value = process.argv[index + 2];
  assert.ok(value, `missing argument ${index}`);
  return value;
}

export function readJson<T>(file: string | URL): T {
  return JSON.parse(fs.readFileSync(file, "utf8")) as T;
}

export function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

export interface PackageManifest {
  name: string;
  version: string;
}

export interface ProcessPacket {
  status: number | null;
  signal: NodeJS.Signals | null;
  stdout: string;
  stderr: string;
}

export function packet(result: SpawnSyncReturns<string>): ProcessPacket {
  return {
    status: result.status,
    signal: result.signal,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

export interface RuntimeResult {
  rows: Array<{ id: string; passed: boolean; observed: Record<string, unknown> }>;
}

export interface ConfigResult {
  rows: Array<{ id: string; equal: boolean }>;
}

export interface ContributionResult {
  native: string;
  rows: unknown[];
}

export interface LintReport {
  filename: string;
  errorCount: number;
  warningCount: number;
  diagnostics: unknown[];
}

export interface RoleBefore {
  version: string;
  source: string;
  options: Record<string, unknown>;
  result: LintReport;
}

export interface RoleResult {
  positive: LintReport;
  negative: LintReport;
}
