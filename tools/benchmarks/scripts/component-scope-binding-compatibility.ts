/** One authenticated dead-alias transition can refuse performance comparison, never qualify it. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";
import { scopeBindingCases } from "./component-scope-binding-cases.ts";

type Row = Record<string, unknown>;
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const record = (value: unknown): Row => {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
const save = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });

// Full native packets are kept in the raw profile files. These digests only
// authorize an INCOMPARABLE refusal for the observed source transition.
export const intendedScopeChanges = [
  {
    shape: "builtin",
    sourceSha256: "3dec3310601d0b3cb4a72436b69b4ebaa80a5ea1e842c3519eed1f35ca03fdf8",
    beforePacketSha256: "5427ede0c252c9c7c52649dacf2f24aa44c860ced5de27eeafae379327a6e722",
    afterPacketSha256: "cc28d914a3a8f67a8c1ae8adc91be4cb693ca61df2e755592abe45668cf76ae2",
  },
  {
    shape: "global",
    sourceSha256: "8362e1017ed6b97e2ced20791e58d262f4be010b548d5b5e45ed5013fb6a5e11",
    beforePacketSha256: "15b5cd7474245dacff22d5f2435da147e135e69273b66694bde6998e8dec2929",
    afterPacketSha256: "24b7b6ca633d45191d387bcc4178868f5f238a690d63fa28aa3bbddd7daecfde",
  },
] as const;
export const intendedScopeSources = [
  {
    path: "crates/vize_canon/src/virtual_ts/scope/component_prop_checker.rs",
    beforeSha256: "2e4ea0d3fa187454472a17bc35606db0a47be5bb05f1afc93078b56ed69f5c30",
    afterSha256: "f8f12d9e7735cbe998852e708a4f6eb3ed0032be756be03e394816d9cd87265d",
  },
  {
    path: "crates/vize_canon/src/virtual_ts/scope/component_props.rs",
    beforeSha256: "6e112d0a7245eee9543655b06aaa2d8678883b9ffc9acedb5f30f11bbcd831d0",
    afterSha256: "9b8891cf4a3740d07c38815e1e3472029a447e38b6dc24feeb44451c36c2aea0",
  },
] as const;

type Pair = {
  shape: string;
  count: number;
  scriptBindings: number;
  sourceSha256: string;
  beforePacketSha256: string;
  afterPacketSha256: string;
  identical: boolean;
};
export function classifyScopePair(pair: Pair) {
  if (pair.identical) {
    return "equal" as const;
  }
  const known = intendedScopeChanges.find((row) => row.shape === pair.shape);
  return known &&
    pair.count === 32 &&
    pair.scriptBindings === 10000 &&
    pair.sourceSha256 === known.sourceSha256 &&
    pair.beforePacketSha256 === known.beforePacketSha256 &&
    pair.afterPacketSha256 === known.afterPacketSha256
    ? ("incomparable" as const)
    : ("unexpected" as const);
}

export function scanScopePackets(root: string, provenance: Row) {
  const comparisons = [];
  for (const input of scopeBindingCases)
    for (const mode of ["runtime-off", "allocation"])
      for (const round of [1, 2, 3]) {
        const rows = ["base", "head"].map((role) => {
          const file = `${role}-${input.shape}-${input.count}-${input.scriptBindings}-${mode}-${round}.json`;
          const row = record(JSON.parse(fs.readFileSync(path.join(root, file), "utf8")));
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
        const [before, after] = rows;
        assert(before && after);
        assert.equal(before.source, after.source);
        assert.equal(before.file, after.file);
        const pair = {
          ...input,
          sourceSha256: hash(before.source as string),
          beforePacketSha256: hash(JSON.stringify(before.packet)),
          afterPacketSha256: hash(JSON.stringify(after.packet)),
          identical: isDeepStrictEqual(before.packet, after.packet),
        };
        comparisons.push({ ...pair, mode, round, status: classifyScopePair(pair) });
      }
  assert.equal(comparisons.length, 96);
  return comparisons;
}

export function requireScopeSourceTransition(
  actual: readonly { path: string; beforeSha256: string; afterSha256: string }[],
) {
  assert.deepEqual(
    actual,
    intendedScopeSources,
    "unrecognized source transition is not an allowed refusal",
  );
}

function captureSourceTransition(root: string, provenance: Row) {
  const actual = intendedScopeSources.map((expected, index) => {
    const digests = ["base", "head"].map((role) => {
      const revision = provenance[role === "base" ? "productBase" : "productHead"];
      assert(typeof revision === "string" && /^[a-f0-9]{40}$/u.test(revision));
      const args = ["show", revision + ":" + expected.path];
      const result = spawnSync("git", args, {
        maxBuffer: 1024 * 1024,
        stdio: ["ignore", "pipe", "pipe"],
      });
      const prefix = path.join(root, `source-transition-${role}-${index}`);
      for (const stream of ["stdout", "stderr"] as const)
        if (result[stream] !== null)
          fs.writeFileSync(prefix + "." + stream + ".bin", result[stream]!, { flag: "wx" });
      save(prefix + ".process.json", {
        program: "git",
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        streams: {
          stdout: result.stdout === null ? null : hash(result.stdout),
          stderr: result.stderr === null ? null : hash(result.stderr),
        },
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0);
      assert(result.stdout !== null && result.stderr !== null);
      return hash(result.stdout);
    });
    return { path: expected.path, beforeSha256: digests[0]!, afterSha256: digests[1]! };
  });
  save(path.join(root, "source-transition.json"), { provenance, actual });
  requireScopeSourceTransition(actual);
  return actual;
}

/** Returns before any normal mapper command; an incomparable packet never becomes a pass. */
export function auditScopeCompatibility(root: string, provenance: Row) {
  const comparisons = scanScopePackets(root, provenance);
  save(path.join(root, "packet-comparisons.json"), {
    schema: "vize.component-scope-packets.v1",
    provenance,
    comparisons,
  });
  assert(
    comparisons.every((row) => row.status !== "unexpected"),
    "unexpected whole packet change",
  );
  const incomparable = comparisons.filter((row) => row.status === "incomparable");
  if (incomparable.length === 0) return null;
  assert.equal(incomparable.length, 12);
  for (const { shape } of intendedScopeChanges)
    assert.equal(incomparable.filter((row) => row.shape === shape).length, 6);
  const sourceTransition = captureSourceTransition(root, provenance);
  return { comparisons, sourceTransition, normal: [] };
}
