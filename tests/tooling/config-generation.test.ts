import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { generationAndCliTasks } from "../../tools/vite-plus/tasks/generation-cli.ts";
import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

const root = repoRoot;

type TaskShape = {
  command: string;
};

function workspaceBinary(name: string): string {
  return path.join(
    root,
    "node_modules",
    ".bin",
    process.platform === "win32" ? `${name}.cmd` : name,
  );
}

function assertCommandSucceeded(
  result: SpawnSyncReturns<string>,
  description: string,
): asserts result is SpawnSyncReturns<string> & { status: 0 } {
  assert.equal(
    result.status,
    0,
    `${description}\n${result.error?.message ?? ""}\n${result.stderr ?? ""}\n${result.stdout ?? ""}`.trim(),
  );
}

test("root config generation resolves and reproduces checked-in artifacts", () => {
  const schemaTask = generationAndCliTasks["gen:schema"] as TaskShape;
  assert.match(
    schemaTask.command,
    /pkl eval --project-dir npm\/cli\/pkl -f json npm\/cli\/pkl\/jsonschema\/generate\.pkl/,
  );
  const typesTask = generationAndCliTasks["gen:types"] as TaskShape;
  assert.match(
    typesTask.command,
    /oxfmt --write npm\/cli\/schemas\/vize\.config\.schema\.json npm\/cli\/src\/types\/generated\.ts/,
  );

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-generation-"));
  const schemaPath = path.join(tempDir, "npm", "cli", "schemas", "vize.config.schema.json");
  const typesPath = path.join(tempDir, "npm", "cli", "src", "types", "generated.ts");
  const pkl = path.join(
    root,
    "npm",
    "cli",
    "node_modules",
    ".bin",
    process.platform === "win32" ? "pkl.cmd" : "pkl",
  );

  try {
    fs.mkdirSync(path.dirname(schemaPath), { recursive: true });
    fs.mkdirSync(path.dirname(typesPath), { recursive: true });

    const pklResult = spawnSync(
      pkl,
      [
        "eval",
        "--project-dir",
        "npm/cli/pkl",
        "-f",
        "json",
        "npm/cli/pkl/jsonschema/generate.pkl",
        "-o",
        schemaPath,
      ],
      { cwd: root, encoding: "utf8" },
    );
    assertCommandSucceeded(pklResult, "Pkl schema generation failed");

    const json2tsResult = spawnSync(
      workspaceBinary("json2ts"),
      ["-i", schemaPath, "-o", typesPath],
      { cwd: root, encoding: "utf8" },
    );
    assertCommandSucceeded(json2tsResult, "TypeScript declaration generation failed");

    const postprocessResult = runMoonScript("postprocess_types", [], {
      env: { VIZE_REPO_ROOT: tempDir },
    });
    assertCommandSucceeded(postprocessResult, "Generated declaration post-processing failed");

    const formatResult = spawnSync(workspaceBinary("oxfmt"), ["--write", schemaPath, typesPath], {
      cwd: root,
      encoding: "utf8",
    });
    assertCommandSucceeded(formatResult, "Generated artifact formatting failed");

    assert.equal(
      fs.readFileSync(schemaPath, "utf8"),
      fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
      "generated schema is stale",
    );
    assert.equal(
      fs.readFileSync(typesPath, "utf8"),
      fs.readFileSync(path.join(root, "npm/cli/src/types/generated.ts"), "utf8"),
      "generated TypeScript declarations are stale",
    );
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
