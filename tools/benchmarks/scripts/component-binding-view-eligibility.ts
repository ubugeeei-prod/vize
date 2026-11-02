/** Ordinary correctness captures never qualify the historical gain experiment. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const save = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
const record = (value: unknown): Record<string, unknown> => {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
};
export function bindingViewMode(args: string[]) {
  const [root, flag] = args;
  assert(
    root &&
      (args.length === 1 ||
        (args.length === 2 && ["--experiment", "--correctness"].includes(flag))),
  );
  return { root, purpose: flag === "--correctness" ? "correctness" : "experiment" };
}
export const sameBindingPackets = (base: unknown, head: unknown) => isDeepStrictEqual(base, head);
/** Exact authored input from the existing fixture, not a generated-output oracle. */
export function bindingViewSource(shape: string, count: number): string {
  assert(
    ["camel", "missing", "dynamic", "empty"].includes(shape) && [32, 128, 512].includes(count),
  );
  let source = '<!-- 日本 🧭 -->\r\n<script setup lang="ts">\r\n';
  for (let index = 0; index < count; index++)
    source += `const choiceItem${index} = {id: ${index}, label: '日本 🧭'};\r\n`;
  source +=
    'const { label: localLabel, ...rest } = defineProps<{label: string; class?: string}>();\r\n</script>\r\n<template><section :class="rest.class">{{ localLabel }}';
  for (let index = 0; index < count; index++) {
    if (shape === "camel") source += `<choice-item-${index} />`;
    if (shape === "missing") source += `<missing-item-${index} />`;
    if (shape === "dynamic") source += `<component :is="choiceItem${index}" />`;
  }
  return source + "</section></template>\r\n";
}
export function bindingViewObservation(
  comparisons: readonly { identical: boolean; sameInput: boolean; knownInput: boolean }[],
) {
  assert.equal(comparisons.length, 72);
  return comparisons.every((row) => row.identical && row.sameInput && row.knownInput)
    ? "NOT_APPLICABLE"
    : "INCOMPARABLE";
}
function retainSourceDelta(root: string, provenance: Record<string, unknown>) {
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
  return {
    file: "correctness-source-delta.stdout.bin",
    sha256: hash(result.stdout),
    qualification: false,
    scope: "actual source retained; no per-head digest admission",
  };
}
export function auditBindingViewCorrectness(root: string, provenance: Record<string, unknown>) {
  assert.equal(provenance.observerPurpose, "correctness");
  assert.equal(provenance.eventName, "pull_request");
  const sourceDelta = retainSourceDelta(root, provenance),
    comparisons = [];
  for (const shape of ["camel", "missing", "dynamic", "empty"])
    for (const count of [32, 128, 512])
      for (const mode of ["runtime-off", "allocation"])
        for (const round of [1, 2, 3]) {
          const [base, head] = ["base", "head"].map((role) => {
            const row = record(
              JSON.parse(
                fs.readFileSync(
                  path.join(root, `${role}-${shape}-${count}-${mode}-${round}.json`),
                  "utf8",
                ),
              ),
            );
            assert.equal(row.schema, "vize.component-binding-profile.v1");
            assert.equal(row.shape, shape);
            assert.equal(row.count, count);
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
            shape,
            count,
            mode,
            round,
            sourceSha256: hash(base.source as string),
            headSourceSha256: hash(head.source as string),
            basePacketSha256: hash(JSON.stringify(base.packet)),
            headPacketSha256: hash(JSON.stringify(head.packet)),
            identical: sameBindingPackets(base.packet, head.packet),
            sameInput: base.source === head.source && base.file === head.file,
            knownInput:
              base.source === bindingViewSource(shape, count) &&
              head.source === bindingViewSource(shape, count),
          });
        }
  const observation = bindingViewObservation(comparisons);
  save(path.join(root, "correctness-packet-comparisons.json"), {
    provenance,
    sourceDelta,
    comparisons,
    observation,
    qualification: false,
  });
  return { observation, sourceDelta, comparisons, normal: [] };
}

export function bindingViewCorrectnessReport(
  provenance: unknown,
  binaries: unknown,
  observed: {
    comparisons: Array<
      Record<string, unknown> & { identical: boolean; sameInput: boolean; knownInput: boolean }
    >;
    sourceDelta: unknown;
  },
) {
  return {
    schema: "vize.component-binding-comparison.v1",
    status: bindingViewObservation(observed.comparisons),
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
