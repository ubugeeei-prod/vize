import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseTomlLite } from "../../support/compat/davinci/toml-lite.ts";

// Measurements use the contributor pin; a toolchain bump re-baselines them in the same PR.
export const MEASUREMENT_RUST = "1.99.1";

export function measurementToolchain(root: string): string {
  const text = fs.readFileSync(path.join(root, "rust-toolchain.toml"), "utf8");
  const manifest = parseTomlLite(text) as { toolchain: { channel: string } };
  const toolchain = manifest.toolchain.channel;
  assert.equal(
    toolchain,
    MEASUREMENT_RUST,
    `instruction source toolchain must be ${MEASUREMENT_RUST}`,
  );
  return toolchain;
}
