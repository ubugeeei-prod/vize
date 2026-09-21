// P5-4a: salsa belongs to the resident tier only (charter #10 and #39).
//
// The one-shot CLI (`vize build`, `fmt`, `lint`) stays on the fused non-salsa
// pipeline. These checks keep it that way: no workspace crate outside the
// resident tier may name `salsa` (proven to fail on an injected edge), salsa is
// pinned exactly with its `rayon` default off, and the `vize` binary's build
// graph without default features contains neither salsa nor the resident crate.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse as parseToml } from "@iarna/toml";

import {
  type Metadata,
  dependency,
  metadata,
  readRepoFile,
  repoRoot,
  workspacePackage,
} from "./support/davinci-stage-dependencies.ts";

/** The crates allowed to depend on salsa: the resident tier. */
const residentTier = new Set(["vize_resident"]);

/** Workspace packages that name `salsa` in any dependency kind, sorted. */
function salsaDependents(graph: Metadata): string[] {
  return graph.packages
    .filter((pkg) => pkg.dependencies.some((dep) => dep.name === "salsa"))
    .map((pkg) => pkg.name)
    .sort();
}

/** The dependents that are not resident-tier crates: must be empty. */
function outsideResidentTier(graph: Metadata): string[] {
  return salsaDependents(graph).filter((name) => !residentTier.has(name));
}

test("only the resident tier depends on salsa", () => {
  assert.deepEqual(salsaDependents(metadata), ["vize_resident"]);
  assert.deepEqual(outsideResidentTier(metadata), []);
});

test("the salsa check fails on an injected edge in vize_atelier_sfc", () => {
  const injected = structuredClone(metadata);
  workspacePackage(injected, "vize_atelier_sfc").dependencies.push({
    name: "salsa",
    features: [],
    rename: null,
    kind: null,
    optional: false,
    req: "=0.28.4",
  });
  assert.deepEqual(outsideResidentTier(injected), ["vize_atelier_sfc"]);
});

test("salsa is pinned exactly with rayon off, and the resident crate is unpublished", () => {
  const manifest = parseToml(readRepoFile("Cargo.toml")) as {
    workspace: { dependencies: Record<string, Record<string, unknown>> };
  };
  const salsa = manifest.workspace.dependencies.salsa;
  assert.deepEqual(salsa, {
    version: "=0.28.4",
    "default-features": false,
    features: ["macros", "inventory"],
  });
  assert.equal(dependency(metadata, "vize_resident", "salsa", null).req, "=0.28.4");
  assert.deepEqual(workspacePackage(metadata, "vize_resident").publish, []);
});

test("the one-shot vize binary has no salsa in its build graph", () => {
  const result = spawnSync(
    "cargo",
    [
      "tree",
      "--locked",
      "-p",
      "vize",
      "--no-default-features",
      "--edges",
      "normal,build",
      "--prefix",
      "none",
      "--format",
      "{p}",
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr);
  const packages = new Set(
    result.stdout
      .split("\n")
      .map((line) => line.split(" ")[0])
      .filter((name) => name !== ""),
  );
  // Scope proof: the tree is the real graph (the binary and its compiler
  // stack), not an empty or failed listing.
  for (const name of ["vize", "vize_atelier_sfc", "vize_s1_to_s2"]) {
    assert.ok(packages.has(name), `cargo tree must list ${name}`);
  }
  assert.deepEqual(
    [...packages].filter((name) => name.startsWith("salsa") || residentTier.has(name)),
    [],
  );
});
