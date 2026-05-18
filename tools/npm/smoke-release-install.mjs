#!/usr/bin/env node

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const dependencySections = [
  "dependencies",
  "optionalDependencies",
  "peerDependencies",
  "devDependencies",
];

function parseArgs(argv) {
  const options = {
    keepTemp: false,
    packageDirs: /** @type {string[]} */ ([]),
    prepareManifests: false,
    runtimeChecks: false,
  };

  for (const arg of argv) {
    if (arg === "--keep-temp") {
      options.keepTemp = true;
      continue;
    }
    if (arg === "--prepare-manifests") {
      options.prepareManifests = true;
      continue;
    }
    if (arg === "--runtime-checks") {
      options.runtimeChecks = true;
      continue;
    }
    if (arg.startsWith("--")) {
      throw new Error(`Unknown argument: ${arg}`);
    }
    options.packageDirs.push(path.resolve(arg));
  }

  if (options.packageDirs.length === 0) {
    throw new Error(
      "Usage: node tools/npm/smoke-release-install.mjs [--prepare-manifests] [--runtime-checks] [--keep-temp] <package-dir>...",
    );
  }

  return options;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? root,
    encoding: "utf8",
    env: options.env ?? process.env,
    input: options.input,
    stdio: ["pipe", "pipe", "pipe"],
  });

  if (result.error != null) {
    throw result.error;
  }

  if (result.status !== 0) {
    const rendered = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      [`${command} ${args.join(" ")} failed with exit ${result.status}`, rendered]
        .filter(Boolean)
        .join("\n"),
    );
  }

  return result.stdout;
}

function preparePublishManifest(packageDir) {
  const scriptPath = path.join(root, "tools/moon/scripts/prepare_npm_publish_manifest.mbtx");
  const moonBin = process.env.MOON_BIN || "moon";
  run(moonBin, ["run", "-q", "--target", "native", "-", "--", packageDir], {
    cwd: root,
    input: fs.readFileSync(scriptPath, "utf8"),
  });
}

function readPackageJson(packageDir) {
  const packageJsonPath = path.join(packageDir, "package.json");
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
  assert.notEqual(packageJson.private, true, `${packageJsonPath} must be publishable`);
  assert.equal(typeof packageJson.name, "string", `${packageJsonPath} is missing name`);
  assert.equal(typeof packageJson.version, "string", `${packageJsonPath} is missing version`);
  return packageJson;
}

function collectStrings(value, out) {
  if (typeof value === "string") {
    out.push(value);
    return;
  }

  if (Array.isArray(value)) {
    for (const item of value) collectStrings(item, out);
    return;
  }

  if (value != null && typeof value === "object") {
    for (const item of Object.values(value)) collectStrings(item, out);
  }
}

function normalizeManifestPath(value) {
  const trimmed = value.trim();
  if (
    trimmed === "" ||
    trimmed.startsWith("#") ||
    trimmed.startsWith("http://") ||
    trimmed.startsWith("https://") ||
    /^[a-z][a-z0-9+.-]*:/i.test(trimmed)
  ) {
    return null;
  }

  return trimmed.startsWith("./") ? trimmed.slice(2) : trimmed;
}

function assertPublishEntrypointsExist(packageDir, packageJson) {
  const manifestPaths = [];
  collectStrings(packageJson.main, manifestPaths);
  collectStrings(packageJson.types, manifestPaths);
  collectStrings(packageJson.bin, manifestPaths);
  collectStrings(packageJson.exports, manifestPaths);

  const missing = [];
  for (const manifestPath of new Set(manifestPaths)) {
    const normalized = normalizeManifestPath(manifestPath);
    if (normalized == null) continue;
    if (!fs.existsSync(path.join(packageDir, normalized))) {
      missing.push(manifestPath);
    }
  }

  assert.deepEqual(missing, [], `${packageJson.name} publishes missing entrypoint files`);
}

function assertNoWorkspaceProtocols(packageDir, packageJson) {
  const unresolved = [];

  for (const section of dependencySections) {
    const dependencies = packageJson[section];
    if (dependencies == null || typeof dependencies !== "object") continue;

    for (const [name, version] of Object.entries(dependencies)) {
      if (typeof version === "string" && /^(workspace|catalog):/.test(version)) {
        unresolved.push(`${section}.${name}=${version}`);
      }
    }
  }

  assert.deepEqual(unresolved, [], `${path.join(packageDir, "package.json")} is not publishable`);
}

