/** Lossless admission for the opt-in owned-instance pair. Raw files precede every gate. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { checkPacket } from "./pr-performance-observation-check.ts";
import { parseHostRun, splitHostOutput } from "./typecheck-corpus-pair-parse.ts";

export type RawRun = {
  stdout: Buffer;
  stderr: Buffer;
  status: number | null;
  signal: string | null;
  error: string | null;
};
export const sha256 = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
export function writeJson(file: string, value: unknown) {
  writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}
export function saveRawRun(directory: string, raw: RawRun, metadata: object) {
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(directory, "stdout.raw"), raw.stdout);
  writeFileSync(join(directory, "stderr.raw"), raw.stderr);
  writeJson(join(directory, "run.json"), {
    ...metadata,
    status: raw.status,
    signal: raw.signal,
    error: raw.error,
    stdout: { bytes: raw.stdout.length, sha256: sha256(raw.stdout) },
    stderr: { bytes: raw.stderr.length, sha256: sha256(raw.stderr) },
  });
}

function canonicalRoot(text: string, project: string): string {
  assert.ok(project.startsWith("/") && project !== "/");
  const escaped = project.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return text.replace(new RegExp(`(^|[\\s"'(=:])${escaped}(?=/|$)`, "gmu"), "$1<project>");
}

export function diagnosticEvidence(raw: RawRun, project: string, format: "json" | "text") {
  assert.equal(raw.error, null, "spawn failure");
  assert.equal(raw.signal, null, "terminated checker");
  assert.equal(raw.stderr.length, 0, "complete stderr must be empty");
  const stdout = new TextDecoder("utf-8", { fatal: true }).decode(raw.stdout);
  if (format === "json") {
    const packet = checkPacket(stdout);
    assert.equal(raw.status, packet.errorCount > 0 ? 1 : 0, "JSON exit/diagnostic disagreement");
    // Admission may sort a view; identity uses the entire original packet, including unknown fields.
    const full = JSON.parse(stdout);
    const canonical = (value: unknown): unknown => {
      if (typeof value === "string") return canonicalRoot(value, project);
      if (Array.isArray(value)) return value.map(canonical);
      if (value && typeof value === "object")
        return Object.fromEntries(
          Object.entries(value).map(([key, child]) => [key, canonical(child)]),
        );
      return value;
    };
    return { status: raw.status, packet: JSON.stringify(canonical(full)), format };
  }
  parseHostRun({ status: raw.status, stdout, stderr: "" }, project);
  splitHostOutput(stdout); // Admit the complete statistics trailer, not an arbitrary truncation.
  const start = /^Files:\s+\d+\r?$/mu.exec(stdout);
  assert.ok(start, "missing host statistics");
  return {
    status: raw.status,
    packet: canonicalRoot(stdout.slice(0, start.index), project),
    format,
  };
}
/** Fingerprint the complete canonical diagnostic packet and exit status. */
export const diagnosticFingerprint = (evidence: ReturnType<typeof diagnosticEvidence>) =>
  sha256(JSON.stringify(evidence));
export function assertSameEvidence(
  base: ReturnType<typeof diagnosticEvidence>,
  head: ReturnType<typeof diagnosticEvidence>,
) {
  assert.deepEqual(head, base, "complete diagnostic packet changed");
}

export function batchCommand(build: string, project: string, backend: string, width: 1 | 8) {
  assert.ok(width === 1 || width === 8, "explicit width must be 1 or 8");
  return [
    process.execPath,
    join(build, "npm/cli/bin/vize"),
    "check",
    "--quiet",
    "--format",
    "json",
    "--checkers",
    String(width),
    "--tsconfig",
    join(project, "tsconfig.json"),
    "--corsa-path",
    backend,
  ];
}
export async function freshCache<T>(
  run: (env: { XDG_CACHE_HOME: string }) => Promise<T>,
): Promise<T> {
  const cache = mkdtempSync(join(tmpdir(), "vize-owned-pair-cache-"));
  try {
    assert.deepEqual(readdirSync(cache), []);
    return await run({ XDG_CACHE_HOME: cache });
  } finally {
    rmSync(cache, { recursive: true, force: true });
  }
}

/** Exclude only installed providers; caches and generated project inputs stay visible. */
export function snapshotInputs(
  directory: string,
  excludeModules = true,
): Array<{ file: string; sha256: string }> {
  const rows: Array<{ file: string; sha256: string }> = [];
  const walk = (prefix: string) => {
    for (const entry of readdirSync(join(directory, prefix), { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name),
    )) {
      if (excludeModules && !prefix && entry.name === "node_modules") continue;
      const file = prefix ? `${prefix}/${entry.name}` : entry.name;
      assert.ok(!entry.isSymbolicLink(), `project input cannot hide behind a symlink: ${file}`);
      if (entry.isDirectory()) walk(file);
      else {
        assert.ok(entry.isFile());
        rows.push({ file, sha256: sha256(readFileSync(join(directory, file))) });
      }
    }
  };
  walk("");
  return rows;
}
export function assertSameInputs(
  base: ReturnType<typeof snapshotInputs>,
  head: ReturnType<typeof snapshotInputs>,
) {
  assert.deepEqual(head, base, "authored/generated input closure changed");
}
export type NativeAuthority = { source: string; image: string; sha256: string; token: string };
export function assertNativeReceipt(expected: NativeAuthority, observed: unknown) {
  assert.match(expected.source, /^[a-f0-9]{40}$/u);
  assert.match(expected.sha256, /^[a-f0-9]{64}$/u);
  assert.match(expected.token, /^[a-f0-9]{32}$/u);
  assert.ok(observed && typeof observed === "object");
  const row = observed as Record<string, unknown>;
  assert.equal(row.kind, "native", "Wasm or unknown native provider");
  for (const key of ["source", "image", "sha256", "token"] as const)
    assert.equal(row[key], expected[key], `native ${key} custody mismatch`);
}
