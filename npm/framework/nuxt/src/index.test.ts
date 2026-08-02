import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  registerNuxtMuseaStaticPublicAsset,
  resolveNuxtMuseaStaticPublicAsset,
} from "./musea-static.ts";

const NUXT2_SAFE_KIT_VERSION = "3.11.2";

void test("Nuxt module entry avoids loader-unsafe syntax and static kit imports", () => {
  const fixtures = [
    ["src/index.ts", new URL("./index.ts", import.meta.url)],
    ["dist/index.mjs", new URL("../dist/index.mjs", import.meta.url)],
    ["dist/nuxt2-entry.cjs", new URL("../dist/nuxt2-entry.cjs", import.meta.url)],
    ["src/resolver.ts", new URL("./resolver.ts", import.meta.url)],
  ] as const;

  const offsetsByFile = fixtures.map(([name, url]) => {
    const source = fs.readFileSync(url, "utf8");
    assert.doesNotMatch(source, /from\s+["']@nuxt\/kit["']/);
    return [name, importMetaOffsets(source)];
  });

  assert.deepEqual(offsetsByFile, [
    ["src/index.ts", []],
    ["dist/index.mjs", []],
    ["dist/nuxt2-entry.cjs", []],
    ["src/resolver.ts", []],
  ]);
});

void test("Nuxt module entry runs in a Nuxt 2 webpack-style context", async () => {
  const { default: nuxtModule } = await import(new URL("../dist/index.mjs", import.meta.url).href);
  const hookNames: string[] = [];
  const nuxt: {
    _version: string;
    options: Record<string, unknown> & {
      rootDir: string;
      builder: string;
      build: { publicPath: string };
      router: { base: string };
      modules: unknown[];
      buildDir: string;
      dev: boolean;
    };
    hook(name: string, callback: (...args: unknown[]) => unknown): void;
  } = {
    _version: "2.17.3",
    options: {
      rootDir: process.cwd(),
      builder: "webpack",
      build: { publicPath: "/_nuxt/" },
      router: { base: "/docs/" },
      modules: [],
      buildDir: ".nuxt",
      dev: false,
    },
    hook(name: string, callback: (...args: unknown[]) => unknown) {
      assert.equal(typeof callback, "function");
      hookNames.push(name);
    },
  };

  await nuxtModule(
    {
      compiler: false,
      lint: false,
      musea: false,
      compatibility: { nuxtVersion: 2, vueVersion: 2 },
    },
    nuxt,
  );

  assert.deepEqual(await nuxtModule.getMeta(), {
    name: "@vizejs/nuxt",
    configKey: "vize",
  });
  assert.equal(nuxtModule.defaults.checker, false);
  assert.deepEqual(
    {
      hookNames,
      requiredModules: nuxt.options._requiredModules,
      vite: nuxt.options.vite,
    },
    {
      hookNames: ["close", "builder:prepared", "build:templates"],
      requiredModules: { "@vizejs/nuxt": true },
      vite: undefined,
    },
  );
});

void test("Nuxt 2 host-compiler compatibility skips Vite plugin loading", async () => {
  const { default: nuxtModule } = await import(new URL("../dist/index.mjs", import.meta.url).href);
  const hookNames: string[] = [];
  const nuxt = {
    _version: "2.17.3",
    options: {
      rootDir: process.cwd(),
      builder: "webpack",
      build: { publicPath: "/_nuxt/" },
      router: { base: "/" },
      modules: [],
      buildDir: ".nuxt",
      dev: false,
      vite: {},
    },
    hook(name: string, callback: (...args: unknown[]) => unknown) {
      assert.equal(typeof callback, "function");
      hookNames.push(name);
    },
  };

  await nuxtModule(
    {
      compiler: true,
      lint: false,
      musea: false,
      bridge: true,
      compatibility: {
        nuxtVersion: 2,
        vueVersion: 2,
        hostCompiler: true,
        webpackVersion: 4,
      },
    },
    nuxt,
  );

  assert.deepEqual(hookNames, ["close", "builder:prepared", "build:templates"]);
  assert.deepEqual(nuxt.options.vite, {});
});

void test("Nuxt Musea static public asset points at generated client output", () => {
  let nitroConfigHook: ((config: { publicAssets?: unknown[] }) => unknown) | undefined;
  registerNuxtMuseaStaticPublicAsset(
    {
      options: { rootDir: "/project", buildDir: ".nuxt" },
      hook(name, callback) {
        assert.equal(name, "nitro:config");
        nitroConfigHook = callback;
      },
    },
    "/docs/musea/",
  );

  const nitroConfig = { publicAssets: [{ dir: "/existing", baseURL: "/existing" }] };
  nitroConfigHook?.(nitroConfig);

  assert.deepEqual(nitroConfig.publicAssets, [
    { dir: "/existing", baseURL: "/existing" },
    {
      dir: path.join("/project", ".nuxt", "dist", "client", "docs/musea"),
      baseURL: "/docs/musea",
    },
  ]);

  assert.deepEqual(resolveNuxtMuseaStaticPublicAsset("/project", ".nuxt", "/docs/musea/"), {
    dir: path.join("/project", ".nuxt", "dist", "client", "docs/musea"),
    baseURL: "/docs/musea",
  });
});

void test("Nuxt Musea static public asset preserves root base path", () => {
  assert.deepEqual(resolveNuxtMuseaStaticPublicAsset("/project", "/tmp/.nuxt", "/"), {
    dir: path.join("/tmp/.nuxt", "dist", "client"),
    baseURL: "/",
  });
});

void test("packed Nuxt module uses a jiti-safe require entry and keeps the ESM entry", async () => {
  const packageRoot = new URL("..", import.meta.url);
  const packDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-pack-"));

  try {
    execFileSync("pnpm", ["pack", "--pack-destination", packDir], {
      cwd: packageRoot,
      stdio: "pipe",
    });

    const tarballs = fs.readdirSync(packDir).filter((name) => name.endsWith(".tgz"));
    assert.equal(tarballs.length, 1);

    const tarball = path.join(packDir, tarballs[0]);
    const packedPackageJson = JSON.parse(
      execFileSync("tar", ["-xOf", tarball, "package/package.json"], {
        encoding: "utf8",
      }),
    ) as {
      dependencies?: Record<string, string>;
      exports?: { "."?: { import?: string; require?: string } };
      main?: string;
      module?: string;
    };
    const packedKitVersion = packedPackageJson.dependencies?.["@nuxt/kit"];

    assert.equal(packedKitVersion, NUXT2_SAFE_KIT_VERSION);
    assert.ok(
      !packedKitVersion?.startsWith("4."),
      "Nuxt 2 must not load @nuxt/kit 4.x through @vizejs/nuxt",
    );
    assert.equal(packedPackageJson.main, "./dist/nuxt2-entry.cjs");
    assert.equal(packedPackageJson.module, "./dist/index.mjs");
    assert.equal(packedPackageJson.exports?.["."]?.require, "./dist/nuxt2-entry.cjs");
    assert.equal(packedPackageJson.exports?.["."]?.import, "./dist/index.mjs");

    execFileSync("tar", ["-xf", tarball, "-C", packDir]);
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
        { cwd: fixtureRoot, encoding: "utf8" },
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

function importMetaOffsets(source: string): string[] {
  const offsets: string[] = [];
  let index = 0;
  let line = 1;
  let column = 1;
  let state: "code" | "line-comment" | "block-comment" | "single" | "double" | "template" = "code";
  let escaped = false;

  while (index < source.length) {
    const char = source[index];
    const next = source[index + 1];
    const currentLine = line;
    const currentColumn = column;

    if (state === "code") {
      if (char === "/" && next === "/") {
        advance(2);
        state = "line-comment";
        continue;
      }
      if (char === "/" && next === "*") {
        advance(2);
        state = "block-comment";
        continue;
      }
      if (char === "'") {
        advance(1);
        escaped = false;
        state = "single";
        continue;
      }
      if (char === '"') {
        advance(1);
        escaped = false;
        state = "double";
        continue;
      }
      if (char === "`") {
        advance(1);
        escaped = false;
        state = "template";
        continue;
      }
      if (
        source.startsWith("import.meta", index) &&
        isIdentifierBoundary(source[index - 1]) &&
        isIdentifierBoundary(source[index + "import.meta".length])
      ) {
        offsets.push(`${currentLine}:${currentColumn}`);
      }
      advance(1);
      continue;
    }

    if (state === "line-comment") {
      advance(1);
      if (char === "\n") {
        state = "code";
      }
      continue;
    }

    if (state === "block-comment") {
      if (char === "*" && next === "/") {
        advance(2);
        state = "code";
        continue;
      }
      advance(1);
      continue;
    }

    if (escaped) {
      advance(1);
      escaped = false;
      continue;
    }

    if (char === "\\") {
      advance(1);
      escaped = true;
      continue;
    }

    if (
      (state === "single" && char === "'") ||
      (state === "double" && char === '"') ||
      (state === "template" && char === "`")
    ) {
      advance(1);
      state = "code";
      continue;
    }

    advance(1);
  }

  return offsets;

  function advance(count: number) {
    for (let i = 0; i < count; i++) {
      const consumed = source[index];
      index++;
      if (consumed === "\n") {
        line++;
        column = 1;
      } else {
        column++;
      }
    }
  }
}

function isIdentifierBoundary(char: string | undefined): boolean {
  return char === undefined || !/[A-Za-z0-9_$]/.test(char);
}
