/** Seal clean source images and the actual SDK10 declaration closure. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { identify, save } from "./canon-instance-directives-io.ts";
import { isRecord } from "./canon-instance-directives-model.ts";
import { captureDeclarationGraph } from "../../support/release/computed_event_custody.ts";
import { sdkFiles } from "../../support/release/constructor_original27_public/authority.ts";

export const git = (root: string, ...args: string[]) =>
  execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
export function sdkClosure(root: string) {
  const packageRoot = path.join(root, "npm/cli"),
    graph = captureDeclarationGraph(packageRoot);
  assert.deepEqual(
    graph.files.map((f) => path.relative(packageRoot, f.path)).sort(),
    [...sdkFiles, "virtual-ts-slots.d.ts"].sort(),
  );
  assert.equal(graph.edges.length, 13);
  return { package: identify(path.join(packageRoot, "package.json")), graph };
}
export function sameSdk(beforeRoot: string, afterRoot: string) {
  const before = sdkClosure(beforeRoot),
    after = sdkClosure(afterRoot);
  const relative = (value: typeof before, root: string) => ({
    package: { ...value.package, path: path.relative(root, value.package.path) },
    graph: {
      ...value.graph,
      files: value.graph.files.map((f) => ({ ...f, path: path.relative(root, f.path) })),
      edges: value.graph.edges.map((e) => ({
        ...e,
        from: path.relative(root, e.from),
        target: path.relative(root, e.target),
      })),
    },
  });
  assert.deepEqual(
    relative(after, afterRoot),
    relative(before, beforeRoot),
    "SDK10/source package closure must be identical between roles",
  );
  return { before, after };
}
export function sealBuild(root: string, source: string, cargo: string, image: string, out: string) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  assert.equal(git(root, "rev-parse", "HEAD"), source);
  assert.equal(git(root, "status", "--porcelain", "--untracked-files=all"), "");
  const rows = fs
    .readFileSync(cargo, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line) as unknown);
  const artifacts = rows.filter(
    (row) =>
      isRecord(row) &&
      row.reason === "compiler-artifact" &&
      isRecord(row.target) &&
      row.target.name === "vize" &&
      typeof row.executable === "string",
  );
  assert.equal(artifacts.length, 1);
  const artifact = artifacts[0];
  assert(isRecord(artifact));
  assert.equal(artifact.fresh, false);
  assert(typeof artifact.executable === "string");
  assert(Array.isArray(artifact.features));
  for (const feature of ["default", "glyph", "maestro"])
    assert(artifact.features.includes(feature));
  assert(!artifact.features.includes("profiling") && !artifact.features.includes("cache-custody"));
  assert(!fs.existsSync(image));
  fs.copyFileSync(artifact.executable, image);
  assert.equal(identify(image).sha256, identify(artifact.executable).sha256);
  save(out, {
    source,
    tree: git(root, "rev-parse", "HEAD^{tree}"),
    cleanBeforeObserverCopy: true,
    productionOverlay: false,
    observerImportedYet: false,
    artifact,
    image: identify(image),
    cargo: identify(cargo),
    sdk: sdkClosure(root),
    timingComparison: false,
  });
}
if (process.argv[2] === "seal") {
  assert.equal(process.argv.length, 8);
  sealBuild(...(process.argv.slice(3) as [string, string, string, string, string]));
}
