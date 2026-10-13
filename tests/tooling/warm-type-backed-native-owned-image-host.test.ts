import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { historicalCompositionBody } from "./support/native-sdk-historical-source.mjs";
import {
  assertNativeTerminalLocks,
  nativeTerminalManifest,
  qualifyNativeTerminalHost as qualifyOriginalHost,
} from "../performance/support/warm-type-backed-native-terminal-host.ts";
import {
  nativeOwnedImageBodyPairs,
  nativeOwnedImageManifest,
  nativeOwnedImageProduction,
  qualifyNativeTerminalHost,
} from "../performance/support/warm-type-backed-native-owned-image-host.ts";

const original = nativeTerminalManifest;
const successor = nativeOwnedImageManifest;
const harness = original.qualifiedHarnessBodies.map(
  ([file, before, after]) =>
    [
      file,
      before,
      successor.qualifiedHarnessBodies.find(([name]) => file === name)?.[2] ?? after,
    ] as const,
);
const entries = new Map([
  ...original.preservedBodies.map(([file, hash]) => [file, { before: hash, after: hash }] as const),
  ...[...nativeOwnedImageBodyPairs, ...successor.corpusBodies, ...harness].map(
    ([file, before, after]) => [file, { before, after }] as const,
  ),
]);
const digest = (side: "before" | "after", file: string) => entries.get(file)?.[side] ?? null;
const production = nativeOwnedImageProduction;
const locks = (side: "before" | "after") =>
  Object.fromEntries(
    ["Cargo.toml", "Cargo.lock", "package.json", "pnpm-lock.yaml", "rust-toolchain.toml"].map(
      (file) => [file, digest(side, file)!],
    ),
  );

test("whole owned-image successor admits121 production and one executed corpus body while preserving original authority", () => {
  assert.equal(qualifyOriginalHost(production, new Set(), digest), null);
  const result = qualifyNativeTerminalHost(production, new Set(), digest);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 121);
  assert.equal("wholeSourceBodyPairs" in result && result.wholeSourceBodyPairs, 122);
  assert.equal(result.preservedBodyPairs, 23);
  assert.equal(result.qualifiedHarnessBodyPairs, 2);
  assertNativeTerminalLocks(locks("before"), locks("after"), result);
});

test("retained original SDK successor source and full runtime bodies match their finite review", () => {
  const actual = (side: "before" | "after", file: string) =>
    side === "before"
      ? digest(side, file)
      : createHash("sha256").update(historicalCompositionBody("sdk", file)).digest("hex");
  assert.ok(qualifyNativeTerminalHost(production, new Set(), actual));
});

test("absent or foreign whole source corpus runtime and final guard bodies refuse on either side", () => {
  for (const [file] of entries)
    for (const side of ["before", "after"] as const)
      for (const changed of [null, "f".repeat(64)]) {
        if (changed === digest(side, file)) continue;
        assert.equal(
          qualifyNativeTerminalHost(production, new Set(), (candidate, name) =>
            candidate === side && name === file ? changed : digest(candidate, name),
          ),
          null,
          `${side}:${file}`,
        );
      }
});

test("unknown partial duplicated and mixed historical source membership does not enroll", () => {
  for (const file of production) {
    assert.equal(
      qualifyNativeTerminalHost(
        production.filter((name) => file !== name),
        new Set(),
        digest,
      ),
      null,
    );
    assert.equal(qualifyNativeTerminalHost([...production, file], new Set(), digest), null);
  }
  assert.equal(
    qualifyNativeTerminalHost(
      [...production, "crates/vize_canon/src/unknown.rs"],
      new Set(),
      digest,
    ),
    null,
  );
  const historical = new Map(
    original.files.map(([file, before, after]) => [file, { before, after }]),
  );
  assert.equal(
    qualifyNativeTerminalHost(
      production,
      new Set(),
      (side, file) => historical.get(file)?.[side] ?? digest(side, file),
    ),
    null,
  );
});

test("foreign provider and Cargo locks stay fatal despite a complete successor receipt", () => {
  const result = qualifyNativeTerminalHost(production, new Set(), digest);
  assert.ok(result);
  for (const side of ["before", "after"] as const)
    for (const file of Object.keys(locks(side))) {
      const before = locks("before");
      const after = locks("after");
      (side === "before" ? before : after)[file] = "f".repeat(64);
      assert.throws(() => assertNativeTerminalLocks(before, after, result));
    }
  assert.throws(() =>
    assertNativeTerminalLocks(
      locks("before"),
      { ...locks("after"), "unknown.lock": "f".repeat(64) },
      result,
    ),
  );
});
