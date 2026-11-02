/** Correctness captures remain unqualified; the historical experiment stays strict. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";
import { scopeBindingCases } from "./component-scope-binding-cases.ts";
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const record = (value: unknown): Record<string, unknown> => {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
};
const save = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
export function scopeBindingMode(args: string[]) {
  const [root, flag] = args;
  assert(
    root &&
      (args.length === 1 ||
        (args.length === 2 && ["--experiment", "--correctness"].includes(flag))),
  );
  return { root, purpose: flag === "--correctness" ? "correctness" : "experiment" };
}
export const scopePacketEqual = (base: unknown, head: unknown) => isDeepStrictEqual(base, head);
export function scopeObservation(rows: readonly { identical: boolean; sameInput: boolean }[]) {
  assert.equal(rows.length, 96);
  return rows.every((row) => row.identical && row.sameInput) ? "NOT_APPLICABLE" : "INCOMPARABLE";
}
export function auditScopeCompatibility(root: string, provenance: Record<string, unknown>) {
  assert.equal(provenance.observerPurpose, "correctness");
  assert.equal(provenance.eventName, "pull_request");
  const base = provenance.productBase,
    head = provenance.productHead;
  assert(typeof base === "string" && /^[a-f0-9]{40}$/u.test(base));
  assert(typeof head === "string" && /^[a-f0-9]{40}$/u.test(head));
  const args = [
    "diff",
    "--no-ext-diff",
    "--no-renames",
    base,
    head,
    "--",
    ":(glob)crates/*/src/**",
  ];
  const result = spawnSync("git", args, {
    maxBuffer: 4 * 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const prefix = path.join(root, "correctness-source-delta");
  for (const stream of ["stdout", "stderr"] as const)
    if (result[stream] !== null)
      fs.writeFileSync(prefix + "." + stream + ".bin", result[stream]!, { flag: "wx" });
  save(prefix + ".process.json", {
    program: "git",
    args,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  assert(result.stdout !== null && result.stderr !== null);
  const sourceDelta = {
    file: "correctness-source-delta.stdout.bin",
    sha256: hash(result.stdout),
    qualification: false,
    scope: "actual source retained, no per-head digest admission",
  };
  const comparisons: Array<Record<string, unknown> & { identical: boolean; sameInput: boolean }> =
    [];
  for (const input of scopeBindingCases)
    for (const mode of ["runtime-off", "allocation"])
      for (const round of [1, 2, 3]) {
        const [before, after] = ["base", "head"].map((role) => {
          const row = record(
            JSON.parse(
              fs.readFileSync(
                path.join(
                  root,
                  `${role}-${input.shape}-${input.count}-${input.scriptBindings}-${mode}-${round}.json`,
                ),
                "utf8",
              ),
            ),
          );
          assert.equal(row.schema, "vize.component-scope-binding-profile.v1");
          assert.equal(row.shape, input.shape);
          assert.equal(row.count, input.count);
          assert.equal(row.scriptBindings, input.scriptBindings);
          assert.equal(row.mode, mode);
          assert.equal(
            row.sourceRevision,
            provenance[role === "base" ? "productBase" : "productHead"],
          );
          assert.deepEqual(row.cold, { generationCalls: 1, warmups: 0, persistentCache: false });
          assert.equal(typeof row.source, "string");
          record(row.packet);
          return row;
        });
        comparisons.push({
          ...input,
          mode,
          round,
          sourceSha256: hash(before.source as string),
          headSourceSha256: hash(after.source as string),
          beforePacketSha256: hash(JSON.stringify(before.packet)),
          afterPacketSha256: hash(JSON.stringify(after.packet)),
          identical: scopePacketEqual(before.packet, after.packet),
          sameInput: before.source === after.source && before.file === after.file,
        });
      }
  const observation = scopeObservation(comparisons);
  save(path.join(root, "packet-comparisons.json"), {
    provenance,
    sourceDelta,
    comparisons,
    observation,
    qualification: false,
  });
  return { observation, sourceDelta, comparisons, normal: [] };
}

export function scopeCorrectnessReport(
  provenance: unknown,
  binaries: unknown,
  observed: {
    comparisons: Array<Record<string, unknown> & { identical: boolean; sameInput: boolean }>;
    sourceDelta: unknown;
  },
) {
  return {
    schema: "vize.component-scope-binding-comparison.v1",
    status: scopeObservation(observed.comparisons),
    qualification: false,
    performanceQualification: false,
    provenance,
    binaries,
    sourceDelta: observed.sourceDelta,
    comparisons: observed.comparisons,
    normal: [],
    scope: {
      normal: "Not executed",
      performance:
        "Unqualified ordinary correctness observation; differences and unknowns retained",
      correctness: "Independent actual suites remain required",
    },
  };
}
