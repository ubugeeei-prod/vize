import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "vite-plus/test";

const environmentNames = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
] as const;

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-user-runtime-"));
  const packageRoot = path.join(root, "node_modules", "vize");
  const platformPackage = "@typescript/typescript-" + process.platform + "-" + process.arch;
  const stalePackage = path.join(packageRoot, "node_modules", platformPackage);
  fs.mkdirSync(path.join(stalePackage, "lib"), { recursive: true });
  fs.writeFileSync(
    path.join(stalePackage, "package.json"),
    JSON.stringify({
      name: platformPackage,
      version: "7.0.2",
    }),
  );
  fs.writeFileSync(
    path.join(stalePackage, "lib", process.platform === "win32" ? "tsc.exe" : "tsc"),
    "stale bundled runtime must not be selected",
  );
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    JSON.stringify({
      name: "vize",
      type: "module",
      optionalDependencies: { [platformPackage]: "7.0.2" },
    }),
  );
  const stub = path.join(packageRoot, "node_modules", "@vizejs", "native");
  fs.mkdirSync(stub, { recursive: true });
  fs.writeFileSync(
    path.join(stub, "package.json"),
    JSON.stringify({ name: "@vizejs/native", main: "index.cjs" }),
  );
  fs.writeFileSync(
    path.join(stub, "index.cjs"),
    "exports.runCli = (args) => process.stdout.write(JSON.stringify({args, environment: Object.fromEntries(" +
      JSON.stringify(environmentNames) +
      ".map(name => [name, process.env[name] ?? null]))}));\n",
  );
  const installed = path.join(packageRoot, "dist", "cli.mjs");
  fs.cpSync(fileURLToPath(new URL("../dist", import.meta.url)), path.dirname(installed), {
    recursive: true,
  });
  const run = (args: string[], override: Record<string, string> = {}) => {
    const environment = { ...process.env };
    for (const name of environmentNames) delete environment[name];
    Object.assign(environment, override);
    const result = spawnSync(process.execPath, [installed, ...args], {
      cwd: root,
      env: environment,
      encoding: "utf8",
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, "");
    return JSON.parse(result.stdout) as {
      args: string[];
      environment: Record<string, string | null>;
    };
  };
  return { run, remove: () => fs.rmSync(root, { force: true, recursive: true }) };
}

test("CLI, LSP, mapper and syntax-only commands delegate without selecting a stale bundled runtime", () => {
  const f = fixture();
  try {
    for (const command of ["check", "lsp", "content-mapper", "lint", "fmt", "--version"]) {
      assert.deepEqual(f.run([command]), {
        args: [command],
        environment: Object.fromEntries(environmentNames.map((name) => [name, null])),
      });
    }
  } finally {
    f.remove();
  }
});

test("every explicit runtime environment and CLI path reaches the shared native resolver unchanged", () => {
  const f = fixture();
  try {
    for (const name of environmentNames) {
      const value = "/project/日本語/native runtime";
      assert.deepEqual(f.run(["check"], { [name]: value }), {
        args: ["check"],
        environment: Object.fromEntries(
          environmentNames.map((key) => [key, key === name ? value : null]),
        ),
      });
    }
    const args = ["check", "--corsa-path", "/project/config/native runtime"];
    assert.deepEqual(f.run(args), {
      args,
      environment: Object.fromEntries(environmentNames.map((name) => [name, null])),
    });
  } finally {
    f.remove();
  }
});

test("published CLI manifest does not automatically install a native TypeScript runtime", () => {
  const manifest = JSON.parse(
    fs.readFileSync(fileURLToPath(new URL("../package.json", import.meta.url)), "utf8"),
  ) as {
    optionalDependencies?: Record<string, string>;
  };
  assert.deepEqual(
    Object.keys(manifest.optionalDependencies ?? {}).filter(
      (name) => name === "typescript" || name.startsWith("@typescript/"),
    ),
    [],
  );
});
