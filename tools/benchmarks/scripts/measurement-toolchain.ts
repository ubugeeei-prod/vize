import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseTomlLite } from "../../support/compat/davinci/toml-lite.ts";

// Measurements use the contributor pin; a toolchain bump re-baselines them in the same PR.
export const MEASUREMENT_RUST = "1.99.1";

export function measurementToolchain(root: string): string {
  const manifest = fs.readFileSync(path.join(root, "rust-toolchain.toml"), "utf8");
  const { toolchain: { channel: toolchain } } = parseTomlLite(manifest) as {
    toolchain: { channel: string };
  };
  assert.equal(
    toolchain,
    MEASUREMENT_RUST,
    `instruction source toolchain must be ${MEASUREMENT_RUST}`,
  );
  return toolchain;
}
