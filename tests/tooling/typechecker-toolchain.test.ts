import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import {
  typecheckerSourceIdentity,
  validateTypecheckerReceipt,
} from "../differential/typechecker-shards.ts";

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

function receipt(identity: ReturnType<typeof typecheckerSourceIdentity>) {
  return {
    schemaVersion: 3,
    sha: identity.sha,
    tree: identity.tree,
    cargoProfile: "ci",
    nextestVersion: "0.9.146",
    rustcVersion: `rustc ${identity.toolchain} synthetic-unit-test`,
    requireTsgo: "1",
    disableTsgo: null,
    nuxtIterations: "100",
    archiveSha256: "0".repeat(64),
  };
}

test("both reviewed source pins retain strict compiler identity without runtime credit", (t) => {
  for (const channel of ["1.98.0", "1.99.0"]) {
    const identity = typecheckerSourceIdentity(source(t, channel));
    const original = receipt(identity);
    assert.equal(identity.toolchain, channel);
    assert.deepEqual(
      validateTypecheckerReceipt(Buffer.from(JSON.stringify(original)), identity),
      original,
    );
  }
});

test("current source rejects old, future, malformed and missing actual compiler stamps", (t) => {
  const identity = typecheckerSourceIdentity(source(t, "1.99.0"));
  for (const compiler of [
    "rustc 1.98.0 synthetic-unit-test",
    "rustc 1.100.0 synthetic-unit-test",
    "rustc 1.99.0-nightly synthetic-unit-test",
    "rustc 1x99x0 synthetic-unit-test",
    "rustc 1.99.01 synthetic-unit-test",
    undefined,
  ]) {
    const changed = { ...receipt(identity), rustcVersion: compiler };
    assert.throws(() => validateTypecheckerReceipt(Buffer.from(JSON.stringify(changed)), identity));
  }
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
  const historical = { ...receipt(expected), rustcVersion: "rustc 1.98.0 synthetic-unit-test" };
  assert.throws(() =>
    validateTypecheckerReceipt(Buffer.from(JSON.stringify(historical)), expected),
  );
});
