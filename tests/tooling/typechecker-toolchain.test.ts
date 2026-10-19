import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { typecheckerSourceIdentity } from "../differential/typechecker-shards.ts";

function source(t: TestContext, channel?: string) {
  const directory = fs.mkdtempSync(path.join(tmpdir(), "vize-typechecker-toolchain-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: directory, encoding: "utf8", stdio: "pipe" }).trim();
  git("init", "--quiet");
  git("config", "user.name", "Fixture");
  git("config", "user.email", "fixture@example.invalid");
  git("config", "commit.gpgsign", "false");
  fs.writeFileSync(
    path.join(directory, "rust-toolchain.toml"),
    channel === undefined ? "[toolchain]\n" : `[toolchain]\nchannel = "${channel}"\n`,
  );
  git("add", "rust-toolchain.toml");
  git("commit", "--quiet", "-m", "test: retain source compiler stamp");
  return directory;
}

test("reviewed source pins resolve their committed compiler identity", (t) => {
  for (const channel of ["1.98.0", "1.99.0", "1.99.1"])
    assert.equal(typecheckerSourceIdentity(source(t, channel)).toolchain, channel);
});

test("unsupported or absent committed source stamps cannot qualify observations", (t) => {
  for (const channel of [undefined, "1.100.0", "stable", "1.99.0-nightly"])
    assert.throws(() => typecheckerSourceIdentity(source(t, channel)));
});

test("a working-tree pin edit cannot change the compiler authority at the stamped source", (t) => {
  const directory = source(t, "1.99.0");
  const expected = typecheckerSourceIdentity(directory);
  fs.writeFileSync(
    path.join(directory, "rust-toolchain.toml"),
    '[toolchain]\nchannel = "1.98.0"\n',
  );
  assert.deepEqual(typecheckerSourceIdentity(directory), expected);
});
