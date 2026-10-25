import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { discoverWorkspaceServers } from "../../editors/vscode/src/workspace-server.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function controlledRealpaths(realpaths) {
  const folders = ["C:\\one", "C:\\two"];
  const files = new Map();
  const binaries = new Map();
  for (const [index, folder] of folders.entries()) {
    files.set(path.win32.join(folder, "package.json"), "{}");
    files.set(
      path.win32.join(folder, "node_modules/vize/package.json"),
      JSON.stringify({
        name: "vize",
        version: "9.8.7",
        bin: "bin/vize.cjs",
      }),
    );
    binaries.set(path.win32.join(folder, "node_modules/vize/bin/vize.cjs"), realpaths[index]);
  }
  const source = stripTypeScriptTypes(
    fs.readFileSync(path.join(root, "editors/vscode/src/workspace-server.ts"), "utf8"),
  )
    .replace(/^import[\s\S]*?;\n/gm, "")
    .replace(/^export /gm, "");
  const discover = new vm.Script(`${source}\ndiscoverWorkspaceServers;`).runInNewContext({
    path: path.win32,
    fs: {
      existsSync: (file) => files.has(file),
      readFileSync: (file) => {
        assert.ok(files.has(file));
        return files.get(file);
      },
      statSync: (file) => {
        assert.ok(binaries.has(file));
        return { isFile: () => true };
      },
      realpathSync: (file) => {
        assert.ok(binaries.has(file));
        return binaries.get(file);
      },
    },
  });
  return discover(folders, true, "win32");
}

await test("the same Windows realpath with a drive-letter alias selects one engine", () => {
  const result = controlledRealpaths([
    "C:\\actual\\Engine\\bin\\vize.cjs",
    "c:\\actual\\Engine\\bin\\vize.cjs",
  ]);
  assert.equal(result.reason, undefined);
  assert.equal(result.candidates.length, 1);
});

await test("distinct case-sensitive Windows realpaths never share an engine at the same version", () => {
  const result = controlledRealpaths([
    "C:\\actual\\Engine\\bin\\vize.cjs",
    "C:\\actual\\engine\\bin\\vize.cjs",
  ]);
  assert.equal(result.candidates.length, 0);
  assert.match(result.reason, /different Vize installations/);
  assert.ok(result.reason.includes("C:\\actual\\Engine\\bin\\vize.cjs"));
  assert.ok(result.reason.includes("C:\\actual\\engine\\bin\\vize.cjs"));
});

function recordCapability(t, classification, reason, extra = {}) {
  const evidence = {
    sourceSha: process.env.VIZE_TEST_SOURCE_SHA,
    platform: process.platform,
    classification,
    reason,
    ...extra,
  };
  const destination = path.join(root, "editors/vscode/.vscode-test/workspace-server");
  fs.mkdirSync(destination, { recursive: true });
  fs.writeFileSync(
    path.join(destination, "windows-identity-capability.json"),
    JSON.stringify(evidence, null, 2) + "\n",
  );
  t.diagnostic(JSON.stringify(evidence));
}

await test("actual case-sensitive Windows directories keep distinct same-version builds separate", (t) => {
  if (process.platform !== "win32") {
    recordCapability(
      t,
      "not-applicable",
      "The actual NTFS control requires a Windows runner; controlled realpath refusal runs on every OS.",
    );
    t.skip("Actual NTFS case sensitivity is only available on Windows");
    return;
  }
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-case-identity-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  try {
    execFileSync("fsutil.exe", ["file", "setCaseSensitiveInfo", directory, "enable"], {
      stdio: "pipe",
      timeout: 5000,
    });
  } catch (error) {
    const reason = `The runner cannot enable per-directory case sensitivity: ${String(error)}`;
    recordCapability(t, "unavailable", reason);
    t.skip(reason);
    return;
  }
  const folders = [path.join(directory, "Engine"), path.join(directory, "engine")];
  fs.mkdirSync(folders[0]);
  try {
    fs.mkdirSync(folders[1]);
  } catch (error) {
    if (error.code !== "EEXIST") throw error;
    const reason =
      "fsutil accepted the flag, but distinct-case directory creation is not supported by this runner's filesystem API.";
    recordCapability(t, "unavailable", reason);
    t.skip(reason);
    return;
  }
  const binaries = folders.map((folder, index) => {
    const packageRoot = path.join(folder, "node_modules/vize");
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(folder, "package.json"), "{}");
    fs.writeFileSync(
      path.join(packageRoot, "package.json"),
      JSON.stringify({ name: "vize", version: "9.8.7", bin: "vize.cjs" }),
    );
    const binary = path.join(packageRoot, "vize.cjs");
    fs.writeFileSync(
      binary,
      `#!/usr/bin/env node\n// Distinct engine build ${index}, both report vize 9.8.7.\n`,
    );
    return fs.realpathSync(binary);
  });
  assert.notEqual(binaries[0], binaries[1]);
  assert.notDeepEqual(fs.readFileSync(binaries[0]), fs.readFileSync(binaries[1]));
  const result = discoverWorkspaceServers(folders, true);
  assert.equal(result.candidates.length, 0);
  assert.match(result.reason, /different Vize installations/);
  recordCapability(
    t,
    "verified",
    "fsutil enabled NTFS case sensitivity and both distinct same-version engine files were refused.",
    { realpaths: binaries },
  );
});
