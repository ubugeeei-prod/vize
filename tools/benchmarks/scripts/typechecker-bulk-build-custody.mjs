import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const sourceIdentity = (cwd, expected) => {
  const paths = [
    "Cargo.toml",
    "Cargo.lock",
    "crates/vize_canon/Cargo.toml",
    "crates/vize_canon/src",
    "pnpm-lock.yaml",
    ".cargo",
  ];
  const git = (...args) => execFileSync("git", args, { cwd, maxBuffer: 64 * 1024 * 1024 });
  const rows = git("ls-tree", "-r", "-z", expected, "--", ...paths)
    .toString()
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const tab = entry.indexOf("\t");
      const [mode, kind, expectedGitBlob] = entry.slice(0, tab).split(" ");
      const path = entry.slice(tab + 1);
      assert(kind === "blob" && ["100644", "100755"].includes(mode));
      const bytes = readFileSync(join(cwd, path));
      const gitBlob = createHash("sha1")
        .update("blob " + bytes.length + "\0")
        .update(bytes)
        .digest("hex");
      return {
        path,
        expectedGitBlob,
        gitBlob,
        matches: gitBlob === expectedGitBlob,
        bytes: bytes.length,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      };
    });
  return {
    expected,
    head: git("rev-parse", "HEAD").toString().trim(),
    diffBase64: git("diff", "--binary", expected, "--", ...paths).toString("base64"),
    porcelainBase64: git(
      "status",
      "--porcelain=v1",
      "-z",
      "--untracked-files=all",
      "--",
      ...paths,
    ).toString("base64"),
    rows,
  };
};
const assertSourceIdentity = (packet) => {
  assert.equal(packet.head, packet.expected);
  assert.equal(
    packet.diffBase64,
    "",
    "physical Canon source or locked authority differs from immutable tree",
  );
  assert.equal(
    packet.porcelainBase64,
    "",
    "Canon source or authority has a visible membership change",
  );
  assert(
    packet.rows.length && packet.rows.every((row) => row.matches),
    "physical Canon source bytes differ from immutable Git blobs",
  );
};

// Shared dependency caches are safe only after rebuilding each owned Canon arm.
export function buildMatchedBinaries({
  root,
  parent,
  source,
  baseline,
  temporary,
  output,
  receipt,
  persist,
}) {
  const binaries = {};
  receipt.binaries = binaries;
  for (const [arm, cwd] of [
    ["original", parent],
    ["bulk", root],
  ]) {
    const env = { ...process.env, CARGO_TARGET_DIR: join(root, "target") };
    const productionSource = arm === "original" ? baseline : source;
    binaries[arm] = { productionSource, sourceBefore: sourceIdentity(cwd, productionSource) };
    persist();
    assertSourceIdentity(binaries[arm].sourceBefore);
    const clean = spawnSync("cargo", ["clean", "--locked", "--profile", "ci", "-p", "vize_canon"], {
      cwd,
      env,
      encoding: "utf8",
      maxBuffer: 128 * 1024 * 1024,
    });
    writeFileSync(join(output, `${arm}-clean.stdout`), clean.stdout ?? "");
    writeFileSync(join(output, `${arm}-clean.stderr`), clean.stderr ?? "");
    binaries[arm].clean = {
      status: clean.status,
      signal: clean.signal,
      error: clean.error?.message,
    };
    persist();
    assert.equal(clean.status, 0, `${arm} Canon artifact eviction failed`);
    const result = spawnSync(
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_canon",
        "--test",
        "tier_l_incremental",
        "--no-run",
        "--message-format=json",
      ],
      { cwd, env, encoding: "utf8", maxBuffer: 128 * 1024 * 1024 },
    );
    writeFileSync(join(output, `${arm}-build.jsonl`), result.stdout ?? "");
    writeFileSync(join(output, `${arm}-build.stderr`), result.stderr ?? "");
    binaries[arm].build = {
      status: result.status,
      signal: result.signal,
      error: result.error?.message,
    };
    persist();
    assert.equal(
      result.status,
      0,
      `${arm} source build failed: ${result.error ?? result.signal ?? result.status}`,
    );
    const artifacts = result.stdout
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line));
    const manifest = join(cwd, "crates/vize_canon/Cargo.toml");
    const owned = artifacts.filter(
      (item) => item.reason === "compiler-artifact" && item.manifest_path === manifest,
    );
    const library = owned.find(
      (item) => item.target.name === "vize_canon" && item.target.kind.includes("lib"),
    );
    const test = owned.find((item) => item.target.name === "tier_l_incremental" && item.executable);
    binaries[arm].compilerArtifacts = { library, test };
    binaries[arm].sourceAfter = sourceIdentity(cwd, productionSource);
    persist();
    assertSourceIdentity(binaries[arm].sourceAfter);
    assert(library && test, `${arm} owned Canon library/test artifacts absent`);
    assert.equal(library.target.src_path, join(cwd, "crates/vize_canon/src/lib.rs"));
    assert.equal(test.target.src_path, join(cwd, "crates/vize_canon/tests/tier_l_incremental.rs"));
    assert.equal(library.fresh, false, `${arm} reused a Canon library from another source tree`);
    assert.equal(test.fresh, false, `${arm} reused a test binary from another source tree`);
    assert.deepEqual(test.features, library.features);
    const pinned = join(temporary, `${arm}-tier-l-test`);
    copyFileSync(test.executable, pinned);
    binaries[arm].path = pinned;
    binaries[arm].sha256 = createHash("sha256").update(readFileSync(pinned)).digest("hex");
    persist();
  }
  assert.deepEqual(
    binaries.original.compilerArtifacts.library.features,
    binaries.bulk.compilerArtifacts.library.features,
  );
  assert.deepEqual(
    binaries.original.compilerArtifacts.library.profile,
    binaries.bulk.compilerArtifacts.library.profile,
  );
  assert.deepEqual(
    binaries.original.compilerArtifacts.test.profile,
    binaries.bulk.compilerArtifacts.test.profile,
  );
  assert.notEqual(
    binaries.original.sha256,
    binaries.bulk.sha256,
    "different production arms produced the same binary; refuse timing acceptance",
  );
  return binaries;
}
