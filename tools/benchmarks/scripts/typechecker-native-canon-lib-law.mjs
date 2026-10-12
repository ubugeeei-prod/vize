import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import TOML from "@iarna/toml";
import { canonSource } from "./typechecker-native-canon-lib-scope.mjs";
import { physicalSource } from "./typechecker-native-canon-lib-source.mjs";

const repository = fileURLToPath(new URL("../../../", import.meta.url));
export const actual = canonSource(process.env.NATIVE_PHASE_SOURCE_ROOT ?? repository);
export const texts = new Map(
  [...actual.paths]
    .filter(
      (file) =>
        file.endsWith(".rs") ||
        file.endsWith(".toml") ||
        file === "Cargo.lock" ||
        file === ".cargo/config",
    )
    .map((file) => [file, actual.read(file)]),
);
// Provider override manifests are actual authored inputs, not executed modules.
const rootManifest = TOML.parse(texts.get("Cargo.toml"));
for (const declarations of Object.values(rootManifest.patch ?? {})) {
  for (const target of Object.values(declarations)) {
    if (target.path) {
      const manifest = `${target.path}/Cargo.toml`;
      texts.set(manifest, actual.read(manifest));
    }
  }
}
export const canonManifest = "crates/vize_canon/Cargo.toml";
export const integration = "crates/vize_canon/tests/ambient_declarations.rs";
export const library = "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests.rs";
export const workflow = physicalSource(repository).read(
  ".github/workflows/typechecker-native-phases.yml",
);
export function source(changes = {}) {
  const files = new Map(texts);
  for (const [file, text] of Object.entries(changes)) {
    if (text === null) files.delete(file);
    else files.set(file, text);
  }
  return {
    paths: new Set(files.keys()),
    read(file) {
      assert.ok(files.has(file), `missing source: ${file}`);
      return files.get(file);
    },
  };
}
export function append(file, text) {
  return { [file]: `${texts.get(file) ?? ""}\n${text}\n` };
}
export function manifestChange(file, mutate) {
  const parsed = TOML.parse(texts.get(file));
  mutate(parsed);
  return { [file]: TOML.stringify(parsed) };
}
