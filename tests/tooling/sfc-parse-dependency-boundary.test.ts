import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Metadata = {
  packages: Array<{ id: string; name: string }>;
  resolve: {
    nodes: Array<{
      id: string;
      deps: Array<{ pkg: string; dep_kinds: Array<{ kind: string | null }> }>;
    }>;
  };
};

test("the parse-only SFC facade excludes compiler dependencies transitively", () => {
  const result = spawnSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);

  const metadata = JSON.parse(result.stdout) as Metadata;
  const parser = metadata.packages.find(({ name }) => name === "vize_croquis");
  assert.ok(parser, "workspace must contain the parse-only SFC facade");

  const names = new Map(metadata.packages.map((pkg) => [pkg.id, pkg.name]));
  const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const reachable = new Set<string>();
  const pending = [parser.id];
  while (pending.length > 0) {
    const id = pending.pop() as string;
    if (reachable.has(id)) continue;
    reachable.add(id);
    for (const dependency of nodes.get(id)?.deps ?? []) {
      if (dependency.dep_kinds.some(({ kind }) => kind !== "dev")) pending.push(dependency.pkg);
    }
  }

  const dependencyNames = new Set([...reachable].map((id) => names.get(id)));
  for (const forbidden of [
    "lightningcss",
    "oxc_codegen",
    "oxc_transformer",
    "parcel_selectors",
    "vize_atelier_dom",
    "vize_atelier_ssr",
    "vize_atelier_vapor",
  ]) {
    assert.equal(
      dependencyNames.has(forbidden),
      false,
      `parse-only dependency graph must not include ${forbidden}`,
    );
  }
});
