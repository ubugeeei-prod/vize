import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { runFreshProjectInitChecks } from "./smoke-release-init-fresh.mjs";
import { runInitTypecheckChecks } from "./smoke-release-init-typecheck.mjs";

export const RUNTIME_PEER_DEPENDENCIES = {
  typescript: "6.0.3",
  vite: "^8.0.0",
  "vite-plus": "0.1.24",
  vue: "3.5.34",
};

function writeRuntimeSmokeProject(installDir) {
  const sourceDir = path.join(installDir, "src");
  fs.mkdirSync(sourceDir, { recursive: true });
  fs.writeFileSync(
    path.join(installDir, "index.html"),
    '<div id="app"></div><script type="module" src="/src/main.ts"></script>\n',
  );
  fs.writeFileSync(
    path.join(installDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          lib: ["ES2022", "DOM", "DOM.Iterable"],
          module: "ESNext",
          moduleResolution: "Bundler",
          strict: true,
          target: "ES2022",
          types: [],
        },
        include: ["src/**/*.ts", "src/**/*.vue"],
      },
      null,
      2,
    ),
  );
  fs.writeFileSync(
    path.join(installDir, "vite.config.mjs"),
    [
      'import { defineConfig } from "vite";',
      'import vize from "@vizejs/vite-plugin";',
      "",
      "export default defineConfig({",
      "  plugins: [vize()],",
      '  build: { outDir: "dist", emptyOutDir: true },',
      "});",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(sourceDir, "App.vue"),
    [
      "<template>",
      '  <button class="smoke" @click="count++">{{ label }} {{ count }}</button>',
      "</template>",
      "",
      '<script setup lang="ts">',
      'import { ref } from "vue";',
      "",
      'const label: string = "vize smoke";',
      "const count = ref(0);",
      "</script>",
      "",
      "<style scoped>",
      ".smoke {",
      "  color: #0f766e;",
      "}",
      "</style>",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(sourceDir, "main.ts"),
    [
      'import { createApp } from "vue";',
      'import App from "./App.vue";',
      "",
      'createApp(App).mount("#app");',
      "",
    ].join("\n"),
  );
}

export function assertInstalledMapperContract(installDir) {
  const packageRoot = path.join(installDir, "node_modules", "vize");
  assert.equal(fs.lstatSync(packageRoot).isSymbolicLink(), false);
  const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
  assert.deepEqual(
    manifest.typescript?.contentMapper,
    {
      exec: ["node", "./bin/vize", "content-mapper"],
      compilerOptions: ["noUnusedLocals"],
    },
    `${path.join(packageRoot, "package.json")} must expose the production typescript.contentMapper contract`,
  );
  const mapperPath = path.join(packageRoot, "bin", "vize");
  assert.ok(fs.existsSync(mapperPath), `${mapperPath} must exist`);
  const mapperBin = fs.statSync(mapperPath);
  assert.ok(mapperBin.isFile());
  if (process.platform !== "win32") {
    assert.notEqual(mapperBin.mode & 0o111, 0, `${mapperPath} must be executable`);
  }
}

export function runInstalledContentMapperChecks(installDir, repoRoot, run) {
  const tsgo = process.env.VIZE_TEST_CONTENT_MAPPER_TSGO;
  if (tsgo == null || tsgo === "") return;

  assertInstalledMapperContract(installDir);
  const projectDir = path.join(installDir, "content mapper project with spaces");
  fs.cpSync(path.join(repoRoot, "crates/vize/tests/fixtures/content_mapper_project"), projectDir, {
    recursive: true,
  });
  const common = ["--runExternalCode", "--pretty", "false"];
  run(tsgo, [...common, "--noEmit", "-p", "tsconfig.json"], { cwd: projectDir });
  const broken = spawnSync(tsgo, [...common, "--noEmit", "-p", "tsconfig.error.json"], {
    cwd: projectDir,
    encoding: "utf8",
  });
  if (broken.error != null) throw broken.error;
  assert.notEqual(broken.status, 0, "installed mapper error fixture unexpectedly passed");
  const brokenOutput = `${broken.stdout}\n${broken.stderr}`;
  const diagnosticLines = brokenOutput.split(/\r?\n/u);
  for (const [file, code] of [
    ["errors/Broken.vue", "TS2322"],
    ["errors/JavaScriptConsumer.js", "TS2322"],
    ["errors/JsxConsumer.jsx", "TS2322"],
    ["src/Unused.vue", "TS6133"],
  ]) {
    assert.ok(
      diagnosticLines.some((line) => line.includes(file) && line.includes(code)),
      `${file} did not report ${code}:\n${brokenOutput}`,
    );
  }
  run(tsgo, [...common, "-p", "tsconfig.emit.json"], { cwd: projectDir });

  const declarationNames = ["App.d.vue.ts", "App.vue.d.ts"];
  assert.ok(
    declarationNames.some((name) => fs.existsSync(path.join(projectDir, "dist", name))),
    "installed mapper did not emit an App.vue declaration",
  );
  assert.ok(fs.existsSync(path.join(projectDir, "dist", "main.d.ts")));
  fs.copyFileSync(
    path.join(projectDir, "consumer", "verify.ts"),
    path.join(projectDir, "dist", "verify.ts"),
  );
  const consumerArgs = [
    "--ignoreConfig",
    "--noEmit",
    "--strict",
    "--module",
    "preserve",
    "--moduleResolution",
    "bundler",
    "--allowArbitraryExtensions",
    "--pretty",
    "false",
    "dist/verify.ts",
  ];
  run(tsgo, consumerArgs, { cwd: projectDir });
  run(
    process.execPath,
    [path.join(installDir, "node_modules", "typescript", "bin", "tsc"), ...consumerArgs],
    { cwd: projectDir },
  );
  console.log("runtime: packed vize Content Mapper check and declaration emit");
}

export function runRuntimeChecks(
  installDir,
  packages,
  { allPackages = packages, repoRoot, resolveInstalledBin, run, tempDir },
) {
  writeRuntimeSmokeProject(installDir);

  if (packages.some((pkg) => pkg.name === "@vizejs/native")) {
    run(
      process.execPath,
      [
        "-e",
        [
          'const required = require("@vizejs/native");',
          "(async () => {",
          'const imported = await import("@vizejs/native");',
          "const importedNative = imported.default ?? imported;",
          "for (const [label, native] of [['require', required], ['import', importedNative]]) {",
          "if (typeof native.compileSfc !== 'function') {",
          "  throw new Error(`compileSfc missing from ${label} smoke`);",
          "}",
          "const result = native.compileSfc(",
          '  \'<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>\',',
          "  { filename: 'Smoke.vue', isTs: true },",
          ");",
          "if (!result || result.errors.length > 0 || typeof result.code !== 'string' || result.code.length === 0) {",
          "  throw new Error(`compileSfc ${label} runtime smoke failed`);",
          "}",
          "}",
          "})().catch((error) => { console.error(error); process.exit(1); });",
        ].join("\n"),
      ],
      { cwd: installDir },
    );
    console.log("runtime: @vizejs/native require/import compileSfc");
  }

  if (packages.some((pkg) => pkg.name === "vize")) {
    const vizeBin = resolveInstalledBin(installDir, "vize", "vize");
    run(process.execPath, [vizeBin, "--version"], { cwd: installDir });
    console.log("runtime: vize --version");
    run(
      process.execPath,
      [vizeBin, "check", "src/App.vue", "--format", "json", "--quiet", "--no-config"],
      { cwd: installDir },
    );
    console.log("runtime: vize check");
    run(
      process.execPath,
      [vizeBin, "lint", "src/App.vue", "--format", "json", "--quiet", "--no-config"],
      { cwd: installDir },
    );
    console.log("runtime: vize lint");
    runInitTypecheckChecks(installDir, vizeBin, repoRoot, RUNTIME_PEER_DEPENDENCIES);
    runFreshProjectInitChecks({
      installDir,
      packed: new Map(allPackages.map((pkg) => [pkg.name, pkg.tarball])),
      peers: RUNTIME_PEER_DEPENDENCIES,
      repoRoot,
      tempDir,
      versions: new Map(allPackages.map((pkg) => [pkg.name, pkg.version])),
      vizeBin,
    });
  }

  if (packages.some((pkg) => pkg.name === "@vizejs/vite-plugin")) {
    const viteBin = resolveInstalledBin(installDir, "vite", "vite");
    run(process.execPath, [viteBin, "build"], { cwd: installDir });
    console.log("runtime: @vizejs/vite-plugin vite build");
  }
}
