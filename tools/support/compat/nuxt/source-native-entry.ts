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
  assert.ok(
    installedEntry.startsWith(path.join(fs.realpathSync(fixture), "node_modules") + path.sep),
    "the fixture native entrypoint must remain in its own node_modules",
  );
  const sourceEntry = fs.realpathSync(path.join(source, "index.js"));
  assert.notEqual(installedEntry, sourceEntry, "the fixture must own its published entrypoint");
  const installed = path.dirname(installedEntry);
  const loaders = ["native-binding.js", "native-binding-identity.js", "native-targets.js"];
  // A loader the source changed since the fixture's pinned release is paired
  // from source as well, so the source binary never runs behind a mix of
  // published and source loaders. Unchanged loaders keep their published bytes.
  const pairedLoaders = loaders.flatMap((name) => {
    const published = fs.readFileSync(path.join(installed, name));
    const sourceLoader = fs.readFileSync(path.join(source, name));
    return published.equals(sourceLoader) ? [] : [{ name, published, source: sourceLoader }];
  });
  const loaderHashes = Object.fromEntries(
    loaders.map((name) => [name, hash(fs.readFileSync(path.join(source, name)))]),
  );
  const original = fs.readFileSync(installedEntry);
  const current = fs.readFileSync(sourceEntry);
  fs.writeFileSync(path.join(artifacts, "published-native-index.js"), original, { flag: "wx" });
  fs.writeFileSync(path.join(artifacts, "source-native-index.js"), current, { flag: "wx" });
  for (const loader of pairedLoaders) {
    const published = path.join(artifacts, `published-${loader.name}`);
    fs.writeFileSync(published, loader.published, { flag: "wx" });
    fs.writeFileSync(path.join(artifacts, `source-${loader.name}`), loader.source, { flag: "wx" });
  }
  const restoreLoaders = () => {
    for (const loader of pairedLoaders)
      fs.writeFileSync(path.join(installed, loader.name), loader.published);
  };
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
        pairedLoaders: pairedLoaders.map((loader) => loader.name),
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  try {
    fs.writeFileSync(installedEntry, current);
    for (const loader of pairedLoaders)
      fs.writeFileSync(path.join(installed, loader.name), loader.source);
    verify();
  } catch (error) {
    fs.writeFileSync(installedEntry, original);
    restoreLoaders();
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
        restoreLoaders();
        assert.deepEqual(fs.readFileSync(installedEntry), original);
        restored = true;
      }
    },
  };
}
