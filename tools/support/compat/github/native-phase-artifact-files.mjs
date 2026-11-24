import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { join, relative, resolve, sep } from "node:path";

export const rawPaths = JSON.parse(
  readFileSync(new URL("./native-phase-artifact-paths.json", import.meta.url), "utf8"),
);
export const manifestName = ".native-phase-artifact-manifest.json";
export const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const revision = (stat) =>
  [stat.dev, stat.ino, stat.size, stat.mode, stat.mtimeNs, stat.ctimeNs].map(String).join(":");

export function fileManifest(root, patterns = rawPaths) {
  root = resolve(root);
  const files = new Map();
  const hashes = new Map();
  let uniqueBytes = 0;
  const inspect = patterns === null ? lstatSync : statSync;
  function visit(path, ancestors = new Set()) {
    let stat;
    try {
      stat = inspect(path, { bigint: true });
    } catch (error) {
      // Pinned upload-artifact follows links and omits broken links/disappeared paths.
      if (error.code === "ENOENT") return;
      throw error;
    }
    assert(!stat.isSymbolicLink(), `symbolic link in reconstructed native evidence: ${path}`);
    if (stat.isDirectory()) {
      const real = realpathSync(path);
      // Match @actions/glob's traversal-chain cycle exclusion; retain all aliases.
      if (ancestors.has(real)) return;
      const parents = new Set([...ancestors, real]);
      for (const name of readdirSync(path).sort()) visit(join(path, name), parents);
      return;
    }
    assert(stat.isFile(), `unsupported native evidence file: ${path}`);
    const name = relative(root, path).split(sep).join("/");
    assert(name && !name.split("/").includes(".."), "native evidence escaped its root");
    if (files.has(name)) return;
    const key = revision(stat);
    let sha256 = hashes.get(key);
    if (!sha256) {
      const bytes = readFileSync(path);
      assert.equal(
        revision(inspect(path, { bigint: true })),
        key,
        `changed native evidence: ${name}`,
      );
      sha256 = digest(bytes);
      hashes.set(key, sha256);
      uniqueBytes += bytes.length;
    }
    files.set(name, {
      path: name,
      bytes: Number(stat.size),
      mode: Number(stat.mode & 0o777n),
      sha256,
    });
  }
  if (patterns === null) {
    for (const name of readdirSync(root).sort()) {
      if (name === manifestName) {
        assert(
          lstatSync(join(root, name)).isFile(),
          "reconstructed manifest must be a regular file",
        );
      } else visit(join(root, name));
    }
  } else {
    for (const pattern of patterns) {
      if (pattern.endsWith("*")) {
        const prefix = pattern.slice(0, -1);
        for (const name of readdirSync(root).sort()) {
          if (name.startsWith(prefix)) visit(join(root, name));
        }
      } else {
        visit(join(root, pattern));
      }
    }
  }
  return {
    files: [...files.values()].sort((left, right) => left.path.localeCompare(right.path, "en")),
    uniqueBytes,
  };
}

export function verifyReconstruction(root, expected) {
  const actual = fileManifest(root, null);
  assert.deepEqual(actual.files, expected.files, "native evidence reconstruction differs");
  return actual;
}
