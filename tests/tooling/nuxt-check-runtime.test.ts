import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { test } from "node:test";

import { prepareNuxtCheckFixture } from "../_helpers/nuxt-check-prepare.ts";
import {
  inspectPrepareNode,
  inspectPrepareNpx,
  selectNuxtPrepareRuntime,
} from "../_helpers/nuxt-check-runtime.ts";

test("preparation binds an actual executable and scopes child Node without changing its caller", () => {
  const ambient = environmentFingerprint(process.env);
  const env = { PATH: process.env.PATH, VP_NODE_VERSION: "foreign-selection" };
  const before = { ...env };
  const selected = inspectPrepareNode(process.execPath, env);
  const actual = JSON.parse(
    execFileSync(
      "node",
      ["-p", "JSON.stringify({execPath:process.execPath,version:process.version})"],
      {
        env: selected.env,
        encoding: "utf8",
        timeout: 10_000,
      },
    ),
  );
  assert.equal(fs.realpathSync(actual.execPath), fs.realpathSync(process.execPath));
  assert.equal(actual.version, process.version);
  assert.equal(selected.env.VP_NODE_VERSION, process.versions.node);
  assert.equal(
    selected.observation.sha256,
    createHash("sha256").update(fs.readFileSync(actual.execPath)).digest("hex"),
  );
  assert.deepEqual(env, before);
  assert.equal(environmentFingerprint(process.env), ambient);
});

