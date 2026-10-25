import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import {
  beginNativeBuildReceipt,
  finishNativeBuildReceipt,
} from "../../tools/support/github/native-build-receipt.ts";
import { pathWith, runToolCommand, writeRecordingCommand } from "./_helpers/tool-commands.ts";

function fixture(t: TestContext) {
  const root = fs.mkdtempSync(path.join(tmpdir(), "native-build-receipt-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const packageDir = path.join(root, "npm/native");
  fs.mkdirSync(packageDir, { recursive: true });
  fs.writeFileSync(path.join(packageDir, "package.json"), '{"version":"0.571.0"}');
  execFileSync("git", ["init", "--quiet", root]);
  execFileSync("git", [
    "-C",
    root,
    "-c",
    "user.name=Receipt test",
    "-c",
    "user.email=receipt@example.test",
    "-c",
    "commit.gpgsign=false",
    "commit",
    "--allow-empty",
    "--quiet",
    "-m",
    "fixture",
  ]);
  const source = execFileSync("git", ["-C", root, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  const output = path.join(root, ".artifacts/native/build-receipt.json");
  const env = {
    ...process.env,
    VIZE_RELEASE_VERSION: "0.571.0",
    VIZE_NATIVE_BUILD_RECEIPT: output,
    VIZE_NATIVE_BUILD_TOKEN: "a".repeat(32),
  };
  return { root, packageDir, output, source, env };
}

void test("the producer binds the image to its own freshly minted token and checkout", (t) => {
  const { root, packageDir, output, source, env } = fixture(t);
  const bin = path.join(root, "bin");
  const image = path.join(packageDir, "vize-vitrine.linux-x64-gnu.node");
  writeRecordingCommand(bin, "vp", path.join(root, "calls.log"), {
    script: `fs.writeFileSync(${JSON.stringify(image)}, 'image:' + process.env.VIZE_NATIVE_BUILD_TOKEN);`,
  });
  const result = runToolCommand(
    "tools/support/github/build-napi-package.ts",
    ["npm/native", "m.toml", "vize_vitrine", "x86_64-unknown-linux-gnu"],
    { cwd: root, env: { ...env, PATH: pathWith(bin) } },
  );
  assert.equal(result.status, 0, result.stdout + result.stderr);
  const receipt = JSON.parse(fs.readFileSync(output, "utf8"));
  const bytes = fs.readFileSync(image);
  assert.deepEqual(receipt, {
    schema: "vize.nativeBuildReceipt.v1",
    source,
    releaseVersion: "0.571.0",
    rustTarget: "x86_64-unknown-linux-gnu",
    nodeFile: path.basename(image),
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    nativeBuildToken: bytes.toString().slice("image:".length),
  });
  assert.match(receipt.nativeBuildToken, /^[0-9a-f]{32}$/);
  assert.notEqual(receipt.nativeBuildToken, env.VIZE_NATIVE_BUILD_TOKEN);
});

void test("stale images and failed builds cannot retain a publishable receipt", (t) => {
  const { root, packageDir, output, env } = fixture(t);
  const bin = path.join(root, "bin");
  const image = path.join(packageDir, "vize-vitrine.linux-x64-gnu.node");
  fs.writeFileSync(image, "an earlier build's image");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, "stale receipt");
  writeRecordingCommand(bin, "vp", path.join(root, "calls.log"));
  const result = runToolCommand(
    "tools/support/github/build-napi-package.ts",
    ["npm/native", "m.toml", "vize_vitrine", "x86_64-unknown-linux-gnu"],
    { cwd: root, env: { ...env, PATH: pathWith(bin) } },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /does not contain this invocation's build token/);
  assert.equal(fs.existsSync(output), false);
  fs.writeFileSync(output, "stale receipt");
  writeRecordingCommand(bin, "vp", path.join(root, "failed.log"), { exitCodes: [42] });
  const failed = runToolCommand(
    "tools/support/github/build-napi-package.ts",
    ["npm/native", "m.toml", "vize_vitrine", "x86_64-unknown-linux-gnu"],
    { cwd: root, env: { ...env, PATH: pathWith(bin) } },
  );
  assert.equal(failed.status, 42);
  assert.equal(fs.existsSync(output), false);
});

void test("receipt preflight rejects version drift and completion rejects checkout drift", (t) => {
  const { root, packageDir, output, env } = fixture(t);
  assert.throws(
    () =>
      beginNativeBuildReceipt(packageDir, "x86_64-unknown-linux-gnu", {
        ...env,
        VIZE_RELEASE_VERSION: "0.572.0",
      }),
    /version must match/,
  );
  assert.throws(
    () => beginNativeBuildReceipt(packageDir, "unexpected-target", env),
    /Unsupported receipt target/,
  );
  const receipt = beginNativeBuildReceipt(packageDir, "x86_64-unknown-linux-gnu", env);
  fs.writeFileSync(
    path.join(packageDir, "vize-vitrine.linux-x64-gnu.node"),
    env.VIZE_NATIVE_BUILD_TOKEN,
  );
  execFileSync("git", [
    "-C",
    root,
    "-c",
    "user.name=Receipt test",
    "-c",
    "user.email=receipt@example.test",
    "-c",
    "commit.gpgsign=false",
    "commit",
    "--allow-empty",
    "--quiet",
    "-m",
    "drift",
  ]);
  assert.throws(
    () =>
      finishNativeBuildReceipt(
        receipt,
        packageDir,
        "vize_vitrine",
        "x86_64-unknown-linux-gnu",
        env,
      ),
    /checkout changed/,
  );
  assert.equal(fs.existsSync(output), false);
});
