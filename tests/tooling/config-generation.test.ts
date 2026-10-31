import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { generationAndCliTasks } from "../../tools/config/vite-plus/tasks/generation-cli.ts";
import { repoRoot, runToolCommand } from "./_helpers/tool-commands.ts";

const root = repoRoot;

type TaskShape = {
  command: string;
};

function runWorkspaceBinary(name: string, args: string[]): SpawnSyncReturns<string> {
  return spawnSync(process.platform === "win32" ? "pnpm.cmd" : "pnpm", ["exec", name, ...args], {
    cwd: root,
    encoding: "utf8",
  });
}

/**
 * `pkl` is declared by `npm/cli`, so pnpm links its bin only into that package.
 * `pnpm exec pkl` from the repository root cannot resolve it, hence the direct path.
 */
function runPkl(args: string[]): SpawnSyncReturns<string> {
  const pkl = path.join(
    root,
    "npm",
    "cli",
    "node_modules",
    ".bin",
    process.platform === "win32" ? "pkl.cmd" : "pkl",
  );
  const cache = process.env.VIZE_PKL_SCHEMA_CACHE;
  const preparedArgs = cache ? [args[0], "--cache-dir", cache, ...args.slice(1)] : args;
  return spawnSync(pkl, preparedArgs, { cwd: root, encoding: "utf8" });
}

function assertCommandSucceeded<
  Result extends Pick<SpawnSyncReturns<string>, "status" | "error" | "stderr" | "stdout">,
>(result: Result, description: string): asserts result is Result & { status: 0 } {
  assert.equal(
    result.status,
    0,
    `${description}\n${result.error?.message ?? ""}\n${result.stderr ?? ""}\n${result.stdout ?? ""}`.trim(),
  );
}

test("root config generation resolves and reproduces checked-in artifacts", () => {
  const schemaTask = generationAndCliTasks["gen:schema"] as TaskShape;
  assert.match(schemaTask.command, /--project-dir\s+npm\/cli\/pkl\b/);

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-generation-"));
  const schemaPath = path.join(tempDir, "npm", "cli", "schemas", "vize.config.schema.json");
  const typesPath = path.join(tempDir, "npm", "cli", "src", "types", "generated.ts");

  try {
    fs.mkdirSync(path.dirname(schemaPath), { recursive: true });
    fs.mkdirSync(path.dirname(typesPath), { recursive: true });

    const pklResult = runPkl([
      "eval",
      "--project-dir",
      "npm/cli/pkl",
      "-f",
      "json",
      "npm/cli/pkl/jsonschema/generate.pkl",
      "-o",
      schemaPath,
    ]);
    assertCommandSucceeded(pklResult, "Pkl schema generation failed");

    const json2tsResult = runWorkspaceBinary("json2ts", ["-i", schemaPath, "-o", typesPath]);
    assertCommandSucceeded(json2tsResult, "TypeScript declaration generation failed");

    const postprocessResult = runToolCommand("tools/support/tasks/postprocess-types.ts", [], {
      env: { VIZE_REPO_ROOT: tempDir },
    });
    assertCommandSucceeded(postprocessResult, "Generated declaration post-processing failed");

    const formatResult = runWorkspaceBinary("oxfmt", ["--write", schemaPath, typesPath]);
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

test("both public Pkl entrypoints validate declared prop casing policies", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pkl-prop-casing-"));
  const policies = [
    { name: "default", source: "", expected: "camelCase", field: "casing" },
    { name: "camel", source: 'casing = "camelCase"', expected: "camelCase", field: "casing" },
    { name: "snake", source: 'casing = "snake_case"', expected: "snake_case", field: "casing" },
    { name: "invalid", source: 'casing = "unsupported"', expected: null, field: "casing" },
    {
      name: "unexposed-ignore",
      source: 'ignoreProps = new Listing { "authoredName" }',
      expected: null,
      field: "ignoreProps",
    },
  ];
  try {
    for (const { entrypoint, types } of [
      { entrypoint: "vize.pkl", types: "vize.pkl" },
      { entrypoint: "VizeConfig.pkl", types: "LinterConfig.pkl" },
    ]) {
      for (const policy of policies) {
        const input = path.join(directory, `${entrypoint}-${policy.name}.pkl`);
        const source = [
          `amends ${JSON.stringify(pathToFileURL(path.join(root, "npm/cli/pkl", entrypoint)).href)}`,
          `import ${JSON.stringify(pathToFileURL(path.join(root, "npm/cli/pkl", types)).href)} as Types`,
          "linter = new Types.LinterConfig {",
          "  ruleOptions = new Types.LintRuleOptions {",
          "    `vue/prop-name-casing` = new Types.PropNameCasingOptions {",
          `      ${policy.source}`,
          "    }",
          "  }",
          "}",
          "",
        ].join("\n");
        fs.writeFileSync(input, source);
        const result = runPkl([
          "eval",
          "--project-dir",
          "npm/cli/pkl",
          "-f",
          "json",
          "-x",
          "linter.ruleOptions.`vue/prop-name-casing`",
          input,
        ]);
        console.log(
          "vize-prop-casing-pkl-packet",
          JSON.stringify({
            entrypoint,
            policy: policy.name,
            source,
            status: result.status,
            signal: result.signal,
            stdout: result.stdout,
            stderr: result.stderr,
            error: result.error?.message ?? null,
          }),
        );
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        if (policy.expected === null) {
          assert.equal(result.status, 1);
          assert.equal(result.stdout, "");
          assert.ok(result.stderr.includes(policy.field), result.stderr);
        } else {
          assertCommandSucceeded(result, `${entrypoint}: ${policy.name}`);
          assert.equal(result.stderr, "");
          assert.deepEqual(JSON.parse(result.stdout), { casing: policy.expected });
        }
        assert.equal(fs.readFileSync(input, "utf8"), source);
      }
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
