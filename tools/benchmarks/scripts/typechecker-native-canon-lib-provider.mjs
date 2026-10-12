import assert from "node:assert/strict";
import path from "node:path";
import TOML from "@iarna/toml";
import { qualifyCargoConfigurations } from "./typechecker-native-canon-lib-config.mjs";

const compare = (left, right) => (left < right ? -1 : left > right ? 1 : 0);

// Only the procedural derive provider is pinned, not changing product bodies.
// These reviewed crates.io identities form serde_derive's complete lock closure.
const reviewed = {
  "proc-macro2": [
    "1.0.106",
    "8fd00f0bb2e90d81d1044c2b32617f68fcb9fa3bb7640c23e9c748e53fb30934",
    ["unicode-ident"],
  ],
  quote: [
    "1.0.45",
    "41f2619966050689382d2b44f664f4bc593e129785a36d6ee376ddf37259b924",
    ["proc-macro2"],
  ],
  serde: [
    "1.0.228",
    "9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e",
    ["serde_core", "serde_derive"],
  ],
  serde_core: [
    "1.0.228",
    "41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad",
    ["serde_derive"],
  ],
  serde_derive: [
    "1.0.228",
    "d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79",
    ["proc-macro2", "quote", "syn 2.0.117"],
  ],
  syn: [
    "2.0.117",
    "e665b8803e7b1d2a727f4023456bbbbe74da67099c585258af0ad9c5013b9b99",
    ["proc-macro2", "quote", "unicode-ident"],
  ],
  "unicode-ident": [
    "1.0.24",
    "e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75",
    [],
  ],
};

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
  assert.deepEqual(
    root.workspace.dependencies.serde,
    { version: "=1.0.228", features: ["derive"] },
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
    assert.ok(Object.hasOwn(reviewed, name), `unreviewed derive dependency: ${name}`);
    const [reviewedVersion, checksum, dependencies] = reviewed[name];
    assert.equal(entry.version, reviewedVersion, `derive version: ${name}`);
    assert.equal(
      entry.source,
      "registry+https://github.com/rust-lang/crates.io-index",
      `derive source: ${name}`,
    );
    assert.equal(entry.checksum, checksum, `derive checksum: ${name}`);
    assert.deepEqual(
      [...(entry.dependencies ?? [])].sort(compare),
      [...dependencies].sort(compare),
      `derive dependencies: ${name}`,
    );
    closure.set(name, entry);
    queue.push(...(entry.dependencies ?? []));
  }
  assert.deepEqual(
    [...closure.keys()].sort(compare),
    Object.keys(reviewed).sort(compare),
    "incomplete derive closure",
  );
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
