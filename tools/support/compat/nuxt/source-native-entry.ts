import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

// Pair the source binary with its static CommonJS export declarations. Node
// derives an ESM namespace from these declarations before loading the binary.
export function stageSourceNativeEntrypoint(root: string, fixture: string, artifacts: string) {
  const source = path.join(root, "npm/native");
  const requirePlugin = createRequire(
    path.join(fixture, "node_modules/@vizejs/vite-plugin/package.json"),
  );
  const installedEntry = fs.realpathSync(requirePlugin.resolve("@vizejs/native"));
  const sourceEntry = fs.realpathSync(path.join(source, "index.js"));
  assert.notEqual(installedEntry, sourceEntry, "the fixture must own its published entrypoint");
  const installed = path.dirname(installedEntry);
  const loaders = ["native-binding.js", "native-binding-identity.js", "native-targets.js"];
  const loaderHashes = Object.fromEntries(
    loaders.map((name) => {
      const original = fs.readFileSync(path.join(installed, name));
      assert.deepEqual(
        original,
        fs.readFileSync(path.join(source, name)),
        `source and fixture loader must match before pairing: ${name}`,
      );
      return [name, hash(original)];
    }),
  );
  const original = fs.readFileSync(installedEntry);
  const current = fs.readFileSync(sourceEntry);
  fs.writeFileSync(path.join(artifacts, "published-native-index.js"), original, { flag: "wx" });
  fs.writeFileSync(path.join(artifacts, "source-native-index.js"), current, { flag: "wx" });
  const verify = () => {
    assert.deepEqual(fs.readFileSync(installedEntry), current, "source entrypoint changed");
    assert.deepEqual(fs.readFileSync(sourceEntry), current, "source entrypoint snapshot changed");
    for (const [name, expected] of Object.entries(loaderHashes))
      assert.equal(hash(fs.readFileSync(path.join(installed, name))), expected);
  };
  fs.writeFileSync(
    path.join(artifacts, "source-native-entry.json"),
    JSON.stringify(
      {
        installedEntry,
        sourceEntry,
        sourceSha256: hash(current),
        publishedSha256: hash(original),
        loaderHashes,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  try {
    fs.writeFileSync(installedEntry, current);
    verify();
  } catch (error) {
    fs.writeFileSync(installedEntry, original);
    throw error;
  }
  let restored = false;
  return {
    installedEntry,
    sourceEntry,
    sourceSha256: hash(current),
    publishedSha256: hash(original),
    loaderHashes,
    verify,
    restore() {
      if (restored) return;
      try {
        verify();
      } finally {
        fs.writeFileSync(installedEntry, original);
        assert.deepEqual(fs.readFileSync(installedEntry), original);
        restored = true;
      }
    },
  };
}
