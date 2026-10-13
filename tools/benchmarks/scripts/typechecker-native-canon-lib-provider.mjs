import assert from "node:assert/strict";
import path from "node:path";
import TOML from "@iarna/toml";
import { qualifyCargoConfigurations } from "./typechecker-native-canon-lib-config.mjs";

const compare = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
const registry = "registry+https://github.com/rust-lang/crates.io-index";

// Only where the procedural derive provider comes from is checked: Canon takes
// serde from the workspace, and serde_derive's lock closure resolves to single
// crates.io packages that no patch or replacement overrides. Versions and
// checksums are ordinary dependency updates, so they are not pinned here.
export function qualifyDeriveProvider(source, root, canon) {
  assert.deepEqual(
    canon.dependencies.serde,
    { workspace: true },
    "unreviewed Canon serde provider",
  );
  const inspectDependencies = (table) => {
    for (const [key, value] of Object.entries(table)) {
      if (["dependencies", "dev-dependencies", "build-dependencies"].includes(key)) {
        assert.ok(
          value.std === undefined && value.core === undefined,
          "shadowed standard derive provider",
        );
      }
      if (
        ["dependencies", "dev-dependencies", "build-dependencies"].includes(key) &&
        value.serde !== undefined
      ) {
        assert.deepEqual(value.serde, { workspace: true }, "shadowed Canon serde dependency");
      } else if (value && typeof value === "object" && !Array.isArray(value))
        inspectDependencies(value);
    }
  };
  inspectDependencies(canon);
  const workspaceSerde = root.workspace.dependencies.serde;
  assert.ok(
    workspaceSerde &&
      typeof workspaceSerde.version === "string" &&
      workspaceSerde.features?.includes("derive") &&
      workspaceSerde.path === undefined &&
      workspaceSerde.git === undefined &&
      workspaceSerde.package === undefined,
    "unreviewed workspace serde provider",
  );
  const lock = TOML.parse(source.read("Cargo.lock")).package;
  const closure = new Map();
  const queue = ["serde", "serde_core", "serde_derive"];
  while (queue.length) {
    const spec = queue.pop();
    const [name, version, extra] = spec.split(" ");
    assert.equal(extra, undefined, "opaque derive lock dependency");
    const candidates = lock.filter(
      (entry) => entry.name === name && (!version || entry.version === version),
    );
    assert.equal(candidates.length, 1, `ambiguous derive lock provider: ${spec}`);
    const entry = candidates[0];
    if (closure.has(name)) continue;
    assert.equal(entry.source, registry, `derive source: ${name}`);
    assert.match(entry.checksum ?? "", /^[0-9a-f]{64}$/u, `derive checksum: ${name}`);
    closure.set(name, entry);
    queue.push(...(entry.dependencies ?? []));
  }
  const inspectOverride = (alias, declaration) => {
    const name = declaration.package ?? alias.split(":")[0];
    assert.ok(!closure.has(name) && !closure.has(alias), `patched derive provider: ${alias}`);
    if (declaration.path) {
      assert.ok(!path.posix.isAbsolute(declaration.path), "external provider patch");
      const manifest = path.posix.normalize(`${declaration.path}/Cargo.toml`);
      const target = TOML.parse(source.read(manifest));
      assert.equal(target.package.name, name, "ambiguous patch package");
      assert.ok(!closure.has(target.package.name), "renamed derive provider patch");
    }
  };
  for (const table of Object.values(root.patch ?? {})) {
    for (const [alias, declaration] of Object.entries(table)) inspectOverride(alias, declaration);
  }
  for (const [alias, declaration] of Object.entries(root.replace ?? {}))
    inspectOverride(alias, declaration);
  qualifyCargoConfigurations(source);
  return [...closure.keys()].sort(compare);
}
