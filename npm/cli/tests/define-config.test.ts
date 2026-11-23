import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vite-plus/test";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));

function withInstalledConfig(run: (directory: string, loads: string) => void): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-cold-native-"));
  const modules = path.join(directory, "node_modules");
  const installed = path.join(modules, "vize");
  const loads = path.join(directory, "binding-loads.txt");
  try {
    fs.mkdirSync(installed, { recursive: true });
    fs.copyFileSync(path.join(packageRoot, "package.json"), path.join(installed, "package.json"));
    fs.cpSync(path.join(packageRoot, "dist"), path.join(installed, "dist"), { recursive: true });
    for (const [name, type, entry] of [
      ["@vizejs/native", "commonjs", "index.cjs"],
      ["oxc-transform", "module", "index.mjs"],
    ]) {
      const dependency = path.join(modules, name);
      fs.mkdirSync(dependency, { recursive: true });
      fs.writeFileSync(
        path.join(dependency, "package.json"),
        JSON.stringify({ name, type, exports: `./${entry}` }),
      );
      const source = type === "module" ? "unavailable-transform.js" : "unavailable-native.cjs";
      fs.copyFileSync(
        new URL("./fixtures/installed-config/" + source, import.meta.url),
        path.join(dependency, entry),
      );
    }
    run(directory, loads);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function execute(directory: string, name: string, ...args: string[]): void {
  const driver = path.join(directory, name + ".mjs");
  fs.copyFileSync(new URL("./fixtures/installed-config/" + name + ".js", import.meta.url), driver);
  const result = spawnSync(process.execPath, [driver, ...args], {
    cwd: directory,
    encoding: "utf8",
    timeout: 15_000,
    maxBuffer: 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.stderr);
  assert.equal(result.signal, null, result.stderr);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "");
  assert.equal(result.stderr, "");
}

test("installed defineConfig preserves imported CLI settings without loading native bindings", () => {
  withInstalledConfig((directory, loads) => {
    fs.copyFileSync(
      new URL("./fixtures/installed-config/shared-config.js", import.meta.url),
      path.join(directory, "shared.mjs"),
    );
    execute(directory, "define");
    assert.equal(fs.existsSync(loads), false, "helper import must not load either native binding");
  });
});

test("installed resolveConfigExport still requires native structural normalization", () => {
  withInstalledConfig((directory, loads) => {
    execute(directory, "resolve");
    assert.equal(fs.readFileSync(loads, "utf8"), "@vizejs/native\n");
  });
});

test("installed canonical Vite config keeps native normalization failures visible", () => {
  withInstalledConfig((directory, loads) => {
    fs.writeFileSync(
      path.join(directory, "vite.config.mjs"),
      "export default {vue:{linter:{enabled:true}}};",
    );
    execute(directory, "canonical");
    assert.equal(fs.readFileSync(loads, "utf8"), "@vizejs/native\n");
  });
});

test("installed TypeScript Vite config loading still requires its native transformer", () => {
  withInstalledConfig((directory, loads) => {
    fs.writeFileSync(
      path.join(directory, "vite.config.ts"),
      "export default { vue: { linter: { enabled: true } } };\n",
    );
    execute(directory, "typescript");
    assert.equal(fs.readFileSync(loads, "utf8"), "oxc-transform\n");
  });
});

test("installed retired config files report migration without loading native bindings", () => {
  for (const extension of ["json", "ts", "mjs"]) {
    withInstalledConfig((directory, loads) => {
      const file = `vize.config.${extension}`;
      fs.writeFileSync(
        path.join(directory, file),
        extension === "json" ? "{}" : "export default {};",
      );
      execute(directory, "retired", file);
      assert.equal(
        fs.existsSync(loads),
        false,
        "retired files must fail before either native binding",
      );
    });
  }
});