function npmAllows(list, current) {
  if (!Array.isArray(list)) return true;
  if (list.includes(`!${current}`)) return false;

  const positives = list.filter((item) => typeof item === "string" && !item.startsWith("!"));
  return positives.length === 0 || positives.includes(current);
}

function currentLibc() {
  if (process.platform !== "linux") return undefined;

  const report = process.report?.getReport?.();
  const header = report?.header;
  if (header != null && typeof header.glibcVersionRuntime === "string") {
    return "glibc";
  }

  const ldd = spawnSync("ldd", ["--version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  const output = `${ldd.stdout ?? ""}\n${ldd.stderr ?? ""}`;
  if (/musl/i.test(output)) {
    return "musl";
  }

  return undefined;
}

function isCompatibleWithCurrentRunner(packageJson) {
  return (
    npmAllows(packageJson.os, process.platform) &&
    npmAllows(packageJson.cpu, process.arch) &&
    npmAllows(packageJson.libc, currentLibc())
  );
}

function packPackage(packageDir, packDir) {
  const before = new Set(fs.readdirSync(packDir));
  run(process.env.NPM_BIN || "npm", ["pack", "--ignore-scripts", "--pack-destination", packDir], {
    cwd: packageDir,
  });

  const created = fs
    .readdirSync(packDir)
    .filter((entry) => entry.endsWith(".tgz") && !before.has(entry))
    .map((entry) => path.join(packDir, entry));

  assert.equal(created.length, 1, `expected exactly one tarball from ${packageDir}`);
  return created[0];
}

function installedPackageDir(nodeModules, name) {
  if (name.startsWith("@")) {
    const [scope, packageName] = name.split("/");
    return path.join(nodeModules, scope, packageName);
  }
  return path.join(nodeModules, name);
}

function assertInstalledPackage(nodeModules, packageInfo) {
  const packageDir = installedPackageDir(nodeModules, packageInfo.name);
  const packageJson = readPackageJson(packageDir);

  assert.equal(packageJson.name, packageInfo.name);
  assert.equal(packageJson.version, packageInfo.version);
  assertNoWorkspaceProtocols(packageDir, packageJson);
  // Per-platform native sub-packages: see the comment near the same guard in
  // main(). The single-host smoke does not ship .node binaries for non-host
  // platforms into the sub-package tarball, so asserting entrypoint existence
  // on the installed tree red-lights the matrix.
  const isPlatformSpecificSubPackage =
    Array.isArray(packageJson.os) || Array.isArray(packageJson.cpu);
  if (!isPlatformSpecificSubPackage) {
    assertPublishEntrypointsExist(packageDir, packageJson);
  }

  if (packageJson.bin != null && process.platform !== "win32") {
    const bins =
      typeof packageJson.bin === "string"
        ? { [packageInfo.name]: packageJson.bin }
        : packageJson.bin;
    for (const binName of Object.keys(bins)) {
      const binPath = path.join(nodeModules, ".bin", binName);
      assert.ok(fs.existsSync(binPath), `${packageInfo.name} did not install ${binName}`);
      assert.ok(
        (fs.statSync(binPath).mode & 0o111) !== 0,
        `${packageInfo.name} installed non-executable ${binName}`,
      );
    }
  }
}

function installPackedPackages(tempDir, packages) {
  const installDir = path.join(tempDir, "install");
  fs.mkdirSync(installDir, { recursive: true });
  fs.writeFileSync(
    path.join(installDir, "package.json"),
    JSON.stringify({ name: "vize-release-install-smoke", private: true }, null, 2),
  );

  run(
    process.env.NPM_BIN || "npm",
    [
      "install",
      "--ignore-scripts",
      "--package-lock=false",
      "--no-audit",
      "--fund=false",
      "--legacy-peer-deps",
      ...packages.map((pkg) => pkg.tarball),
    ],
    { cwd: installDir },
  );

  const nodeModules = path.join(installDir, "node_modules");
  for (const packageInfo of packages) {
    assertInstalledPackage(nodeModules, packageInfo);
  }

  return installDir;
}

function installRuntimePeerDependencies(installDir) {
  run(
    process.env.NPM_BIN || "npm",
    [
      "install",
      "--ignore-scripts",
      "--package-lock=false",
      "--no-audit",
      "--fund=false",
      "--legacy-peer-deps",
      "@typescript/native-preview@7.0.0-dev.20260514.1",
      "typescript@6.0.3",
      "vite@npm:@voidzero-dev/vite-plus-core@0.1.21",
      "vue@3.5.34",
    ],
    { cwd: installDir },
  );
}

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

function hasPackage(packages, name) {
  return packages.some((pkg) => pkg.name === name);
}

function runRuntimeChecks(installDir, packages) {
  installRuntimePeerDependencies(installDir);
  writeRuntimeSmokeProject(installDir);

  if (hasPackage(packages, "@vizejs/native")) {
    run(
      process.execPath,
      [
        "-e",
        [
          'const native = require("@vizejs/native");',
          "const result = native.compileSfc(",
          '  \'<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>\',',
          "  { filename: 'Smoke.vue', isTs: true },",
          ");",
          "if (!result || result.errors.length > 0 || typeof result.code !== 'string' || result.code.length === 0) {",
          "  throw new Error('compileSfc runtime smoke failed');",
          "}",
        ].join("\n"),
      ],
      { cwd: installDir },
    );
    console.log("runtime: @vizejs/native compileSfc");
  }

  if (hasPackage(packages, "vize")) {
    run(
      process.env.NPM_BIN || "npm",
      ["exec", "--", "vize", "lint", "src/App.vue", "--format", "json", "--quiet", "--no-config"],
      { cwd: installDir },
    );
    console.log("runtime: vize lint");
  }

  if (hasPackage(packages, "@vizejs/vite-plugin")) {
    run(process.env.NPM_BIN || "npm", ["exec", "--", "vite", "build"], { cwd: installDir });
    console.log("runtime: @vizejs/vite-plugin vite build");
  }
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-"));
  const packDir = path.join(tempDir, "packs");
  fs.mkdirSync(packDir, { recursive: true });

  try {
    const packages = [];
    for (const packageDir of options.packageDirs) {
      if (!fs.existsSync(path.join(packageDir, "package.json"))) {
        throw new Error(`${packageDir} does not contain package.json`);
      }

      if (options.prepareManifests) {
        preparePublishManifest(packageDir);
      }

      const packageJson = readPackageJson(packageDir);
      assertNoWorkspaceProtocols(packageDir, packageJson);
      const compatible = isCompatibleWithCurrentRunner(packageJson);
      // Per-platform native sub-packages (those declaring os/cpu) only ship a
      // platform-specific .node binary that napi emits at the workspace root,
      // not into the per-platform npm subdir on a single-host smoke runner.
      // Asserting their entrypoint existence here would incorrectly red-light
      // the smoke matrix even for the host platform. The umbrella package and
      // every non-platform-specific package still get the check.
      const isPlatformSpecificSubPackage =
        Array.isArray(packageJson.os) || Array.isArray(packageJson.cpu);
      if (!isPlatformSpecificSubPackage) {
        assertPublishEntrypointsExist(packageDir, packageJson);
      }

      const tarball = packPackage(packageDir, packDir);
      packages.push({
        compatible,
        name: packageJson.name,
        packageDir,
        tarball,
        version: packageJson.version,
      });

      const installState = compatible ? "install" : "pack-only";
      console.log(`${installState}: ${packageJson.name}@${packageJson.version}`);
    }

    const installable = packages.filter((pkg) => pkg.compatible);
    assert.ok(installable.length > 0, "no package tarballs are compatible with this runner");
    const installDir = installPackedPackages(tempDir, installable);

    if (options.runtimeChecks) {
      runRuntimeChecks(installDir, installable);
    }

    console.log(`smoked ${installable.length}/${packages.length} package tarballs`);
    if (options.keepTemp) {
      console.log(`kept ${tempDir}`);
    } else {
      fs.rmSync(tempDir, { force: true, recursive: true });
    }
  } catch (error) {
    if (options.keepTemp) {
      console.error(`kept ${tempDir}`);
    } else {
      fs.rmSync(tempDir, { force: true, recursive: true });
    }
    throw error;
  }
}

main();
