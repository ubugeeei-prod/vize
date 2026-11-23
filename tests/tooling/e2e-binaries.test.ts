import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { toolingProgram } from "./support/program-path.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

interface HelperCommandResult {
  status: number | null;
  stderr: string;
  stdout: string;
}

test("compiler e2e binary helpers respect explicit env overrides", () => {
  const vizeBin = path.join(root, "target", "vize-tests", "vize-override");
  const corsaBin = path.join(root, "target", "vize-tests", "corsa-override");
  const result = runAppsHelper({
    env: {
      CORSA_BIN: corsaBin,
      VIZE_BIN: vizeBin,
    },
    mode: "paths",
  });

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
});

test("compiler e2e binary helpers fail loudly when the vize override is missing", () => {
  const vizeBin = path.join(root, "target", "vize-tests", "missing-vize");
  const corsaBin = path.join(root, "target", "vize-tests", "missing-corsa");
  const result = runAppsHelper({
    env: {
      CORSA_BIN: corsaBin,
      VIZE_BIN: vizeBin,
    },
    mode: "require",
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /vize CLI binary not found/);
  assert.ok(result.stderr.includes(vizeBin));
});

test("compiler e2e binary helpers fail loudly when the checker override is missing", () => {
  const corsaBin = path.join(root, "target", "vize-tests", "missing-corsa");
  const result = runAppsHelper({
    env: {
      CORSA_BIN: corsaBin,
      VIZE_BIN: process.execPath,
    },
    mode: "require",
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /TypeScript 7\/Corsa binary not found/);
  assert.ok(result.stderr.includes(corsaBin));
});

function runAppsHelper(options: {
  env: Record<"CORSA_BIN" | "VIZE_BIN", string>;
  mode: "paths" | "require";
}): HelperCommandResult {
  const result = spawnSync(
    process.execPath,
    [toolingProgram("runtime/inspect-e2e-binaries.js"), options.mode],
    {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        ...options.env,
      },
    },
  );

  if (result.error != null) {
    throw result.error;
  }

  return {
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}
