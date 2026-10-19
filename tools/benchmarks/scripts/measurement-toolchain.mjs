import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseTomlLite } from "../../support/compat/davinci/toml-lite.ts";

// Before this manifest existed, the repository pin was also the measurement pin.
// Keep those historical sources reproducible; an explicit malformed pin must fail.
export function measurementToolchain(root) {
  const explicit = path.join(root, "rust-measurement-toolchain.toml");
  const file = fs.existsSync(explicit) ? explicit : path.join(root, "rust-toolchain.toml");
  const toolchain = parseTomlLite(fs.readFileSync(file, "utf8")).toolchain.channel;
  assert.equal(toolchain, "1.99.0", "instruction source toolchain must be 1.99.0");
  return toolchain;
}
