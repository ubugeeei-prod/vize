import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { preparePublishManifest } from "../../../tools/support/compat/npm/prepare-publish-manifest.ts";
import { normalizeOxlintObservation } from "../../../tools/support/release/smoke_release_install/runtime.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const smokeCommand = path.join(root, "tools/commands/release/npm/smoke-release-install.ts");
const smokeModule = path.join(root, "tools/support/release/smoke_release_install/main.ts");

test("release install smoke packs and installs local package tarballs", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  try {
    const dependencyDir = writePackage(tempDir, "dependency", {
      name: "@vizejs/smoke-dependency",
      version: "1.0.0",
    });
    const packageDir = writePackage(tempDir, "package", {
      bin: {
        "smoke-bin": "cli.js",
      },
      dependencies: {
        "@vizejs/smoke-dependency": "1.0.0",
      },
      name: "@vizejs/smoke-package",
      version: "1.0.0",
    });
    fs.writeFileSync(path.join(packageDir, "cli.js"), "#!/usr/bin/env node\n");
    fs.chmodSync(path.join(packageDir, "cli.js"), 0o755);

    const result = runSmoke(packageDir, dependencyDir);

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /smoked 2\/2 package tarballs/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("release install smoke canonicalizes symlinked temp directories", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  try {
    const realTemp = path.join(tempDir, "real temp 新規");
    const linkedTemp = path.join(tempDir, "linked-temp");
    fs.mkdirSync(realTemp, { recursive: true });
    fs.symlinkSync(realTemp, linkedTemp, process.platform === "win32" ? "junction" : "dir");
    const packageDir = writePackage(tempDir, "package", {
      name: "@vizejs/smoke-package",
      version: "1.0.0",
    });

    const result = runSmokeArgs(["--keep-temp", packageDir], {
      env: { TEMP: linkedTemp, TMP: linkedTemp, TMPDIR: linkedTemp },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    const keptTemp = /^kept (.+)$/m.exec(result.stdout)?.[1];
    assert.ok(keptTemp, "release smoke did not report a kept temp directory");
    assert.equal(keptTemp, fs.realpathSync.native(keptTemp));
    const relativeToRealTemp = path.relative(fs.realpathSync.native(realTemp), keptTemp);
    assert.ok(relativeToRealTemp !== "");
    assert.ok(!relativeToRealTemp.startsWith(".."));
    assert.ok(!path.isAbsolute(relativeToRealTemp));
    fs.rmSync(keptTemp, { force: true, recursive: true });
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("release install smoke rejects unresolved workspace protocols", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  try {
    const packageDir = writePackage(tempDir, "package", {
      dependencies: {
        "@vizejs/smoke-dependency": "workspace:*",
      },
      name: "@vizejs/smoke-package",
      version: "1.0.0",
    });

    const result = runSmoke(packageDir);

    assert.notEqual(result.status, 0);
    assert.match(`${result.stderr}\n${result.stdout}`, /workspace:\*/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("release install smoke rejects a vize tarball with a stale native dependency", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  try {
    const nativeDir = writePackage(tempDir, "native", {
      name: "@vizejs/native",
      version: "1.2.3",
    });
    const vizeDir = writePackage(tempDir, "vize", {
      dependencies: {
        "@vizejs/native": "0.9.0",
      },
      name: "vize",
      version: "1.2.3",
    });

    const result = runSmoke(vizeDir, nativeDir);

    assert.notEqual(result.status, 0);
    assert.match(
      `${result.stderr}\n${result.stdout}`,
      /vize@1\.2\.3 must depend on @vizejs\/native@1\.2\.3, found 0\.9\.0/,
    );
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("release install smoke prepares workspace dependencies without MoonBit", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  const repoDir = path.join(tempDir, "repo");
  try {
    fs.mkdirSync(repoDir, { recursive: true });
    fs.writeFileSync(
      path.join(repoDir, "pnpm-workspace.yaml"),
      ["packages:", '  - "npm/*"', ""].join("\n"),
    );
    const dependencyDir = writePackage(repoDir, "npm/dependency", {
      name: "@vizejs/smoke-dependency",
      version: "1.2.3",
    });
    const packageDir = writePackage(repoDir, "npm/package", {
      dependencies: {
        "@vizejs/smoke-dependency": "workspace:*",
      },
      name: "@vizejs/smoke-package",
      version: "1.2.3",
    });

    const result = runSmokeArgs(["--prepare-manifests", packageDir, dependencyDir]);

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /Prepared npm publish manifest/);
    assert.match(result.stdout, /smoked 2\/2 package tarballs/);
    assert.deepEqual(readPackage(packageDir).dependencies, {
      "@vizejs/smoke-dependency": "1.2.3",
    });
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("prepare publish manifest rewrites workspace and catalog dependencies without MoonBit", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  const repoDir = path.join(tempDir, "repo");
  try {
    fs.mkdirSync(repoDir, { recursive: true });
    fs.writeFileSync(
      path.join(repoDir, "pnpm-workspace.yaml"),
      [
        "packages:",
        '  - "npm/*"',
        "",
        "catalogs:",
        "  native-binaries:",
        '    "@vizejs/native-darwin-x64": "0.57.0"',
        "  repo-tooling:",
        '    tinyglobby: "0.2.16"',
        "",
      ].join("\n"),
    );
    const nativeDir = writePackage(repoDir, "npm/native", {
      name: "@vizejs/native",
      optionalDependencies: {
        "@vizejs/native-darwin-x64": "catalog:native-binaries",
      },
      version: "0.58.0",
    });
    const pluginDir = writePackage(repoDir, "npm/builder/vite", {
      dependencies: {
        "@vizejs/native": "workspace:*",
        tinyglobby: "catalog:repo-tooling",
      },
      name: "@vizejs/vite-plugin",
      version: "0.58.0",
    });

    preparePublishManifest(nativeDir);
    preparePublishManifest(pluginDir);

    assert.deepEqual(readPackage(nativeDir).optionalDependencies, {
      "@vizejs/native-darwin-x64": "0.58.0",
    });
    assert.deepEqual(readPackage(pluginDir).dependencies, {
      "@vizejs/native": "0.58.0",
      tinyglobby: "0.2.16",
    });
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("release install smoke skips libc-incompatible tarballs", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  try {
    const compatibleDir = writePackage(tempDir, "compatible", {
      cpu: [process.arch],
      name: "@vizejs/smoke-compatible",
      os: [process.platform],
      version: "1.0.0",
    });
    const incompatibleDir = writePackage(tempDir, "incompatible", {
      cpu: [process.arch],
      libc: [oppositeLibc()],
      name: "@vizejs/smoke-incompatible",
      os: [process.platform],
      version: "1.0.0",
    });

    const result = runSmoke(compatibleDir, incompatibleDir);

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /install: @vizejs\/smoke-compatible@1\.0\.0/);
    assert.match(result.stdout, /pack-only: @vizejs\/smoke-incompatible@1\.0\.0/);
    assert.match(result.stdout, /smoked 1\/2 package tarballs/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("release install smoke can run runtime checks for Vize packages", () => {
  const script = fs.readFileSync(smokeModule, "utf8");
  const runtimeScript = fs.readFileSync(
    path.join(root, "tools/support/release/smoke_release_install/runtime.ts"),
    "utf8",
  );
  const smokeSources = `${script}\n${runtimeScript}`;
  const initTypecheckScript = fs.readFileSync(
    path.join(root, "tools/support/compat/npm/smoke-release-init-typecheck.mjs"),
    "utf8",
  );

  assert.match(script, /--runtime-checks/);
  assert.match(script, /--bun-runtime-checks/);
  assert.match(script, /--content-mapper-checks/);
  assert.match(script, /--bun-runtime-checks requires --runtime-checks/);
  assert.match(runtimeScript, /delete peers\["typescript-native"\]/);
  assert.match(script, /Bun runtime behavior differs from Node/);
  assert.match(script, /Bun and Node diagnostics\/generated output are identical/);
  // vize declares its Corsa runtime itself, so the fresh-install smoke must not
  // manually add the retired native-preview package as a project runtime peer.
  assert.doesNotMatch(smokeSources, /"@typescript\/native-preview":/);
  assert.match(smokeSources, /require\("@vizejs\/native"\)/);
  assert.match(smokeSources, /import\("@vizejs\/native"\)/);
  assert.match(smokeSources, /native\.compileSfc/);
  // Bins are resolved out of the installed tree and invoked through
  // `process.execPath` so that `npm exec` cannot re-resolve native optional
  // deps and drop them mid-run (npm/cli#4828).
  assert.match(smokeSources, /resolveInstalledBin\(installDir, "vize", "vize"\)/);
  assert.match(smokeSources, /vizeBin, "--version"/);
  assert.match(smokeSources, /"check"[\s\S]*"src\/App\.vue"/);
  assert.match(smokeSources, /"lint"[\s\S]*"src\/App\.vue"/);
  assert.match(smokeSources, /peers: RUNTIME_PEER_DEPENDENCIES/);
  assert.match(runtimeScript, /VIZE_TEST_CONTENT_MAPPER_TSGO/);
  assert.match(runtimeScript, /runInstalledContentMapperChecks/);
  assert.match(runtimeScript, /content mapper project with spaces/);
  assert.match(runtimeScript, /--runExternalCode/);
  assert.match(runtimeScript, /tsconfig\.emit\.json/);
  assert.match(initTypecheckScript, /`init-smoke-\$\{language\}`/);
  assert.match(initTypecheckScript, /\["typescript", "javascript"\]/);
  assert.match(initTypecheckScript, /devDependencies/);
  assert.match(
    initTypecheckScript,
    /"init"[\s\S]*"--yes"[\s\S]*"--typecheck"[\s\S]*"--no-install"/,
  );
  assert.doesNotMatch(initTypecheckScript, /"--language"/);
  assert.match(initTypecheckScript, /"run", "--silent", "vize:check"/);
  for (const diagnostic of [
    "standalone source",
    "SFC script",
    "SFC template",
    "component prop",
    "JSX consumer",
  ]) {
    assert.match(initTypecheckScript, new RegExp(diagnostic));
  }
  // vite is installed as upstream vite 8, one supported
  // `@vizejs/vite-plugin` peer range. Real vite exposes a `vite` bin entry, so
  // the smoke can use the same resolver as vize.
  assert.match(smokeSources, /resolveInstalledBin\(installDir, "vite", "vite"\)/);
  assert.match(smokeSources, /viteBin, "build"/);
  assert.match(smokeSources, /vite:\s*"\^8\.0\.0"/);
  assert.match(smokeSources, /runtime: @vizejs\/vite-plugin vite build/);
  // The combined install must request optional deps explicitly so that an
  // inherited `omit=optional` config does not silently drop platform-specific
  // native bindings (e.g. rolldown).
  assert.match(script, /"--include=optional"/);
});

test("Bun install and Node/Bun parity execute through the configured runtime", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bun-runtime-smoke-test-"));
  try {
    const log = path.join(tempDir, "runtime.jsonl");
    const realNpm = resolvePathCommand("npm");
    const npmStub = writeRuntimeStub(tempDir, "npm");
    const bunStub = writeRuntimeStub(tempDir, "bun");
    const nativeDir = writePackage(tempDir, "native", {
      name: "@vizejs/native",
      version: "1.0.0",
      exports: {
        ".": {
          import: "./index.js",
          require: "./index.js",
          types: "./index.d.ts",
        },
      },
    });
    fs.writeFileSync(
      path.join(nativeDir, "index.js"),
      `const runtime = process.env.VIZE_BUN_STUB_RUNTIME === "1" ? "bun" : "node";
module.exports.compileSfc = () => ({
  code: process.env.VIZE_BUN_STUB_DIFFER === "1" ? runtime : "identical",
  errors: [],
  warnings: [],
});
`,
    );
    const environment = {
      BUN_BIN: bunStub,
      NPM_BIN: npmStub,
      VIZE_RUNTIME_STUB_LOG: log,
      VIZE_RUNTIME_STUB_NPM: realNpm,
    };

    const matching = runSmokeArgs(["--runtime-checks", "--bun-runtime-checks", nativeDir], {
      env: environment,
    });
    assert.equal(matching.status, 0, `${matching.stderr}\n${matching.stdout}`.trim());
    assert.match(matching.stdout, /Bun and Node diagnostics\/generated output are identical/);
    const invocations = fs
      .readFileSync(log, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line) as { args: string[]; kind: string });
    assert.ok(
      invocations.some(
        ({ args, kind }) =>
          kind === "bun" && args[0] === "install" && args[1] === "--ignore-scripts",
      ),
      "Bun must install the packed graph without lifecycle scripts",
    );
    assert.ok(
      invocations.some(({ args, kind }) => kind === "bun" && args[0] === "-e"),
      "the packed native product must execute through the configured Bun runtime",
    );

    fs.writeFileSync(log, "");
    const differing = runSmokeArgs(["--runtime-checks", "--bun-runtime-checks", nativeDir], {
      env: { ...environment, VIZE_BUN_STUB_DIFFER: "1" },
    });
    assert.equal(differing.status, 1);
    assert.match(differing.stderr, /Bun runtime behavior differs from Node/);
    assert.match(differing.stderr, /\\"code\\":\\"node\\"/);
    assert.match(differing.stderr, /\\"code\\":\\"bun\\"/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("Oxlint parity excludes only per-invocation timing", () => {
  const report = {
    diagnostics: [
      {
        code: "vize(vue/no-v-html)",
        filename: "/packed/src/App.vue",
        labels: [{ span: { column: 1, length: 6, line: 1, offset: 0 } }],
        message: "unsafe html",
        severity: "warning",
      },
    ],
    number_of_files: 2,
    number_of_rules: 89,
    start_time: 12.5,
    threads_count: 4,
  };
  const first = normalizeOxlintObservation(JSON.stringify(report), "/packed");
  const second = normalizeOxlintObservation(
    JSON.stringify({ ...report, start_time: 0.000_001 }),
    "/packed",
  );

  assert.equal(first, second);
  assert.deepEqual(JSON.parse(first), {
    diagnostics: [
      {
        code: "vize(vue/no-v-html)",
        filename: "/VIZE_PROJECT/src/App.vue",
        labels: [{ span: { column: 1, length: 6, line: 1, offset: 0 } }],
        message: "unsafe html",
        severity: "warning",
      },
    ],
    number_of_files: 2,
    number_of_rules: 89,
    threads_count: 4,
  });
});

function writePackage(tempDir: string, dirName: string, manifest: Record<string, unknown>): string {
  const packageDir = path.join(tempDir, dirName);
  fs.mkdirSync(packageDir, { recursive: true });
  fs.writeFileSync(path.join(packageDir, "index.js"), "export const ok = true;\n");
  fs.writeFileSync(path.join(packageDir, "index.d.ts"), "export declare const ok: true;\n");
  fs.writeFileSync(
    path.join(packageDir, "package.json"),
    JSON.stringify(
      {
        exports: {
          ".": {
            import: "./index.js",
            types: "./index.d.ts",
          },
        },
        files: ["index.js", "index.d.ts", "cli.js"],
        main: "./index.js",
        types: "./index.d.ts",
        ...manifest,
      },
      null,
      2,
    ),
  );
  return packageDir;
}

function readPackage(packageDir: string): Record<string, unknown> {
  return JSON.parse(fs.readFileSync(path.join(packageDir, "package.json"), "utf8")) as Record<
    string,
    unknown
  >;
}

function oppositeLibc(): "glibc" | "musl" {
  if (process.platform !== "linux") return "glibc";
  const header = process.report?.getReport?.().header;
  return header != null && typeof header.glibcVersionRuntime === "string" ? "musl" : "glibc";
}

function runSmoke(...packageDirs: string[]): SmokeResult {
  return runSmokeArgs(packageDirs);
}

type SmokeOptions = {
  env?: NodeJS.ProcessEnv;
};

type SmokeResult = {
  status: number | null;
  stderr: string;
  stdout: string;
};

function runSmokeArgs(args: string[], options: SmokeOptions = {}): SmokeResult {
  const result = spawnSync(process.execPath, [smokeCommand, ...args], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, ...options.env },
  });

  if (result.error != null) {
    throw result.error;
  }

  return {
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

function resolvePathCommand(name: string): string {
  const extensions =
    process.platform === "win32" ? (process.env.PATHEXT ?? ".CMD;.EXE").split(";") : [""];
  for (const directory of (process.env.PATH ?? "").split(path.delimiter))
    for (const extension of extensions) {
      const candidate = path.join(directory, `${name}${extension.toLowerCase()}`);
      try {
        if (fs.statSync(candidate).isFile()) return fs.realpathSync.native(candidate);
      } catch {
        // Try the next PATH entry.
      }
    }
  throw new Error(`${name} is not available on PATH`);
}

function writeRuntimeStub(tempDir: string, kind: "bun" | "npm"): string {
  const script = path.join(tempDir, `${kind}-runtime-stub.cjs`);
  fs.writeFileSync(
    script,
    `#!${process.execPath}
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const kind = ${JSON.stringify(kind)};
const args = process.argv.slice(2);
fs.appendFileSync(process.env.VIZE_RUNTIME_STUB_LOG, JSON.stringify({ kind, args }) + "\\n");
const installing = args[0] === "install";
if (installing) {
  const manifestPath = path.join(process.cwd(), "package.json");
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  manifest.dependencies = Object.fromEntries(
    Object.entries(manifest.dependencies || {}).filter(([, value]) => String(value).startsWith("file:")),
  );
  fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\\n");
  args.splice(0, args.length, "install", "--ignore-scripts", "--package-lock=false", "--no-audit", "--fund=false", "--legacy-peer-deps", "--include=optional");
}
const command = kind === "npm" || installing ? process.env.VIZE_RUNTIME_STUB_NPM : process.execPath;
const env = kind === "bun" && !installing
  ? { ...process.env, VIZE_BUN_STUB_RUNTIME: "1" }
  : process.env;
const child = spawnSync(command, args, {
  cwd: process.cwd(),
  env,
  shell: process.platform === "win32" && /\\.(cmd|bat)$/i.test(command),
  stdio: "inherit",
});
if (child.error) throw child.error;
process.exit(child.status ?? 1);
`,
  );
  if (process.platform !== "win32") {
    fs.chmodSync(script, 0o755);
    return script;
  }
  const wrapper = path.join(tempDir, `${kind}-runtime-stub.cmd`);
  fs.writeFileSync(wrapper, `@echo off\r\n"${process.execPath}" "${script}" %*\r\n`);
  return wrapper;
}
