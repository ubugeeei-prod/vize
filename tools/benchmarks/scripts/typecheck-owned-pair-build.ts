/** Pre-build proof: neither source role can reuse a target or an installed native image. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { requireMeasurementRunner } from "./typecheck-owned-pair-run.ts";
import { sourceRevision } from "./typecheck-owned-pair-authority.ts";
import { writeJson } from "./typecheck-owned-pair-evidence.ts";
export function freshBuildBoundary(directory: string, source: string) {
  assert.equal(sourceRevision(directory), source);
  assert.equal(existsSync(join(directory, "target")), false, "pre-existing Cargo output");
  assert.equal(
    existsSync(join(directory, "npm/native/.artifacts")),
    false,
    "pre-existing N-API build output",
  );
  assert.deepEqual(
    readdirSync(join(directory, "npm/native")).filter((file) => file.endsWith(".node")),
    [],
    "pre-existing loader image",
  );
  return {
    directory: resolve(directory),
    source,
    targetAbsent: true,
    nativeOutputAbsent: true,
    loaderImagesAbsent: true,
    nativeRecipe: "vp run --workspace-root build:native",
    cliRecipe: "vp run --filter './npm/cli' build",
  };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  requireMeasurementRunner();
  const [base, baseSha, head, headSha, out] = process.argv.slice(2);
  assert.ok(base && baseSha && head && headSha && out);
  mkdirSync(out, { recursive: true });
  writeJson(join(out, "build-boundary.json"), {
    base: freshBuildBoundary(base, baseSha),
    head: freshBuildBoundary(head, headSha),
  });
}
