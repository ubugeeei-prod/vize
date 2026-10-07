import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { repository, sha256 } from "./n8n-replay-inputs.mjs";

// The real parent of this performance slice, never a projected main or a
// redistributed n8n branch. Compare its JS against the same current addon.
export const referenceSource = {
  head: "8ec57bb939a1d8206c4a3b73b7f4417a331cc44a",
  tree: "b86be2f56575ff51ac64b5f8e1f014c05422028f",
};

export function prepareReferenceSource(output) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  const run = (command, args, options = {}) => {
    const result = spawnSync(command, args, {
      cwd: repository,
      encoding: "utf8",
      timeout: 120_000,
      maxBuffer: 32 * 1024 * 1024,
      ...options,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    return result;
  };
  const exists = spawnSync("git", ["cat-file", "-e", referenceSource.head + "^{commit}"], {
    cwd: repository,
  });
  if (exists.status !== 0) {
    const fetched = run("git", ["fetch", "--depth=1", "origin", referenceSource.head]);
    fs.writeFileSync(
      path.join(output, "reference-fetch.json"),
      JSON.stringify(fetched, null, 2) + "\n",
    );
  }
  assert.equal(
    run("git", ["rev-parse", referenceSource.head + "^{tree}"]).stdout.trim(),
    referenceSource.tree,
  );
  const prefix = "npm/oxlint/";
  // Runtime dependencies are linked only outside the uploaded artifact tree.
  // Preserve built bundles explicitly, never recursively archive node_modules.
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-reference-package-"));
  process.once("exit", () => fs.rmSync(root, { recursive: true, force: true }));
  const entries = run("git", ["ls-tree", "-rz", referenceSource.head, "--", prefix])
    .stdout.split("\0")
    .filter(Boolean)
    .map((entry) => {
      const [metadata, filename] = entry.split("\t");
      const [mode, kind, blob] = metadata.split(" ");
      assert.equal(kind, "blob");
      assert.ok(["100644", "100755"].includes(mode));
      assert.ok(filename.startsWith(prefix) && !filename.split("/").includes(".."));
      const physical = path.join(root, filename.slice(prefix.length));
      fs.mkdirSync(path.dirname(physical), { recursive: true });
      const bytes = spawnSync("git", ["cat-file", "blob", blob], {
        cwd: repository,
        maxBuffer: 32 * 1024 * 1024,
      });
      assert.equal(bytes.error, undefined);
      assert.equal(bytes.status, 0);
      fs.writeFileSync(physical, bytes.stdout, {
        flag: "wx",
        mode: mode === "100755" ? 0o755 : 0o644,
      });
      assert.ok(fs.readFileSync(physical).equals(bytes.stdout));
      return { filename, mode, blob, sha256: sha256(bytes.stdout), bytes: bytes.stdout.length };
    });
  fs.symlinkSync(path.join(repository, "npm/oxlint/node_modules"), path.join(root, "node_modules"));
  const args = ["pack"];
  const built = spawnSync(path.join(repository, "node_modules/.bin/vp"), args, {
    cwd: root,
    encoding: "utf8",
    timeout: 120_000,
    maxBuffer: 32 * 1024 * 1024,
    env: process.env,
  });
  fs.writeFileSync(
    path.join(output, "reference-build.json"),
    JSON.stringify({ args, cwd: root, ...built }, null, 2) + "\n",
  );
  assert.equal(built.error, undefined);
  assert.equal(built.signal, null);
  assert.equal(built.status, 0, built.stderr);
  for (const entry of entries)
    assert.equal(
      sha256(fs.readFileSync(path.join(root, entry.filename.slice(prefix.length)))),
      entry.sha256,
    );
  const bundles = fs
    .globSync("dist/**/*", { cwd: root })
    .filter((file) => fs.lstatSync(path.join(root, file)).isFile())
    .map((file) => ({ file, sha256: sha256(fs.readFileSync(path.join(root, file))) }));
  assert.ok(bundles.some(({ file }) => file === "dist/index.mjs"));
  assert.ok(bundles.some(({ file }) => file === "dist/cli.mjs"));
  const bundleOutput = path.join(output, "reference-bundles");
  for (const { file } of bundles) {
    const destination = path.join(bundleOutput, file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(root, file), destination, fs.constants.COPYFILE_EXCL);
    assert.equal(
      sha256(fs.readFileSync(destination)),
      sha256(fs.readFileSync(path.join(root, file))),
    );
  }
  const receipt = {
    source: referenceSource,
    physicalSource: entries,
    build: { args, cwd: root },
    bundles,
    limits: [
      "before JS source only, executed on the authenticated current cohort addon",
      "no parent binary or current native acceptance borrowed from this reference build",
    ],
  };
  fs.writeFileSync(
    path.join(output, "reference-source.json"),
    JSON.stringify(receipt, null, 2) + "\n",
  );
  return {
    ...receipt,
    plugin: path.join(root, "dist/index.mjs"),
    wrapper: path.join(root, "dist/cli.mjs"),
  };
}
