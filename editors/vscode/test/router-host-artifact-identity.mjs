import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { validateBuildReceipt } from "../../../tests/differential/build-receipt.ts";

export function fileIdentity(file) {
  const bytes = fs.readFileSync(file);
  return {
    path: fs.realpathSync(file),
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
}

export function recordRouterHostIdentity({ output, root, server, corsa, vsix, vscodeVersion }) {
  const receiptPath = `${server}.differential-build.json`;
  assert.ok(fs.existsSync(receiptPath), "the native artifact must retain its source receipt");
  const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  const head = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(head.status, 0);
  const native = fileIdentity(server);
  fs.copyFileSync(receiptPath, path.join(output, "native-source-receipt.json"));
  fs.copyFileSync(vsix, path.join(output, "vize.vsix"));
  validateBuildReceipt(
    receipt,
    {
      sourceRevision: head.stdout.trim(),
      binaryPath: path.relative(root, server).split(path.sep).join("/"),
      binarySha256: native.sha256,
      cliVersion: run(server, ["--version"], output).toString("utf8").trim(),
    },
    receipt.recipe,
  );
  const packagedManifest = JSON.parse(
    run("unzip", ["-p", vsix, "extension/package.json"], output).toString("utf8"),
  );
  const bundle = run("unzip", ["-p", vsix, "extension/dist/extension.cjs"], output);
  const corsaVersion = run(corsa, ["--version"], output).toString("utf8").trim();
  assert.equal(corsaVersion, "Version 7.0.2");
  const value = {
    schema: "vize.router-installed-editor-identity",
    version: 1,
    harnessCommit: head.stdout.trim(),
    vscodeVersion,
    vsix: fileIdentity(vsix),
    packagedManifest,
    packagedBundleSha256: createHash("sha256").update(bundle).digest("hex"),
    native,
    corsa: fileIdentity(corsa),
    corsaVersion,
    receipt,
    selection: "documented workspace vize.serverPath through passive byte proxy",
    qualification: "same-source controlled server; automatic discovery is outside this campaign",
  };
  fs.writeFileSync(
    path.join(output, "producer-identity.json"),
    `${JSON.stringify(value, null, 2)}\n`,
  );
  return value;
}

function run(executable, args, output) {
  const result = spawnSync(executable, args, { timeout: 30_000, maxBuffer: 32 * 1024 * 1024 });
  fs.appendFileSync(
    path.join(output, "identity-processes.jsonl"),
    `${JSON.stringify({ executable, args, status: result.status, signal: result.signal, error: result.error?.message ?? null, stdoutBase64: result.stdout?.toString("base64") ?? null, stderrBase64: result.stderr?.toString("base64") ?? null })}\n`,
  );
  assert.ifError(result.error);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr.toString("utf8"));
  return result.stdout;
}
