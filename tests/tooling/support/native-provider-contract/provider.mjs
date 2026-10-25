import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./wire.mjs";

export function fileTree(root) {
  const rows = [];
  const visit = (directory) => {
    for (const entry of fs
      .readdirSync(directory, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name, "en"))) {
      const file = path.join(directory, entry.name),
        relative = path.relative(root, file);
      if (entry.isSymbolicLink()) rows.push({ path: relative, symlink: fs.readlinkSync(file) });
      else if (entry.isDirectory()) {
        rows.push({ path: relative, directory: true });
        visit(file);
      } else {
        assert.ok(entry.isFile(), "ordinary owned provider/input files only");
        const bytes = fs.readFileSync(file);
        rows.push({ path: relative, bytes: bytes.length, sha256: sha256(bytes) });
      }
    }
  };
  visit(root);
  return rows;
}
export function loadProvider(root, authority, evidence) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const providerRoot = fs.realpathSync(root);
  const metaRoot = fs.realpathSync(path.join(providerRoot, "node_modules/typescript"));
  const platformRoot = fs.realpathSync(
    path.join(providerRoot, "node_modules/@typescript/typescript-linux-x64"),
  );
  const lock = JSON.parse(fs.readFileSync(path.join(providerRoot, "package-lock.json"), "utf8"));
  for (const [name, directory] of [
    ["typescript", metaRoot],
    ["@typescript/typescript-linux-x64", platformRoot],
  ]) {
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
    assert.equal(manifest.name, name);
    assert.equal(manifest.version, authority.sdkVersion);
    assert.equal(manifest.gitHead, authority.officialGitHead);
    const entry = lock.packages[`node_modules/${name}`];
    assert.equal(entry.version, authority.sdkVersion);
    assert.equal(entry.resolved, authority.npmPackages[name].resolved);
    assert.equal(entry.integrity, authority.npmPackages[name].integrity);
  }
  const executable = fs.realpathSync(path.join(platformRoot, "lib/tsc"));
  const initial = fileTree(providerRoot);
  const check = () => {
    assert.deepEqual(
      fileTree(providerRoot),
      initial,
      "every installed provider byte/link remains unchanged",
    );
    assert.equal(sha256(fs.readFileSync(executable)), authority.linuxX64ImageSha256);
    const version = spawnSync(executable, ["--version"], { encoding: null });
    assert.equal(version.error, undefined);
    assert.equal(version.status, 0);
    assert.equal(version.signal, null);
    assert.equal(version.stderr.length, 0);
    assert.equal(version.stdout.toString("utf8"), `Version ${authority.sdkVersion}\n`);
    return {
      physicalPath: executable,
      sha256: authority.linuxX64ImageSha256,
      version: {
        status: version.status,
        signal: version.signal,
        stdoutBase64: version.stdout.toString("base64"),
        stderrBase64: version.stderr.toString("base64"),
      },
      installedFileTree: initial,
    };
  };
  const receipt = check();
  fs.copyFileSync(executable, path.join(evidence, "sdk-image.bin"));
  assert.equal(
    sha256(fs.readFileSync(path.join(evidence, "sdk-image.bin"))),
    authority.linuxX64ImageSha256,
  );
  return { executable, metaRoot, receipt, check };
}
export function runningImage(pid, provider) {
  const proc = `/proc/${pid}/exe`;
  const physicalPath = fs.realpathSync(proc);
  assert.equal(
    physicalPath,
    provider.executable,
    "the child is the actual SDK, without a wrapper image",
  );
  const digest = sha256(fs.readFileSync(proc));
  assert.equal(digest, provider.receipt.sha256);
  return { pid, proc, physicalPath, sha256: digest };
}
export function isolatedEnvironment(directory) {
  const env = { ...process.env };
  for (const key of [
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
    "TSRS_PATH",
    "TSRS_EXECUTABLE",
    "CORSA_BIN",
  ])
    delete env[key];
  const home = path.join(directory, "empty-home"),
    tools = path.join(directory, "tool-path");
  fs.mkdirSync(home, { recursive: true });
  fs.mkdirSync(tools, { recursive: true });
  fs.symlinkSync(process.execPath, path.join(tools, "node"));
  return { ...env, HOME: home, PATH: tools, NO_COLOR: "1", FORCE_COLOR: "0" };
}
