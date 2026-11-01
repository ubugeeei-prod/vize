import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const SHIPPING_RECIPE = "cargo build --release -p vize";
export const ORIGINAL_BENCHMARK = "tests/performance/lsp-dual-server-benchmark.ts";
export const INVOKED_BENCHMARK = "tests/performance/.lsp-dual-server-shipping-benchmark.ts";
export const ORIGINAL_BENCHMARK_SHA256 = "7e6c64006e1f79704221d194cd5dd70fdb2f2c167899bfc44d0d4b8789cdd205";
const digest = (source: string): string => createHash("sha256").update(source).digest("hex");

/** Two custody-only token changes; all scheduling, wire and oracle bytes stay exact. */
export function shippingBenchmark(original: string): string {
  assert.equal(digest(original), ORIGINAL_BENCHMARK_SHA256, "immutable original benchmark drift");
  const edits = [["  BUILD_RECIPE,\n", "  SHIPPING_BUILD_RECIPE,\n"],
    ["      ? BUILD_RECIPE\n", "      ? SHIPPING_BUILD_RECIPE\n"]] as const;
  let adapted = original;
  for (const [before, after] of edits) {
    assert.equal(adapted.split(before).length, 2, "one exact original custody token");
    assert(!adapted.includes(after), "adapter token must not already exist");
    adapted = adapted.replace(before, after);
  }
  let restored = adapted;
  for (const [before, after] of [...edits].reverse()) restored = restored.replace(after, before);
  assert.equal(restored, original, "entire original body must be recoverable byte-exactly");
  return adapted;
}

export function adapterIdentity(original: string, adapted: string) {
  assert.equal(adapted, shippingBenchmark(original), "untracked benchmark adapter drift");
  return { originalPath: ORIGINAL_BENCHMARK, invokedPath: INVOKED_BENCHMARK,
    originalSha256: digest(original), adapterSha256: digest(adapted), changedCustodyTokens: 2,
    scope: "owned temporary receipt adapter; original sampling/wire/oracles are unchanged" };
}

function object(value: unknown): Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}

export function shippingBinary(value: unknown, source: string, imageSha256: string): void {
  const binary = object(value), receipt = object(binary["receipt"]);
  assert.equal(binary["relativeBinaryPath"], "target/ci/vize", "only the owned release staging path");
  assert.equal(binary["binarySha256"], imageSha256);
  assert.equal(receipt["recipe"], SHIPPING_RECIPE, "never label the release image as CI");
  assert.equal(receipt["sourceRevision"], source);
  assert.equal(receipt["binaryPath"], "target/ci/vize");
  assert.equal(receipt["binarySha256"], imageSha256);
}

export function shippingCargoArtifact(value: unknown): void {
  const artifact = object(value), target = object(artifact["target"]), profile = object(artifact["profile"]);
  assert.equal(target["name"], "vize"); assert.deepEqual(target["kind"], ["bin"]);
  assert.equal(profile["test"], false); assert.equal(profile["opt_level"], "3");
  assert.equal(profile["debug_assertions"], false);
  assert(Array.isArray(artifact["features"]));
  assert.deepEqual([...artifact["features"]].sort(), ["default", "glyph", "maestro"], "exact default shipping features");
}
