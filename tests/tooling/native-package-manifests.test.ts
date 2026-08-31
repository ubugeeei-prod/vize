import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("fresco-native publishes bundled binaries directly from the root package", () => {
  const frescoNativePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/fresco-native/package.json"), "utf-8"),
  ) as {
    files?: string[];
    scripts?: Record<string, string>;
  };
  const vizeNativePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/native/package.json"), "utf-8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.deepEqual(frescoNativePackage.files, ["index.js", "index.d.ts", "*.node"]);
  assert.equal(frescoNativePackage.scripts?.prepublishOnly, undefined);
  assert.equal(
    frescoNativePackage.scripts?.["build:ci"],
    "napi build --platform --profile ci --manifest-path ../../crates/vize_fresco/Cargo.toml -p vize_fresco --features napi --output-dir .",
  );
  assert.equal(
    vizeNativePackage.scripts?.["build:ci"],
    "napi build --platform --profile ci --manifest-path ../../crates/vize_vitrine/Cargo.toml -p vize_vitrine --features napi,legacy --output-dir . && node ./scripts/sync-entrypoint.mjs",
  );

  const frescoNativeLoader = fs.readFileSync(
    path.join(root, "npm/fresco-native/index.js"),
    "utf-8",
  );
  assert.match(frescoNativeLoader, /spawnSync\("ldd", \["--version"\]/);
  assert.doesNotMatch(frescoNativeLoader, /execSync\("which ldd"\)/);
  assert.doesNotMatch(frescoNativeLoader, /readFileSync\(lddPath/);
});