test("a filesystem distribution fixture binds adjacent npx and refuses missing or nonfile entries", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-npx-entry-"));
  const executable = path.join(root, process.platform === "win32" ? "node.exe" : "node");
  const npx = path.join(root, process.platform === "win32" ? "npx.cmd" : "npx");
  try {
    assert.throws(() => inspectPrepareNpx(executable), /ENOENT/);
    fs.mkdirSync(npx);
    assert.throws(() => inspectPrepareNpx(executable), /distribution must own npx/);
    fs.rmdirSync(npx);
    // This file tests path and byte identity only; it is never executed as npx.
    const content = "filesystem-only npx entry fixture\n";
    fs.writeFileSync(npx, content);
    const direct = inspectPrepareNpx(executable);
    assert.equal(path.dirname(direct.npx), path.dirname(executable));
    assert.equal(direct.npxEntry, fs.realpathSync(npx));
    assert.equal(direct.npxSha256, createHash("sha256").update(content).digest("hex"));
    fs.rmSync(npx);
    const target = path.join(root, "npx-entry");
    fs.writeFileSync(target, content);
    fs.symlinkSync(target, npx, "file");
    const linked = inspectPrepareNpx(executable);
    assert.equal(linked.npx, npx);
    assert.equal(linked.npxEntry, fs.realpathSync(target));
    assert.equal(linked.npxSha256, direct.npxSha256);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("a real runtime without adjacent npx authenticates its major before distribution lookup", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-bare-node-"));
  const executable = path.join(root, path.basename(fs.realpathSync(process.execPath)));
  try {
    fs.copyFileSync(fs.realpathSync(process.execPath), executable);
    const env = { VIZE_APP_CHECK_PREPARE_NODE22: executable };
    const actual = inspectPrepareNode(executable, env);
    assert.equal(actual.observation.executable, fs.realpathSync(executable));
    assert.equal(actual.observation.version, process.version);
    assert.equal(
      actual.observation.sha256,
      createHash("sha256").update(fs.readFileSync(process.execPath)).digest("hex"),
    );
    assert.throws(() => inspectPrepareNpx(executable), /ENOENT/);
    assert.throws(
      () => selectNuxtPrepareRuntime("22", env),
      process.versions.node.startsWith("22.") ? /ENOENT/ : /requires Node 22/,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function environmentFingerprint(env: NodeJS.ProcessEnv): string {
  return createHash("sha256")
    .update(JSON.stringify(Object.entries(env).sort(([a], [b]) => a.localeCompare(b))))
    .digest("hex");
}

test("other authored Node contracts retain the original environment and ignore the Node 22 override", () => {
  const env = { PATH: "original-path", VIZE_APP_CHECK_PREPARE_NODE22: "/missing/node" };
  for (const engine of [undefined, "24", ">=20", "^22.0.0"]) {
    const selected = selectNuxtPrepareRuntime(engine, env);
    assert.equal(selected.env, env);
    assert.deepEqual(selected.observation, { authoredEngine: engine, node22Selected: false });
  }
});

test("an authored Node 22 contract rejects unavailable, relative, and wrong runtime paths", () => {
  assert.throws(
    () => selectNuxtPrepareRuntime("22", { VIZE_APP_CHECK_PREPARE_NODE22: "node" }),
    /absolute executable path/,
  );
  assert.throws(
    () => selectNuxtPrepareRuntime("22", { VIZE_APP_CHECK_PREPARE_NODE22: "/missing/node" }),
    /ENOENT/,
  );
  if (!process.versions.node.startsWith("22.")) {
    assert.throws(
      () => selectNuxtPrepareRuntime("22", {}),
      /requires VIZE_APP_CHECK_PREPARE_NODE22/,
    );
    assert.throws(
      () => selectNuxtPrepareRuntime("22", { VIZE_APP_CHECK_PREPARE_NODE22: process.execPath }),
      /requires Node 22/,
    );
  } else {
    const selected = selectNuxtPrepareRuntime("22", {});
    assert.equal(selected.observation.node22Selected, true);
  }
});

test("a missing literal runtime fails before install and retains the original checked fixture", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-runtime-contract-"));
  const sourceDir = path.join(root, "tests/_fixtures/_git/provider");
  const captureDir = path.join(root, "evidence");
  const git = (cwd: string, args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" });
  const commit = (cwd: string) => {
    git(cwd, ["add", "."]);
    git(cwd, [
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.com",
      "commit",
      "-qm",
      "fixture",
    ]);
  };
  try {
    fs.mkdirSync(sourceDir, { recursive: true });
    const manifest = JSON.stringify({ engines: { node: "22" }, dependencies: { nuxt: "4.4.8" } });
    fs.writeFileSync(path.join(sourceDir, "package.json"), manifest);
    fs.writeFileSync(path.join(sourceDir, "pnpm-lock.yaml"), "original lock\n");
    git(sourceDir, ["init", "-q"]);
    commit(sourceDir);
    const revision = git(sourceDir, ["rev-parse", "HEAD"]).trim();
    git(root, ["init", "-q"]);
    git(root, [
      "update-index",
      "--add",
      "--cacheinfo",
      `160000,${revision},tests/_fixtures/_git/provider`,
    ]);
    commit(root);
    let commands = 0;
    assert.throws(
      () =>
        prepareNuxtCheckFixture({
          name: "provider",
          sourceDir,
          authorityRoot: root,
          captureDir,
          env: { VIZE_APP_CHECK_PREPARE_NODE22: "node" },
          run() {
            commands++;
            throw new Error("provider must not run without an actual Node22");
          },
        }),
      /absolute executable path/,
    );
    assert.equal(commands, 0);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(captureDir, "provider/preparation/receipt.json"), "utf8"),
    );
    assert.equal(receipt.failed, true);
    assert.equal(receipt.authoredPreserved, true);
    assert.equal(receipt.lockedInstallation, false);
    assert.deepEqual(receipt.commands, []);
    assert.equal(fs.readFileSync(path.join(sourceDir, "package.json"), "utf8"), manifest);
    assert.equal(
      fs.readFileSync(path.join(sourceDir, "pnpm-lock.yaml"), "utf8"),
      "original lock\n",
    );
    assert.equal(git(sourceDir, ["status", "--porcelain"]).trim(), "");
    assert.equal(fs.existsSync(path.join(sourceDir, ".vize-check-prepare-lock")), false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
