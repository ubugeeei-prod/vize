import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vite-plus/test";

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

test("init and setup preserve every supported dedicated config byte-for-byte", async () => {
  for (const filename of VIZE_CONFIG_FILES) {
    const root = temporaryProject(`preserve-${path.extname(filename).slice(1)}`);
    try {
      writeManifest(root, { name: "fixture", private: true });
      const source = "// User-owned configuration; never replace or parse during init.\n";
      write(root, filename, source);
      const tsconfig = '{"files":[],"references":[{"path":"./app"}]}\n';
      write(root, "tsconfig.json", tsconfig);

      await runInit(root, TYPECHECK_ARGS);
      const setup = setupProject({ root, install: false });

      assert.equal(read(root, filename), source, filename);
      assert.equal(read(root, "tsconfig.json"), tsconfig, filename);
      assert.ok(setup.preservedFiles.includes(filename), filename);
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
