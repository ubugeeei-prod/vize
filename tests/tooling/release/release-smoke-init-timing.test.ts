import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  DIAGNOSTIC_QUIET_MS,
  packedDownloadBytes,
  packedSmokeTiming,
  readLspDiagnosticSettle,
  recordPackedSmokeTiming,
  writeLspDiagnosticSettle,
} from "../../../tools/support/compat/npm/smoke-release-init-timing.ts";
import { readRepoFile } from "../support/github-workflows.ts";

const cell = {
  projectShape: "vite-vue-ts",
  packageManager: "pnpm",
  host: "linux-x64",
  installMs: 12.5,
  firstCheckMs: 40,
  downloadBytes: 128,
  downloadBytesComplete: true,
  lspSession: false,
  lspDiagnosticsNotifications: 0,
  lspDiagnosticsSettled: false,
  lspDiagnosticsSettleMs: null,
  diagnosticQuietMs: null,
};

test("packed smoke timing keeps install, first check, download size, and host", () => {
  const timing = packedSmokeTiming(cell);
  assert.equal(timing.schema, "vize.packedSmokeTiming");
  assert.equal(timing.version, 1);
  assert.equal(timing.installMs, 12.5);
  assert.equal(timing.firstCheckMs, 40);
  assert.equal(timing.downloadBytes, 128);
  assert.equal(timing.host, "linux-x64");
  assert.equal(timing.lspDiagnosticsSettleMs, null);
});

test("a settled LSP diagnostic pass records the quiet window", () => {
  const timing = packedSmokeTiming({
    ...cell,
    lspSession: true,
    lspDiagnosticsNotifications: 2,
    lspDiagnosticsSettled: true,
    lspDiagnosticsSettleMs: 80,
    diagnosticQuietMs: DIAGNOSTIC_QUIET_MS,
  });
  assert.equal(timing.lspDiagnosticsSettleMs, 80);
  assert.equal(timing.diagnosticQuietMs, DIAGNOSTIC_QUIET_MS);
});

test("timing refuses non-finite durations and a settle that did not happen", () => {
  for (const patch of [
    { installMs: Number.NaN },
    { firstCheckMs: -1 },
    { downloadBytes: Number.POSITIVE_INFINITY },
    { downloadBytes: 1.5 },
    { host: "Linux x64" },
    { lspDiagnosticsSettled: true, lspDiagnosticsSettleMs: null },
    {
      lspSession: true,
      lspDiagnosticsSettled: true,
      lspDiagnosticsNotifications: 0,
      lspDiagnosticsSettleMs: 10,
      diagnosticQuietMs: DIAGNOSTIC_QUIET_MS,
    },
    { lspDiagnosticsSettleMs: 10 },
    { lspSession: true, diagnosticQuietMs: null },
  ]) {
    assert.throws(() => packedSmokeTiming({ ...cell, ...patch }));
  }
});

test("download size sums packed tarballs and reports a missing package", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-download-"));
  try {
    const cli = path.join(root, "vize.tgz");
    const vite = path.join(root, "vite.tgz");
    fs.writeFileSync(cli, "abc");
    fs.writeFileSync(vite, "defg");
    const packed = new Map([
      ["vize", cli],
      ["@vizejs/vite-plugin", vite],
    ]);
    assert.deepEqual(packedDownloadBytes(packed, ["vize", "@vizejs/vite-plugin"]), {
      downloadBytes: 7,
      downloadBytesComplete: true,
    });
    assert.deepEqual(packedDownloadBytes(packed, ["vize", "missing"]), {
      downloadBytes: 3,
      downloadBytesComplete: false,
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the timing artifact is one JSON object per cell", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-timing-"));
  const file = path.join(root, "nested", "timing.jsonl");
  const previous = process.env.VIZE_SMOKE_TIMING_FILE;
  process.env.VIZE_SMOKE_TIMING_FILE = file;
  try {
    recordPackedSmokeTiming(cell);
    recordPackedSmokeTiming({ ...cell, packageManager: "npm", installMs: 3 });
    const lines = fs.readFileSync(file, "utf8").trim().split("\n");
    assert.equal(lines.length, 2);
    assert.equal(JSON.parse(lines[0]).packageManager, "pnpm");
    assert.equal(JSON.parse(lines[1]).installMs, 3);
  } finally {
    if (previous === undefined) delete process.env.VIZE_SMOKE_TIMING_FILE;
    else process.env.VIZE_SMOKE_TIMING_FILE = previous;
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("an LSP settle file is readable once and rejects an invented settle", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-lsp-settle-"));
  try {
    writeLspDiagnosticSettle(root, {
      notifications: 1,
      settleMs: 15,
      settled: true,
      diagnosticQuietMs: DIAGNOSTIC_QUIET_MS,
    });
    assert.equal(readLspDiagnosticSettle(root).settleMs, 15);
    assert.equal(fs.readdirSync(root).length, 0);
    assert.throws(() =>
      writeLspDiagnosticSettle(root, {
        notifications: 0,
        settleMs: 15,
        settled: true,
        diagnosticQuietMs: DIAGNOSTIC_QUIET_MS,
      }),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("release smokes retain the timing artifact for every host", () => {
  const release = readRepoFile(".github", "workflows", "release.yml");
  const native = readRepoFile(".github", "workflows", "native-smoke.yml");
  for (const workflow of [release, native]) {
    assert.match(workflow, /VIZE_SMOKE_TIMING_FILE: \$\{\{ runner\.temp \}\}\/packed-smoke-timing\.jsonl/);
    assert.match(workflow, /name: Retain packed smoke timing/);
    assert.match(workflow, /if-no-files-found: warn/);
  }
  assert.match(release, /name: packed-smoke-timing\n/);
  assert.match(
    native,
    /name: packed-smoke-timing-\$\{\{ matrix\.platform\.target \}\}-node\$\{\{ matrix\.node-version \}\}/,
  );
});

test("fresh and workspace cells both record the timing receipt", () => {
  const fresh = readRepoFile("tools", "support", "compat", "npm", "smoke-release-init-fresh.mjs");
  const workspace = readRepoFile(
    "tools",
    "support",
    "compat",
    "npm",
    "smoke-release-init-workspace.ts",
  );
  assert.match(fresh, /recordPackedSmokeTiming\(/);
  assert.match(workspace, /recordPackedSmokeTiming\(/);
  assert.match(fresh, /const installStarted = performance\.now\(\)/);
  assert.match(fresh, /const checkStarted = performance\.now\(\)/);
});
