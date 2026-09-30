import { existsSync, readdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import path from "node:path";

export function moveFoundation(root, mode) {
  const before = path.join(root, "crates/vize_carton");
  const after = path.join(root, "davinci/vize_l0");
  if (mode === "--foundation-moves-only") {
    for (const entry of readdirSync(path.join(before, "src"))) {
      if (entry === "lib.rs") continue;
      renameSync(path.join(before, "src", entry), path.join(after, "src", entry));
    }
    return;
  }
  const read = (base, file) => readFileSync(path.join(base, file), "utf8");
  const write = (base, file, source) => writeFileSync(path.join(base, file), source);
  const oldLib = read(before, "src/lib.rs");
  if (oldLib.includes("pub use vize_l0::*;")) return;
  const levelLib = read(after, "src/lib.rs");
  write(
    after,
    "src/lib.rs",
    oldLib
      .replaceAll("vize_carton", "vize_l0")
      .replace(
        "//! Carton - The artist's toolbox for Vize.",
        "//! L0 — the shared foundation for Vize.\n//!\n//! **Experimental:** level APIs may change during the restructure.",
      ) +
      "\nextern crate alloc;\nextern crate self as vize_l0;\n" +
      levelLib.slice(levelLib.indexOf("pub mod diag;")),
  );
  write(
    before,
    "src/lib.rs",
    "//! Legacy compatibility facade for the shared foundation.\n//! Canonical implementations live in `vize_l0`.\npub use vize_l0::*;\n",
  );
  const manifest = read(before, "Cargo.toml");
  const dependencies = manifest.slice(
    manifest.indexOf("[dependencies]"),
    manifest.indexOf("[lints]"),
  );
  const levelManifest = read(after, "Cargo.toml");
  write(
    after,
    "Cargo.toml",
    levelManifest.slice(0, levelManifest.indexOf("[dependencies]")) +
      dependencies +
      "[lints]\nworkspace = true\n",
  );
  write(
    before,
    "Cargo.toml",
    manifest.slice(0, manifest.indexOf("[dependencies]")) +
      '[dependencies]\nvize_l0 = { workspace = true }\n\n[features]\ndefault = []\nextension = ["vize_l0/extension"]\nlint-glob = ["vize_l0/lint-glob"]\n\n' +
      manifest
        .slice(manifest.indexOf("[dev-dependencies]"))
        .replace("[dev-dependencies]\n", "[dev-dependencies]\nserde_json.workspace = true\n"),
  );
  const walk = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(file);
      else if (entry.name.endsWith(".rs")) {
        const source = readFileSync(file, "utf8");
        writeFileSync(file, source.replaceAll("vize_carton", "vize_l0"));
      }
      // Snapshot bytes stay unchanged; only their crate-name filename changes.
      else if (entry.name.startsWith("vize_carton__")) {
        const target = path.join(directory, entry.name.replace("vize_carton__", "vize_l0__"));
        if (!existsSync(target)) renameSync(file, target);
      }
    }
  };
  walk(path.join(after, "src"));
}
