/** Retained physical callchains for a hash leaf; no source-line lookup or new recording. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { boundedCommand, digest } from "./source-symbolizer-render.ts";
export function rustPidScope(trace: string, binary: string) {
  const rows: { pid: string; image: string }[] = [];
  for (const line of trace.split("\n")) {
    const match = /^\s*(\d+)\/\d+\s+\d+\.\d{9}:\s+\d+\s+[0-9a-f]+\s+\((.+)\)$/iu.exec(line);
    if (match) rows.push({ pid: match[1], image: match[2].replace(/\+0x[0-9a-f]+$/iu, "") });
  }
  const owned = rows.filter((row) => row.image === binary);
  const pids = [...new Set(owned.map((row) => row.pid))];
  return {
    pids,
    rustImageSamples: owned.length,
    selectedPidSamples: rows.filter((row) => pids.includes(row.pid)).length,
  };
}
export function collectRustCallchains(options: {
  binary: string;
  data: string;
  trace: string;
  perf: string;
  out: string;
  binarySha256: string;
  dataSha256: string;
  source: string;
}) {
  assert.ok(!existsSync(options.out));
  mkdirSync(options.out, { recursive: true });
  assert.equal(digest(options.binary), options.binarySha256);
  assert.equal(digest(options.data), options.dataSha256);
  const scope = rustPidScope(readFileSync(options.trace, "utf8"), options.binary);
  assert.equal(scope.rustImageSamples, 389);
  assert.ok(scope.pids.length > 0);
  const command = boundedCommand(
    join(options.out, "physical-stacks"),
    options.perf,
    [
      "script",
      "--no-inline",
      "--ns",
      "--fields",
      "pid,tid,time,period,ip,sym,dso",
      "--pid",
      scope.pids.join(","),
      "--input",
      options.data,
    ],
    30_000,
  );
  const unchanged =
    digest(options.binary) === options.binarySha256 && digest(options.data) === options.dataSha256;
  const result = {
    schemaVersion: 1,
    rawCollected: command.status === 0 && unchanged,
    source: options.source,
    binarySha256: options.binarySha256,
    dataSha256: options.dataSha256,
    unchanged,
    scope,
    command: {
      executable: command.executable,
      args: command.args,
      status: command.status,
      error: command.error,
      wallMs: command.wallMs,
    },
    capturedUserStackBytes: 4096,
    callerProfileQualified: false,
    sourceLineQualified: false,
    callCounts: null,
    hardwareInstructions: null,
    metric: "retained Rust-PID physical symbol stacks, awaiting exact leaf/caller census",
    limitation: "captured DWARF stack may be truncated; an unresolved frame never proves absence",
  };
  writeFileSync(join(options.out, "collection.json"), JSON.stringify(result, null, 2) + "\n");
  assert.ok(
    result.rawCollected,
    "Rust callgraph reconstruction unavailable; raw evidence retained",
  );
  return result;
}
