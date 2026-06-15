import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// `Rendu` and `AtelierOutput` are unstable, internal Atelier-stage plates
// (#1765). They give DOM/SSR/Vapor a shared render-semantic surface, but they
// are not a public contract — they must not leak through the Vitrine surface
// (NAPI/WASM bindings and public payload types), or downstream code would pin a
// shape Vize needs to keep evolving.
const FORBIDDEN = [/\bRendu[A-Za-z0-9_]*/, /\bAtelierOutput\b/];

// Lines that publicly expose a name: re-exports and public signatures/types.
const PUBLIC_PREFIXES = ["pub use ", "pub fn ", "pub struct ", "pub enum ", "pub type "];

function rustFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...rustFiles(full));
    } else if (entry.name.endsWith(".rs")) {
      out.push(full);
    }
  }
  return out;
}

test("vitrine does not expose internal Rendu/AtelierOutput plates", () => {
  const srcDir = path.join(root, "crates", "vize_vitrine", "src");
  const offenders: string[] = [];

  for (const file of rustFiles(srcDir)) {
    const lines = fs.readFileSync(file, "utf8").split("\n");
    lines.forEach((line, index) => {
      const trimmed = line.trim();
      if (!PUBLIC_PREFIXES.some((prefix) => trimmed.startsWith(prefix))) {
        return;
      }
      if (FORBIDDEN.some((pattern) => pattern.test(trimmed))) {
        offenders.push(`${path.relative(root, file)}:${index + 1}: ${trimmed}`);
      }
    });
  }

  assert.deepEqual(
    offenders,
    [],
    `internal Rendu/AtelierOutput plates leaked through the Vitrine surface:\n${offenders.join("\n")}`,
  );
});
