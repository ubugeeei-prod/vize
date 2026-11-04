/**
 * Timing receipt for one packed fresh-project smoke cell (#11510).
 *
 * Install time, the first `vize check`, packed download size, and LSP
 * diagnostic settle are recorded per package manager and host. A missing LSP
 * publication stays null. It is not reported as a settle time.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export const DIAGNOSTIC_QUIET_MS = 400;
const SETTLE_FILE = ".vize-smoke-lsp-settle.json";
const hostPattern = /^[a-z0-9]+-[a-z0-9]+$/u;

export interface PackedSmokeTiming {
  schema: "vize.packedSmokeTiming";
  version: 1;
  projectShape: string;
  packageManager: string;
  host: string;
  installMs: number;
  firstCheckMs: number;
  downloadBytes: number;
  downloadBytesComplete: boolean;
  lspSession: boolean;
  lspDiagnosticsNotifications: number;
  lspDiagnosticsSettled: boolean;
  lspDiagnosticsSettleMs: number | null;
  diagnosticQuietMs: number | null;
}

export interface LspDiagnosticSettle {
  notifications: number;
  settleMs: number | null;
  settled: boolean;
  diagnosticQuietMs: number;
}

function finiteDuration(value: number, label: string): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`${label} must be a finite non-negative number`);
  }
  return value;
}

function identity(value: string, label: string): string {
  if (value.trim() === "" || /\s/u.test(value) || value.includes("\0")) {
    throw new Error(`${label} must be a single token`);
  }
  return value;
}

export function packedDownloadBytes(
  packed: ReadonlyMap<string, string>,
  planned: readonly string[],
): { downloadBytes: number; downloadBytesComplete: boolean } {
  let downloadBytes = 0;
  let downloadBytesComplete = true;
  for (const name of planned) {
    const tarball = packed.get(name);
    if (tarball === undefined) {
      downloadBytesComplete = false;
      continue;
    }
    const size = fs.statSync(tarball).size;
    downloadBytes += finiteDuration(size, `${name} download size`);
    if (!Number.isInteger(size)) throw new Error(`${name} download size must be an integer`);
  }
  return { downloadBytes, downloadBytesComplete };
}

export function packedSmokeTiming(sample: {
  projectShape: string;
  packageManager: string;
  host: string;
  installMs: number;
  firstCheckMs: number;
  downloadBytes: number;
  downloadBytesComplete: boolean;
  lspSession: boolean;
  lspDiagnosticsNotifications: number;
  lspDiagnosticsSettled: boolean;
  lspDiagnosticsSettleMs: number | null;
  diagnosticQuietMs: number | null;
}): PackedSmokeTiming {
  const installMs = finiteDuration(sample.installMs, "installMs");
  const firstCheckMs = finiteDuration(sample.firstCheckMs, "firstCheckMs");
  const downloadBytes = finiteDuration(sample.downloadBytes, "downloadBytes");
  if (!Number.isInteger(downloadBytes)) throw new Error("downloadBytes must be an integer");
  const notifications = sample.lspDiagnosticsNotifications;
  if (!Number.isInteger(notifications) || notifications < 0) {
    throw new Error("lspDiagnosticsNotifications must be a non-negative integer");
  }
  const host = identity(sample.host, "host");
  if (!hostPattern.test(host)) throw new Error(`unsupported host ${host}`);
  if (typeof sample.downloadBytesComplete !== "boolean" || typeof sample.lspSession !== "boolean") {
    throw new Error("timing flags must be booleans");
  }
  if (
    typeof sample.lspDiagnosticsSettled !== "boolean" ||
    (sample.diagnosticQuietMs !== null && typeof sample.diagnosticQuietMs !== "number")
  ) {
    throw new Error("diagnostic settle flags must be booleans");
  }
  if (sample.lspDiagnosticsSettled) {
    if (sample.lspDiagnosticsSettleMs === null) {
      throw new Error("settled diagnostics need a settle time");
    }
    finiteDuration(sample.lspDiagnosticsSettleMs, "lspDiagnosticsSettleMs");
    if (notifications < 1) throw new Error("settled diagnostics need a notification");
    if (sample.diagnosticQuietMs !== DIAGNOSTIC_QUIET_MS) {
      throw new Error("settled diagnostics must record the quiet window");
    }
  } else if (sample.lspDiagnosticsSettleMs !== null) {
    throw new Error("unsettled diagnostics have no settle time");
  }
  if (!sample.lspSession) {
    assert.equal(notifications, 0);
    assert.equal(sample.lspDiagnosticsSettled, false);
    assert.equal(sample.diagnosticQuietMs, null);
  } else if (sample.diagnosticQuietMs !== DIAGNOSTIC_QUIET_MS) {
    throw new Error("an LSP session must record the quiet window");
  }
  return {
    schema: "vize.packedSmokeTiming",
    version: 1,
    projectShape: identity(sample.projectShape, "projectShape"),
    packageManager: identity(sample.packageManager, "packageManager"),
    host,
    installMs,
    firstCheckMs,
    downloadBytes,
    downloadBytesComplete: sample.downloadBytesComplete,
    lspSession: sample.lspSession,
    lspDiagnosticsNotifications: notifications,
    lspDiagnosticsSettled: sample.lspDiagnosticsSettled,
    lspDiagnosticsSettleMs: sample.lspDiagnosticsSettleMs,
    diagnosticQuietMs: sample.diagnosticQuietMs,
  };
}

export function recordPackedSmokeTiming(
  sample: Parameters<typeof packedSmokeTiming>[0],
): PackedSmokeTiming {
  const timing = packedSmokeTiming(sample);
  const line = `${JSON.stringify(timing)}\n`;
  console.log(`runtime: packed smoke timing ${JSON.stringify(timing)}`);
  const file = process.env.VIZE_SMOKE_TIMING_FILE;
  if (file) {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.appendFileSync(file, line);
  }
  return timing;
}

function assertLspDiagnosticSettle(settle: LspDiagnosticSettle): void {
  if (settle.diagnosticQuietMs !== DIAGNOSTIC_QUIET_MS) {
    throw new Error("LSP settle must use the smoke quiet window");
  }
  if (!Number.isInteger(settle.notifications) || settle.notifications < 0) {
    throw new Error("LSP settle notifications must be a non-negative integer");
  }
  if (settle.settled) {
    if (settle.settleMs === null || !Number.isFinite(settle.settleMs) || settle.settleMs < 0) {
      throw new Error("settled LSP diagnostics need a finite settle time");
    }
    if (settle.notifications < 1) throw new Error("settled LSP diagnostics need a notification");
  } else if (settle.settleMs !== null) {
    throw new Error("unsettled LSP diagnostics have no settle time");
  }
}

export function writeLspDiagnosticSettle(projectRoot: string, settle: LspDiagnosticSettle): void {
  assertLspDiagnosticSettle(settle);
  fs.writeFileSync(path.join(projectRoot, SETTLE_FILE), `${JSON.stringify(settle)}\n`);
}

export function readLspDiagnosticSettle(projectRoot: string): LspDiagnosticSettle {
  const filename = path.join(projectRoot, SETTLE_FILE);
  const settle = JSON.parse(fs.readFileSync(filename, "utf8")) as LspDiagnosticSettle;
  assertLspDiagnosticSettle(settle);
  fs.rmSync(filename);
  return settle;
}
