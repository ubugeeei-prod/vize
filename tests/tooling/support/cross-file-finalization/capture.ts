import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const hash = (bytes: Uint8Array | string): string =>
  createHash("sha256").update(bytes).digest("hex");

export function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

export function save(output: string, name: string, value: unknown): void {
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
}

export function run(output: string, label: string, binary: string, args: string[], cwd: string) {
  const start = process.hrtime.bigint();
  const child = spawnSync(binary, args, { cwd, maxBuffer: 32 * 1024 * 1024 });
  const elapsedNs = Number(process.hrtime.bigint() - start);
  const stdout = child.stdout ?? Buffer.alloc(0);
  const stderr = child.stderr ?? Buffer.alloc(0);
  fs.writeFileSync(path.join(output, label + ".stdout"), stdout);
  fs.writeFileSync(path.join(output, label + ".stderr"), stderr);
  const packet = {
    binary,
    args,
    cwd,
    status: child.status,
    signal: child.signal,
    error: child.error?.message ?? null,
    elapsedNs,
    stdout: { bytes: stdout.length, sha256: hash(stdout) },
    stderr: { bytes: stderr.length, sha256: hash(stderr) },
  };
  save(output, label + ".packet.json", packet);
  assert.equal(child.error, undefined, label);
  assert.equal(child.signal, null, label);
  assert.equal(stderr.length, 0, label);
  assert.ok(child.status === 0 || child.status === 1, label);
  return { stdout, packet };
}

export function sourceProfile(output: string, label: string) {
  const profile = record(
    JSON.parse(fs.readFileSync(path.join(output, label + ".profile.json"), "utf8")),
  );
  assert.deepEqual(profile.truncation, { dropped_spans: 0, dropped_counters: 0 });
  assert.ok(Array.isArray(profile.spans) && Array.isArray(profile.counters));
  const spans = profile.spans
    .map(record)
    .filter((row) => String(row.key).startsWith("croquis_cf.dependencies."));
  const counters = profile.counters
    .map(record)
    .filter((row) => String(row.key).startsWith("croquis_cf.dependencies."));
  const bySpan = new Map(spans.map((row) => [row.key, row]));
  const byCounter = new Map(counters.map((row) => [row.key, row]));
  for (const name of [
    "finalize",
    "update",
    "nuxt_render",
    "resolve_import",
    "component_targets",
    "clear_component_edges",
  ]) {
    assert.ok(bySpan.has(`croquis_cf.dependencies.${name}`), `${label}: actual ${name} phase`);
  }
  for (const name of ["finalize.calls", "update.calls", "nuxt_render.calls"]) {
    const counter = byCounter.get(`croquis_cf.dependencies.${name}`);
    const span = bySpan.get(`croquis_cf.dependencies.${name.replace(/\.calls$/u, "")}`);
    assert.ok(counter && span);
    assert.equal(counter.total, span.count, `${label}: independent span/counter call observations`);
  }
  assert.equal(
    byCounter.get("croquis_cf.dependencies.resolve_import.attempts")?.total,
    bySpan.get("croquis_cf.dependencies.resolve_import")?.count,
  );
  return { spans, counters };
}
