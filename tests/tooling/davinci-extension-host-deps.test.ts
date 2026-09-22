import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// P6-3 / charter #39: wasmtime is reachable only through
// `vize_extension_host`'s `extension-host` feature. With default features no
// workspace crate — the `vize` CLI included — pulls it in; with the feature
// on, `vize_extension_host` is the only workspace crate that does.

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type ResolveNode = { id: string; deps: { pkg: string; dep_kinds: { kind: string | null }[] }[] };
type Metadata = {
  packages: { id: string; name: string }[];
  workspace_members: string[];
  resolve: { nodes: ResolveNode[] };
};

const isWasmtime = (id: string) => /#wasmtime@\d/u.test(id);

function metadata(features: string[]): Metadata {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--format-version", "1", ...features], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 256 * 1024 * 1024,
    }),
  ) as Metadata;
}

/** Workspace crates whose shipped (non-dev) dependency closure reaches wasmtime. */
function reachingWasmtime(meta: Metadata): string[] {
  const nodes = new Map(meta.resolve.nodes.map((node) => [node.id, node]));
  const names = new Map(meta.packages.map((pkg) => [pkg.id, pkg.name]));
  return meta.workspace_members
    .filter((member) => {
      const seen = new Set<string>();
      const queue = [member];
      while (queue.length > 0) {
        for (const edge of nodes.get(queue.pop()!)?.deps ?? []) {
          if (!edge.dep_kinds.some((kind) => kind.kind !== "dev") || seen.has(edge.pkg)) continue;
          if (isWasmtime(edge.pkg)) return true;
          seen.add(edge.pkg);
          queue.push(edge.pkg);
        }
      }
      return false;
    })
    .map((member) => names.get(member) ?? member)
    .toSorted();
}

function violations(meta: Metadata, allowed: string[]): string[] {
  return reachingWasmtime(meta)
    .filter((name) => !allowed.includes(name))
    .map((name) => `${name} reaches wasmtime outside the extension-host feature`);
}

const defaults = metadata([]);

test("no workspace crate reaches wasmtime with default features", () => {
  assert.equal(
    defaults.resolve.nodes.some((node) => isWasmtime(node.id)),
    false,
  );
  assert.deepEqual(violations(defaults, []), []);
  assert.ok(defaults.workspace_members.some((id) => /\/crates\/vize#/u.test(id)));
});

test("with extension-host on, only vize_extension_host reaches wasmtime", () => {
  const featured = metadata(["--features", "vize_extension_host/extension-host"]);
  assert.deepEqual(reachingWasmtime(featured), ["vize_extension_host"]);
});

test("the check fails on an injected wasmtime edge", () => {
  const cli = defaults.workspace_members.find((id) => /\/crates\/vize#/u.test(id));
  assert.ok(cli);
  const wasmtime = "registry+https://github.com/rust-lang/crates.io-index#wasmtime@48.0.2";
  const injected: Metadata = {
    ...defaults,
    resolve: {
      nodes: [
        ...defaults.resolve.nodes.map((node) =>
          node.id === cli
            ? { ...node, deps: [...node.deps, { pkg: wasmtime, dep_kinds: [{ kind: null }] }] }
            : node,
        ),
        { id: wasmtime, deps: [] },
      ],
    },
  };
  assert.deepEqual(violations(injected, []), [
    "vize reaches wasmtime outside the extension-host feature",
  ]);
});
