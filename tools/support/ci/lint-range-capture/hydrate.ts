import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  existsSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readlinkSync,
  readSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { directory, git, isMain, save, sha256, source } from "./common.ts";
import { projects } from "./projects.ts";
import { verifyTrackedSource } from "./preflight.ts";

// Only the two pinned upstream registry JSON patterns. Vue/style attributes stay inherited.
export const rawRegistryAttributes =
  "apps/v4/public/r/**/*.json -text -eol\napps/v4/registry/**/*.json -text -eol\n";

export function firstRawCheckout(cwd: string, revision: string) {
  assert.match(revision, /^[0-9a-f]{40}$/u);
  assert.equal(git(cwd, "ls-files", "-z").length, 0, "Require an unmaterialized fixture");
  assert.deepEqual(readdirSync(cwd).sort(), [".git"], "Forbid reused physical fixture inputs");
  const attributes = git(cwd, "rev-parse", "--git-path", "info/attributes").toString().trim();
  const path = resolve(cwd, attributes);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, rawRegistryAttributes, { flag: "wx" });
  // This is the FIRST checkout, not a repair/reset of a previously dirty fixture.
  git(cwd, "checkout", "--detach", revision);
  assert.equal(git(cwd, "rev-parse", "HEAD").toString().trim(), revision);
}

function fileIdentity(path: string, mode: string) {
  const stat = lstatSync(path);
  if (mode === "120000") {
    assert.ok(stat.isSymbolicLink());
    const bytes = readlinkSync(path, { encoding: "buffer" });
    return {
      bytes: bytes.length,
      gitBlob: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
      sha256: sha256(bytes),
    };
  }
  assert.ok(stat.isFile(), "Require a real tracked file: " + path);
  assert.equal(Boolean(stat.mode & 0o111), mode === "100755", "Tracked mode differs: " + path);
  const blob = createHash("sha1").update(`blob ${stat.size}\0`),
    hash = createHash("sha256");
  const fd = openSync(path, "r"),
    buffer = Buffer.alloc(64 * 1024);
  try {
    for (;;) {
      const count = readSync(fd, buffer);
      if (!count) break;
      const bytes = buffer.subarray(0, count);
      blob.update(bytes);
      hash.update(bytes);
    }
  } finally {
    closeSync(fd);
  }
  return { bytes: stat.size, gitBlob: blob.digest("hex"), sha256: hash.digest("hex") };
}

export function attestRawCheckout(cwd: string, revision: string, out: string) {
  assert.equal(git(cwd, "rev-parse", "HEAD").toString().trim(), revision);
  mkdirSync(out, { recursive: true });
  const stdout = join(out, "shadcn-tree.stdout.log"),
    stderr = join(out, "shadcn-tree.stderr.log");
  const stdoutFd = openSync(stdout, "wx"),
    stderrFd = openSync(stderr, "wx");
  try {
    execFileSync("git", ["-C", cwd, "ls-tree", "-r", "-z", revision], {
      stdio: ["ignore", stdoutFd, stderrFd],
    });
  } finally {
    closeSync(stdoutFd);
    closeSync(stderrFd);
  }
  const tree = readFileSync(stdout),
    text = tree.toString("utf8");
  assert.deepEqual(Buffer.from(text), tree, "Require exact UTF-8 tracked paths");
  const rows = text
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const match = /^(100644|100755|120000) blob ([0-9a-f]{40})\t(.+)$/u.exec(entry);
      assert.ok(match, "Unsupported tracked tree entry: " + entry);
      const [, mode, expectedBlob, path] = match;
      assert.ok(!isAbsolute(path) && !path.split("/").includes(".."));
      const actual = fileIdentity(join(cwd, path), mode);
      return { path, mode, expectedBlob, ...actual, byteExact: expectedBlob === actual.gitBlob };
    });
  const receipt = {
    revision,
    attributes: rawRegistryAttributes,
    tree: { path: "shadcn-tree.stdout.log", bytes: tree.length, sha256: sha256(tree) },
    files: rows,
    allOriginalBlobsExact: rows.length > 0 && rows.every((row) => row.byteExact),
    producerExecuted: false,
    acceptance: false,
  };
  writeFileSync(join(out, "shadcn-raw-checkout.json"), JSON.stringify(receipt, null, 2) + "\n", {
    flag: "wx",
  });
  assert.ok(receipt.allOriginalBlobsExact, "Physical fixture bytes differ from original Git blobs");
  return receipt;
}

export function hydrate(root: string, expected: string) {
  source(root, expected);
  verifyTrackedSource(root, expected);
  assert.equal(projects.length, 19);
  const shadcn = projects.find((project) => project.id === "shadcn-vue");
  assert.ok(shadcn);
  git(
    root,
    "submodule",
    "update",
    "--init",
    "--depth",
    "1",
    "--",
    ...projects.filter((project) => project !== shadcn).map((project) => project.fixturePath),
  );
  const cwd = join(root, shadcn.fixturePath);
  assert.ok(!existsSync(join(cwd, ".git")), "Forbid previously hydrated shadcn fixture");
  mkdirSync(cwd, { recursive: true });
  assert.equal(readdirSync(cwd).length, 0);
  git(root, "submodule", "init", "--", shadcn.fixturePath);
  git(cwd, "init", "--quiet");
  git(cwd, "remote", "add", "origin", shadcn.repository);
  git(cwd, "fetch", "--depth", "1", "--no-tags", "origin", shadcn.revision);
  firstRawCheckout(cwd, shadcn.revision);
  git(root, "submodule", "absorbgitdirs", "--", shadcn.fixturePath);
  const attestation = attestRawCheckout(cwd, shadcn.revision, directory(root));
  verifyTrackedSource(root, expected);
  save(root, "fixture-hydration.json", {
    source: expected,
    projects: projects.map((project) => ({ id: project.id, revision: project.revision })),
    originalShadcnBlobs: attestation.files.length,
    producerExecuted: false,
    acceptance: false,
  });
}
if (isMain(import.meta.url)) {
  const [root, expected] = process.argv.slice(2);
  assert.ok(root && expected);
  hydrate(resolve(root), expected);
}
