import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { preparePublishManifest } from "../../tools/npm/prepare-publish-manifest.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const smokeScript = path.join(root, "tools/npm/smoke-release-install.mjs");

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
    const realTemp = path.join(tempDir, "real-temp");
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
  const script = fs.readFileSync(smokeScript, "utf8");
  const runtimeScript = fs.readFileSync(
    path.join(root, "tools/npm/smoke-release-runtime.mjs"),
    "utf8",
  );
  const smokeSources = `${script}\n${runtimeScript}`;
  const initTypecheckScript = fs.readFileSync(
    path.join(root, "tools/npm/smoke-release-init-typecheck.mjs"),
    "utf8",
  );

  assert.match(script, /--runtime-checks/);
  assert.match(script, /--content-mapper-checks/);
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
  assert.match(smokeSources, /runInitTypecheckChecks\([\s\S]*RUNTIME_PEER_DEPENDENCIES\)/);
  assert.match(runtimeScript, /VIZE_TEST_CONTENT_MAPPER_TSGO/);
  assert.match(runtimeScript, /runInstalledContentMapperChecks/);
  assert.match(runtimeScript, /content mapper project with spaces/);
  assert.match(runtimeScript, /--runExternalCode/);
  assert.match(runtimeScript, /tsconfig\.emit\.json/);
  assert.match(initTypecheckScript, /`init-smoke-\$\{language\}`/);
  assert.match(initTypecheckScript, /\["typescript", "javascript"\]/);
  assert.match(initTypecheckScript, /"init"[\s\S]*"--typecheck"[\s\S]*"--no-install"/);
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
  const result = spawnSync(process.execPath, [smokeScript, ...args], {
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
