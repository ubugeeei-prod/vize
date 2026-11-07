import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  buildGitCliffArgs,
  resolveGitCliffAsset,
  verifyGitCliffArchive,
} from "../../../tools/support/compat/release/regenerate-changelog.ts";

test("changelog regeneration builds git-cliff args from supported flags", () => {
  assert.deepEqual(buildGitCliffArgs([]), ["--config", "cliff.toml", "--output", "CHANGELOG.md"]);
  assert.deepEqual(buildGitCliffArgs(["--unreleased", "--latest"]), [
    "--config",
    "cliff.toml",
    "--output",
    "CHANGELOG.md",
    "--unreleased",
    "--latest",
  ]);
  assert.deepEqual(buildGitCliffArgs(["--tag", "v1.2.3"]), [
    "--config",
    "cliff.toml",
    "--output",
    "CHANGELOG.md",
    "--tag",
    "v1.2.3",
  ]);
});

test("changelog regeneration validates tag and unknown arguments before running git-cliff", () => {
  assert.throws(() => buildGitCliffArgs(["--tag"]), /Missing value for --tag/);
  assert.throws(() => buildGitCliffArgs(["--tag", "--latest"]), /Missing value for --tag/);
  assert.throws(() => buildGitCliffArgs(["--bogus"]), /Unknown argument: --bogus/);
});

test("changelog fallback pins every supported git-cliff archive", () => {
  for (const [platform, arch, suffix, sha512] of [
    [
      "linux",
      "x64",
      "x86_64-unknown-linux-gnu.tar.gz",
      "b1e7a6b70fc794accdccd9717eff5a8bf8b507f0308da44d837d935be31b18b19db3758d1a9bb4975fbca7d01ddaae251229a753ae2cd2ecc24845cfc50d067f",
    ],
    [
      "linux",
      "arm64",
      "aarch64-unknown-linux-gnu.tar.gz",
      "6e8aafddd1e1d163290561cd595fa86971b034012b0602fd8f554f831158af56d3c8f9d82793b4e79d1300f6620091602d81b49110f96ca87068d5df6127cf57",
    ],
    [
      "darwin",
      "arm64",
      "aarch64-apple-darwin.tar.gz",
      "d507021de298ceba43fd5cc1594c9b868888b5f8a685bc6910d31dc0902eb7b09e25de06a6ef20fb9fc98a505127e695e647747bc97a3697d626d54d6992b394",
    ],
    [
      "darwin",
      "x64",
      "x86_64-apple-darwin.tar.gz",
      "acc7e28da2af6337fe67b05003d5468cba4fc3014b2d51e7cf9f7f01e8ee11d4bc10845ba282694320648e7627af67bfe62d00a7e7b6f9faec8b2c38fbbd00a3",
    ],
    [
      "win32",
      "x64",
      "x86_64-pc-windows-msvc.zip",
      "9da4c3bb4c407ded67e411544c4d4b0c2b748c20db5181b9da72aeb652efc8c939a0e07411886d4132d49581c02e8b7ecae1d127dcc4508159a2f70fea780e76",
    ],
  ] as const) {
    const asset = resolveGitCliffAsset(platform, arch);
    assert.equal(asset.name, `git-cliff-2.6.1-${suffix}`);
    assert.equal(asset.sha512, sha512);
  }
  assert.throws(() => resolveGitCliffAsset("freebsd", "x64"), /Unsupported platform/);
});

test("changelog fallback rejects changed git-cliff archive bytes", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-git-cliff-proof-"));
  const archive = path.join(directory, "git-cliff.tar.gz");
  try {
    const bytes = Buffer.from("pinned archive fixture");
    fs.writeFileSync(archive, bytes);
    const digest = createHash("sha512").update(bytes).digest("hex");
    assert.doesNotThrow(() => verifyGitCliffArchive(archive, digest));
    assert.throws(() => verifyGitCliffArchive(archive, "0".repeat(128)), /SHA-512 mismatch/);
    fs.truncateSync(archive, 16 * 1024 * 1024 + 1);
    assert.throws(() => verifyGitCliffArchive(archive, digest), /size limit/);
  } finally {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});
