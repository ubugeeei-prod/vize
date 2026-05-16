import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const helperModuleUrl = new URL("../_helpers/apps.ts", import.meta.url).href;

interface HelperCommandResult {
  status: number | null;
  stderr: string;
  stdout: string;
}

test("compiler e2e binary helpers respect explicit env overrides", () => {
  const vizeBin = path.join(root, "__agent_only", "vize-override");
  const corsaBin = path.join(root, "__agent_only", "corsa-override");
  const result = runAppsHelper({
    env: {
      CORSA_BIN: corsaBin,
      VIZE_BIN: vizeBin,
    },
    script: [
      `if (helpers.VIZE_BIN !== ${JSON.stringify(vizeBin)}) throw new Error(helpers.VIZE_BIN);`,
      `if (helpers.CORSA_BIN !== ${JSON.stringify(corsaBin)}) throw new Error(helpers.CORSA_BIN);`,
    ].join("\n"),
  });

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
});

test("compiler e2e binary helpers fail loudly when the vize override is missing", () => {
  const vizeBin = path.join(root, "__agent_only", "missing-vize");
  const corsaBin = path.join(root, "__agent_only", "missing-corsa");
  const result = runAppsHelper({
    env: {
      CORSA_BIN: corsaBin,
      VIZE_BIN: vizeBin,
    },
    script: "helpers.requireVizeAndCorsaBins();",
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /vize CLI binary not found/);
  assert.ok(result.stderr.includes(vizeBin));
});

test("compiler e2e binary helpers fail loudly when the checker override is missing", () => {
  const corsaBin = path.join(root, "__agent_only", "missing-corsa");
  const result = runAppsHelper({
    env: {
      CORSA_BIN: corsaBin,
      VIZE_BIN: process.execPath,
    },
    script: "helpers.requireVizeAndCorsaBins();",
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Corsa\/tsgo binary not found/);
  assert.ok(result.stderr.includes(corsaBin));
});

function runAppsHelper(options: {
  env: Record<"CORSA_BIN" | "VIZE_BIN", string>;
  script: string;
}): HelperCommandResult {
  const result = spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "--eval",
      [`const helpers = await import(${JSON.stringify(helperModuleUrl)});`, options.script].join(
        "\n",
      ),
    ],
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
