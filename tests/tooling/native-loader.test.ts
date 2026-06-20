import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("native loader rejects stale optional binding packages by default", () => {
  const result = runNativeLoaderProbe({ packageVersion: "1.0.0", targetVersion: "0.9.0" });

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /mismatch rejected/);
});

test("native loader requires an explicit escape hatch for stale optional packages", () => {
  const result = runNativeLoaderProbe({
    allowMismatch: true,
    packageVersion: "1.0.0",
    targetVersion: "0.9.0",
  });

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /loaded stale binding/);
});

function runNativeLoaderProbe(options: {
  allowMismatch?: boolean;
  packageVersion: string;
  targetVersion: string;
}): { status: number | null; stdout: string; stderr: string } {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-"));

  try {
    fs.copyFileSync(
      path.join(root, "npm/native/native-targets.js"),
      path.join(tempDir, "native-targets.js"),
    );
    fs.copyFileSync(
      path.join(root, "npm/native/native-binding.js"),
      path.join(tempDir, "native-binding.js"),
    );
    fs.writeFileSync(
      path.join(tempDir, "package.json"),
      JSON.stringify({ name: "@vizejs/native", version: options.packageVersion }, null, 2),
    );
    fs.writeFileSync(path.join(tempDir, "probe.cjs"), probeSource(options.targetVersion));

    const result = spawnSync(process.execPath, ["probe.cjs"], {
      cwd: tempDir,
      encoding: "utf8",
      env: {
        ...process.env,
        ...(options.allowMismatch ? { VIZE_ALLOW_NATIVE_VERSION_MISMATCH: "1" } : {}),
      },
    });

    if (result.error != null) {
      throw result.error;
    }

    return { status: result.status, stdout: result.stdout, stderr: result.stderr };
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
}

function probeSource(targetVersion: string): string {
  return `
const fs = require("node:fs");
const path = require("node:path");
const { nativeTargets } = require("./native-targets");

function causeMessages(error) {
  const messages = [];
  let current = error;
  while (current) {
    messages.push(current.message);
    current = current.cause;
  }
  return messages.join("\\n");
}

const loadErrors = [];
const target = nativeTargets(loadErrors)[0];
if (!target) {
  throw new Error("No native target: " + causeMessages(loadErrors[0]));
}

const packageName = "@vizejs/native-" + target;
const packageDir = path.join(process.cwd(), "node_modules", ...packageName.split("/"));
fs.mkdirSync(packageDir, { recursive: true });
fs.writeFileSync(
  path.join(packageDir, "package.json"),
  JSON.stringify({ name: packageName, version: ${JSON.stringify(targetVersion)}, main: "index.js" }, null, 2),
);
fs.writeFileSync(path.join(packageDir, "index.js"), "module.exports = { marker: 'loaded stale binding' };\\n");

try {
  const binding = require("./native-binding");
  if (binding.marker === "loaded stale binding") {
    console.log("loaded stale binding");
    process.exit(0);
  }
  throw new Error("Unexpected binding");
} catch (error) {
  const messages = causeMessages(error);
  if (
    !messages.includes("Native binding package version mismatch") ||
    !messages.includes("expected 1.0.0 but got ${targetVersion}")
  ) {
    console.error(messages);
    process.exit(1);
  }
  console.log("mismatch rejected");
}
`;
}
