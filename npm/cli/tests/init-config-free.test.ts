import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vite-plus/test";

import { CONFIG_FREE_PROJECT_SHAPES } from "../../../tools/support/compat/npm/smoke-release-init-config-free.mjs";
import { FRESH_INIT_MATRIX } from "../../../tools/support/compat/npm/smoke-release-init-shapes.mjs";
import { PACKAGE_MANAGERS } from "../../../tools/support/compat/npm/smoke-release-init-managers.mjs";
import {
  expectedInitOutput,
  writeFiles,
} from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";

import { setupProject } from "../src/setup.ts";
import { VIZE_CONFIG_FILES } from "../src/setup/config.ts";
import { read, runInit, temporaryProject, write, writeManifest } from "./init-support.ts";

const TYPECHECK_ARGS = [
  "--yes",
  "--no-lint",
  "--no-bundler",
  "--no-fmt",
  "--typecheck",
  "--no-editor",
  "--no-install",
] as const;

test("formatter-only init uses defaults and does not scaffold typecheck settings", async () => {
  const root = temporaryProject("formatter-defaults");
  try {
    writeManifest(root, { name: "fixture", private: true });
    const args = [...TYPECHECK_ARGS, "--fmt", "--no-typecheck"];

    const first = await runInit(root, args);

    assert.deepEqual(first.written, ["package.json"]);
    assert.deepEqual(fs.readdirSync(root), ["package.json"]);
    assert.deepEqual(JSON.parse(read(root, "package.json")).scripts, {
      "vize:fmt": "vize fmt --check src",
      "vize:fmt:fix": "vize fmt --write src",
    });
    assert.equal(
      first.plan?.features.find((feature) => feature.id === "fmt")?.outcome,
      "configured",
    );
    const second = await runInit(root, args);
    assert.deepEqual(second.written, []);
    assert.equal(
      second.plan?.features.find((feature) => feature.id === "fmt")?.outcome,
      "unchanged",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("setup scaffolds a strict TypeScript project and preserves its project references", () => {
  const root = temporaryProject("setup-typescript");
  try {
    writeManifest(root, { name: "fixture", devDependencies: { typescript: "^6.0.0" } });

    const result = setupProject({ root, install: false });

    assert.ok(result.createdFiles.includes("tsconfig.json"));
    const config = JSON.parse(read(root, "tsconfig.json"));
    assert.equal(config.compilerOptions.strict, true);
    assert.equal(config.compilerOptions.allowJs, undefined);
    assert.equal(config.compilerOptions.checkJs, undefined);
    assert.deepEqual(config.include, ["src/**/*"]);
    assert.deepEqual(
      fs.readdirSync(root).filter((name) => name.startsWith("vize.config.")),
      [],
    );

    const references = '{\n  "files": [],\n  "references": [{ "path": "./packages/app" }]\n}\n';
    write(root, "tsconfig.json", references);
    const second = setupProject({ root, install: false });
    assert.deepEqual(second.createdFiles, []);
    assert.ok(second.preservedFiles.includes("tsconfig.json"));
    assert.equal(read(root, "tsconfig.json"), references);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("init and setup preserve removed dedicated files beside a shared Vite config", async () => {
  for (const filename of VIZE_CONFIG_FILES) {
    const root = temporaryProject(`preserve-${path.extname(filename).slice(1)}`);
    try {
      writeManifest(root, { name: "fixture", private: true });
      const source = "// User-owned configuration; never replace or parse during init.\n";
      write(root, filename, source);
      write(root, "vite.config.mjs", "export default { vue: {} };\n");
      const tsconfig = '{"files":[],"references":[{"path":"./app"}]}\n';
      write(root, "tsconfig.json", tsconfig);

      const init = await runInit(root, TYPECHECK_ARGS);
      assert.match(init.output, /removed config:.*not read/u);
      const setup = setupProject({ root, install: false });

      assert.equal(read(root, filename), source, filename);
      assert.equal(read(root, "tsconfig.json"), tsconfig, filename);
      assert.ok(setup.preservedFiles.includes(filename), filename);
      assert.ok(
        setup.migrationWarnings.some((warning) =>
          warning.includes(`${filename} is no longer read`),
        ),
      );
      assert.deepEqual(
        fs.readdirSync(root).filter((name) => name.startsWith("vize.config.")),
        [filename],
      );
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  }
});

test("the packed init command configures Vite+ without a dedicated Vize config", () => {
  const root = temporaryProject("packed-config-free");
  try {
    writeManifest(root, {
      name: "fixture",
      private: true,
      type: "module",
      devDependencies: { "vite-plus": "^0.1.0", typescript: "^6.0.0" },
    });
    write(
      root,
      "vite.config.ts",
      'import { defineConfig } from "vite-plus";\n\nexport default defineConfig({ plugins: [] });\n',
    );
    const cli = new URL("../dist/cli.mjs", import.meta.url);

    const output = execFileSync(
      process.execPath,
      [
        fileURLToPath(cli),
        "init",
        root,
        "--yes",
        "--lint",
        "--vite",
        "--fmt",
        "--typecheck",
        "--editor",
        "--no-install",
      ],
      { encoding: "utf8" },
    );

    assert.deepEqual(
      fs.readdirSync(root).filter((name) => name.startsWith("vize.config.")),
      [],
    );
    assert.match(read(root, "vite.config.ts"), /plugins: \[vize\(\)\]/u);
    assert.match(read(root, "vite.config.ts"), /lint: createVizeLintConfig/u);
    assert.equal(JSON.parse(read(root, "tsconfig.json")).compilerOptions.strict, true);
    assert.equal(JSON.parse(read(root, "package.json")).scripts["vize:check"], "vize check");
    assert.equal(fs.existsSync(path.join(root, "oxlint.config.ts")), false);
    assert.match(output, /will create tsconfig\.json/u);
    assert.doesNotMatch(output, /will create vize\.config/u);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("init and setup reject dedicated-only projects before changing any files", async () => {
  for (const filename of VIZE_CONFIG_FILES) {
    const root = temporaryProject("removed-config");
    try {
      writeManifest(root, { name: "fixture", private: true });
      write(root, filename, "// Keep user settings intact.\n");
      const manifest = read(root, "package.json");
      await assert.rejects(
        runInit(root, TYPECHECK_ARGS),
        /no longer supported; move Vue settings to vue:/u,
      );
      assert.throws(
        () => setupProject({ root, install: false }),
        /no longer supported; move Vue settings to vue:/u,
      );
      assert.equal(read(root, "package.json"), manifest);
      assert.equal(read(root, filename), "// Keep user settings intact.\n");
      assert.equal(fs.existsSync(path.join(root, "tsconfig.json")), false);
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  }
});

test("current release fixtures match complete packed init plans for every manager cell", () => {
  const cli = fileURLToPath(new URL("../dist/cli.mjs", import.meta.url));
  const peers = { typescript: "6.0.3", vite: "8.0.0", "vite-plus": "0.1.0", vue: "3.5.0" };
  for (const cell of FRESH_INIT_MATRIX) {
    const shape = CONFIG_FREE_PROJECT_SHAPES[cell.shape];
    const manager = PACKAGE_MANAGERS[cell.packageManager];
    const root = temporaryProject("release-plan");
    try {
      writeFiles(root, { ...shape.files(peers), ...manager.projectFiles });
      // Bootstrap only the authored package-manager detection signal. Dependency
      // installation and the unchanged diagnostic triples belong to the release smoke.
      write(root, manager.lockfile, "");
      const run = (args: string[]) =>
        execFileSync(process.execPath, [cli, "init", root, ...shape.initFlags, ...args], {
          encoding: "utf8",
        });
      assert.equal(run(["--dry-run"]), expectedInitOutput(shape, root, manager, "dry"));
      assert.equal(run(["--no-install"]), expectedInitOutput(shape, root, manager, "apply"));
      for (const [name, source] of Object.entries(shape.expectedFiles))
        assert.equal(read(root, name), source);
      const manifest = JSON.parse(read(root, "package.json"));
      for (const dependency of shape.plannedDependencies)
        manifest.devDependencies[dependency] = "*";
      write(root, "package.json", JSON.stringify(manifest));
      assert.equal(run(["--no-install"]), expectedInitOutput(shape, root, manager, "rerun"));
      assert.equal(
        fs.readdirSync(root).some((name) => name.startsWith("vize.config.")),
        false,
      );
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  }
});
