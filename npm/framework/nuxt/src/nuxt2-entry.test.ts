import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const NUXT2_SAFE_KIT_VERSION = "3.11.2";
const SUBPROCESS_TIMEOUT_MS = 120_000;

void test("packed Nuxt module uses a jiti-safe require entry and keeps the ESM entry", async () => {
  const packageRoot = new URL("..", import.meta.url);
  const packDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-pack-"));

  try {
    execFileSync("pnpm", ["pack", "--pack-destination", packDir], {
      cwd: packageRoot,
      stdio: "pipe",
      timeout: SUBPROCESS_TIMEOUT_MS,
    });

    const tarballs = fs.readdirSync(packDir).filter((name) => name.endsWith(".tgz"));
    assert.equal(tarballs.length, 1);

    const tarball = path.join(packDir, tarballs[0]);
    const packedPackageJson = JSON.parse(
      execFileSync("tar", ["-xOf", tarball, "package/package.json"], {
        encoding: "utf8",
        timeout: SUBPROCESS_TIMEOUT_MS,
      }),
    ) as {
      dependencies?: Record<string, string>;
      exports?: { "."?: { import?: string; require?: string } };
      main?: string;
      module?: string;
    };
    const packedKitVersion = packedPackageJson.dependencies?.["@nuxt/kit"];

    assert.equal(packedKitVersion, NUXT2_SAFE_KIT_VERSION);
    assert.equal(packedPackageJson.main, "./dist/nuxt2-entry.cjs");
    assert.equal(packedPackageJson.module, "./dist/index.mjs");
    assert.equal(packedPackageJson.exports?.["."]?.require, "./dist/nuxt2-entry.cjs");
    assert.equal(packedPackageJson.exports?.["."]?.import, "./dist/index.mjs");

    execFileSync("tar", ["-xf", tarball, "-C", packDir], { timeout: SUBPROCESS_TIMEOUT_MS });
    const fixtureRoot = packDir;
    const fixtureModules = path.join(fixtureRoot, "node_modules");
    const packageDir = path.join(packDir, "package");
    fs.mkdirSync(path.join(fixtureModules, "@vizejs"), { recursive: true });
    symlinkDirectory(packageDir, path.join(fixtureModules, "@vizejs", "nuxt"));
    writeModuleFixture(
      path.join(fixtureModules, "@vizejs", "nuxt-lint-config"),
      {
        name: "@vizejs/nuxt-lint-config",
        version: "0.0.0-test",
        type: "module",
        exports: "./index.mjs",
      },
      [
        "export function buildNuxtLintPlan() { return []; }",
        "export function collectNuxtLintDirs() { return []; }",
        "export function resolveNuxtLintFeatures() { return {}; }",
      ].join("\n"),
    );

    const viteLoadMarker = "__VIZE_NUXT_VITE_7_LOADED__";
    writeModuleFixture(
      path.join(fixtureModules, "vite"),
      {
        name: "vite",
        version: "7.3.1",
        type: "module",
        main: "./index.mjs",
        exports: {
          ".": {
            import: "./index.mjs",
            require: "./index.mjs",
            default: "./index.mjs",
          },
        },
      },
      [
        `globalThis[${JSON.stringify(viteLoadMarker)}] =`,
        `  (globalThis[${JSON.stringify(viteLoadMarker)}] ?? 0) + 1;`,
        "export const moduleUrl = import.meta.url;",
        "export function parseSync() { return {}; }",
      ].join("\n"),
    );

    const fixtureConfig = path.join(fixtureRoot, "nuxt.config.js");
    const fixtureRequire = createRequire(fixtureConfig);
    assert.equal(
      fs.realpathSync(fixtureRequire.resolve("@vizejs/nuxt")),
      fs.realpathSync(path.join(packageDir, "dist", "nuxt2-entry.cjs")),
    );

    const testRequire = createRequire(import.meta.url);
    const createJiti = testRequire("jiti-nuxt2") as (
      filename: string,
      options?: Record<string, unknown>,
    ) => (specifier: string) => unknown;
    const loadWithNuxt2Jiti = createJiti(fixtureConfig, {
      cache: false,
      requireCache: false,
    });
    const loaded = loadWithNuxt2Jiti("@vizejs/nuxt");
    const nuxtModule = (
      typeof loaded === "function" ? loaded : (loaded as { default?: unknown }).default
    ) as {
      defaults: Record<string, unknown>;
      getMeta(): unknown;
      getOptions(inlineOptions?: Record<string, unknown>, nuxt?: { options?: unknown }): unknown;
    } & ((...args: unknown[]) => Promise<void>);

    assert.equal(typeof nuxtModule, "function");
    assert.deepEqual(nuxtModule.getMeta(), { name: "@vizejs/nuxt", configKey: "vize" });
    assert.equal(nuxtModule.defaults.checker, false);
    assert.deepEqual(
      nuxtModule.getOptions(
        { lint: false, nuxtMusea: { route: { name: "inline" } } },
        { options: { vize: { musea: true, nuxtMusea: { route: { path: "/project" } } } } },
      ),
      {
        checker: false,
        lint: false,
        musea: true,
        nuxtMusea: { route: { name: "inline", path: "/project" } },
      },
    );
    assert.equal((globalThis as Record<string, unknown>)[viteLoadMarker], undefined);

    const hookNames: string[] = [];
    const nuxt = {
      _version: "2.17.3",
      options: {
        rootDir: fixtureRoot,
        builder: "webpack",
        build: { publicPath: "/_nuxt/" },
        router: { base: "/" },
        modules: [],
        buildDir: path.join(fixtureRoot, ".nuxt"),
        dev: false,
      },
      hook(name: string, callback: (...args: unknown[]) => unknown) {
        assert.equal(typeof callback, "function");
        hookNames.push(name);
      },
    };
    await Reflect.apply(nuxtModule, { nuxt }, [
      {
        compiler: false,
        lint: false,
        musea: false,
        compatibility: { nuxtVersion: 2, vueVersion: "2.7", hostCompiler: true },
      },
    ]);
    assert.deepEqual(hookNames, ["close", "builder:prepared", "build:templates"]);
    assert.equal((globalThis as Record<string, unknown>)[viteLoadMarker], 1);

    // The probe runs in a fresh child process, so the parent marker above does not carry over:
    // viteLoads: 1 asserts the ESM entry loads Vite eagerly at import time, without invoking the
    // module, in contrast with the CJS entry which loads no Vite until the handler runs.
    const esmProbe = JSON.parse(
      execFileSync(
        process.execPath,
        [
          "--input-type=module",
          "--eval",
          [
            'const loaded = await import("@vizejs/nuxt");',
            "process.stdout.write(JSON.stringify({",
            "  type: typeof loaded.default,",
            `  viteLoads: globalThis[${JSON.stringify(viteLoadMarker)}],`,
            "}));",
          ].join("\n"),
        ],
        { cwd: fixtureRoot, encoding: "utf8", timeout: SUBPROCESS_TIMEOUT_MS },
      ),
    ) as { type?: string; viteLoads?: number };
    assert.deepEqual(esmProbe, { type: "function", viteLoads: 1 });
    delete (globalThis as Record<string, unknown>)[viteLoadMarker];
  } finally {
    fs.rmSync(packDir, { recursive: true, force: true });
  }
});

function symlinkDirectory(target: string, link: string): void {
  fs.symlinkSync(target, link, process.platform === "win32" ? "junction" : "dir");
}

function writeModuleFixture(
  directory: string,
  packageJson: Record<string, unknown>,
  source: string,
): void {
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    path.join(directory, "package.json"),
    `${JSON.stringify(packageJson, null, 2)}\n`,
  );
  fs.writeFileSync(path.join(directory, "index.mjs"), `${source}\n`);
}
