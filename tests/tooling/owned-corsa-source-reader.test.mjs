import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { ownedCorsaReader } from "../../tools/support/compat/github/owned-corsa-source-reader.mjs";

void test("real Git source reader authenticates raw and physical bodies; replacements and mode/body/path drift refuse", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-owned-source-reader-"));
  const git = (args) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: root,
      stdio: ["ignore", "pipe", "pipe"],
    });
  try {
    git(["init", "--quiet"]);
    const file = path.join(root, "member.rs");
    fs.writeFileSync(file, "pub const VALUE: u8 = 1;\n");
    git(["add", "member.rs"]);
    const commit = () => {
      git([
        "-c",
        "user.name=Source Fixture",
        "-c",
        "user.email=source-fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "fixture",
      ]);
      return git(["rev-parse", "HEAD"]).toString().trim();
    };
    const before = commit();
    fs.writeFileSync(file, "pub const VALUE: u8 = 2;\n");
    git(["add", "member.rs"]);
    const head = commit();
    const reader = ownedCorsaReader(root, before, head);
    assert.equal(reader("before", "member.rs").body.toString(), "pub const VALUE: u8 = 1;\n");
    assert.equal(reader("after", "member.rs").body.toString(), "pub const VALUE: u8 = 2;\n");
    const value = reader("after", "member.rs");
    value.body.fill(0);
    value.mode = "100755";
    assert.equal(reader("after", "member.rs").body.toString(), "pub const VALUE: u8 = 2;\n");
    for (const bad of ["../member.rs", "/member.rs", "a//b", "./member.rs"])
      assert.throws(() => reader("after", bad));
    fs.writeFileSync(file, "foreign bytes\n");
    assert.throws(() => ownedCorsaReader(root, before, head)("after", "member.rs"));
    fs.writeFileSync(file, "pub const VALUE: u8 = 2;\n");
    fs.chmodSync(file, 0o755);
    assert.throws(() => ownedCorsaReader(root, before, head)("after", "member.rs"));
    fs.chmodSync(file, 0o644);
    const foreign = path.join(root, "foreign.rs");
    fs.writeFileSync(foreign, "foreign\n");
    fs.unlinkSync(file);
    fs.symlinkSync(foreign, file);
    assert.throws(() => ownedCorsaReader(root, before, head)("after", "member.rs"));
    fs.unlinkSync(file);
    fs.writeFileSync(file, "pub const VALUE: u8 = 2;\n");
    git(["replace", head, before]);
    assert.equal(git(["show", head + ":member.rs"]).toString(), "pub const VALUE: u8 = 2;\n");
    assert.equal(
      execFileSync("git", ["show", head + ":member.rs"], { cwd: root }).toString(),
      "pub const VALUE: u8 = 1;\n",
    );
    assert.throws(() => ownedCorsaReader(root, before, head), {
      message: /replacement refs refuse source publication/u,
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
