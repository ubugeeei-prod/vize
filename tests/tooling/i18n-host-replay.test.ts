import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import {
  preflight,
  validateManifestEdges,
} from "../../tools/support/levels/i18n-host-preflight.ts";
import { edges } from "../../tools/support/levels/i18n-host-contract.ts";

const root = path.resolve(import.meta.dirname, "../..");
function state(overrides: Record<string, string> = {}) {
  const read = (file: string) => overrides[file] ?? readFileSync(path.join(root, file), "utf8");
  return () =>
    preflight(read, (file) => file === "crates/vize_carton/src/i18n/compiler_error_tests.rs");
}
const lockPath = "Cargo.lock";
const lock = readFileSync(path.join(root, lockPath), "utf8");
function withoutLockEdge(name: string, dependency: string) {
  const start = lock.indexOf(`name = "${name}"\n`);
  const end = lock.indexOf("\n[[package]]", start);
  const entry = lock.slice(start, end);
  return lock.replace(entry, entry.replace(` "${dependency}",\n`, ""));
}

test("complete host integration has a byte-identical preflight plan", () => {
  assert.equal(state()(), lock);
});

test("missing host or retained core lock ownership rejects before replay writes", () => {
  for (const [name, dependency] of [
    ["vize_carton", "once_cell"],
    ["vize_carton", "rustc-hash"],
    ["vize_relief", "vize_carton"],
    ["vize_vitrine", "vize_carton"],
    ["vize_l0", "once_cell"],
    ["vize_l0", "rustc-hash"],
  ]) {
    assert.throws(
      state({ [lockPath]: withoutLockEdge(name, dependency) }),
      /dependency|ownership/u,
    );
  }
});

test("malformed real host calls reject before replay writes", () => {
  for (const [file, before] of [
    [
      "crates/vize/src/commands/explain/page.rs",
      "code.localized_message_with(&catalog.messages())",
    ],
    [
      "crates/vize_relief/src/errors/diagnostic.rs",
      ".localized_help_with(&translator().for_locale(locale))",
    ],
    [
      "crates/vize_relief/src/errors/diagnostic.rs",
      "code.localized_help_with(&translator().for_locale(locale))",
    ],
  ]) {
    const source = readFileSync(path.join(root, file), "utf8");
    assert.ok(source.includes(before));
    assert.throws(
      state({ [file]: source.replace(before, before.replace("_with", "_unexpected")) }),
      /integration state|provider call/u,
    );
  }
});

test("mixed module ownership and missing host manifest edges reject before replay writes", () => {
  const core = "davinci/vize_l0/src/lib.rs";
  assert.throws(
    state({ [core]: readFileSync(path.join(root, core), "utf8") + "pub mod i18n;\n" }),
    /module ownership|integration state/u,
  );
  const file = "crates/vize_vitrine/Cargo.toml";
  const source = readFileSync(path.join(root, file), "utf8");
  assert.throws(
    state({
      [file]: source.replace("vize_carton.workspace = true", "vize_carton.workspace = false"),
    }),
    /manifest ownership/u,
  );
});

test("old manifests reject existing false, quoted or table dependency keys before insertion", () => {
  const original: Record<string, string> = {};
  for (const [name, additions] of edges) {
    const file = `crates/${name}/Cargo.toml`;
    let source = readFileSync(path.join(root, file), "utf8");
    for (const dependency of additions)
      source = source.replace(`${dependency}.workspace = true\n`, "");
    original[file] = source;
  }
  validateManifestEdges((file) => original[file], true);
  for (const [name, additions] of edges)
    for (const dependency of additions) {
      const file = `crates/${name}/Cargo.toml`;
      for (const declaration of [
        `${dependency}.workspace = false`,
        `"${dependency}".workspace = false`,
        `'${dependency}'.workspace = false`,
        `"${dependency}"."workspace" = false`,
        `${dependency} . workspace = false`,
        `[dependencies.${dependency}]\nworkspace = false`,
        `[target.'cfg(unix)'.dependencies]\n${dependency}.workspace = false`,
        `renamed = { package = "${dependency}", workspace = false }`,
      ]) {
        const changed = original[file].replace(
          "[dependencies]\n",
          "[dependencies]\n" + declaration + "\n",
        );
        assert.throws(
          () => validateManifestEdges((path) => (path === file ? changed : original[path]), true),
          /manifest ownership|malformed host manifest/u,
        );
      }
    }
});
