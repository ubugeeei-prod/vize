import assert from "node:assert/strict";
import test from "node:test";

import { setupNuxtLintChecker } from "./setup.ts";
import type { NuxtLintConfigGeneration } from "../generation.ts";

const generation: NuxtLintConfigGeneration = {
  configFile: "/project/.nuxt/oxlint.config.json",
  regenerate: async () => false,
};

function nuxt(
  overrides: Partial<{
    buildDir: string;
    builder: string;
    dev: boolean;
    rootDir: string;
    srcDir: string;
  }> = {},
) {
  return {
    options: {
      buildDir: "/project/.nuxt",
      builder: "@nuxt/vite-builder",
      dev: true,
      rootDir: "/project",
      srcDir: "/project/app",
      ...overrides,
    },
  };
}

void test("setup is checker-opt-in and strictly dev-server-only", async () => {
  let registrations = 0;
  const dependencies = {
    addVitePlugin: async () => {
      registrations += 1;
    },
  };
  assert.equal(await setupNuxtLintChecker(false, nuxt(), generation, dependencies), undefined);
  assert.equal(
    await setupNuxtLintChecker(true, nuxt({ dev: false }), generation, dependencies),
    undefined,
  );
  assert.equal(registrations, 0);
});

void test("setup connects the generated config seam to the Vite addon", async () => {
  let registered: unknown;
  const result = await setupNuxtLintChecker(true, nuxt(), generation, {
    addVitePlugin: async (plugin) => {
      registered = plugin;
    },
  });

  assert.equal(result?.builder, "vite");
  assert.equal((registered as { apply: string }).apply, "serve");
  assert.equal(result?.configFile, generation.configFile);
  assert.deepEqual(result?.options, {
    cache: true,
    emitError: true,
    emitWarning: true,
    exclude: ["**/node_modules/**", "/project/.nuxt"],
    fix: false,
    formatter: "stylish",
    include: ["/project/app/**/*.{js,jsx,ts,tsx,vue}"],
    lintOnStart: true,
  });
});

void test("setup connects every explicit option to the webpack addon", async () => {
  let registered: unknown;
  const result = await setupNuxtLintChecker(
    {
      cache: false,
      emitError: false,
      emitWarning: true,
      exclude: ["vendor/**"],
      fix: true,
      formatter: "unix",
      include: ["src/**/*.vue"],
      lintOnStart: false,
    },
    nuxt({ builder: "@nuxt/webpack-builder" }),
    generation,
    {
      addWebpackPlugin: async (plugin) => {
        registered = plugin;
      },
    },
  );

  assert.equal(result?.builder, "webpack");
  assert.equal(typeof (registered as { apply: unknown }).apply, "function");
  assert.deepEqual(result?.options, {
    cache: false,
    emitError: false,
    emitWarning: true,
    exclude: ["vendor/**"],
    fix: true,
    formatter: "unix",
    include: ["src/**/*.vue"],
    lintOnStart: false,
  });
});

void test("setup rejects a checker without the generated artifact", async () => {
  await assert.rejects(
    setupNuxtLintChecker(true, nuxt(), undefined),
    /requires lint config generation/u,
  );
});

void test("setup reports an unsupported dev builder without partial registration", async () => {
  const warnings: string[] = [];
  const result = await setupNuxtLintChecker(true, nuxt({ builder: "custom-builder" }), generation, {
    warn: (message) => warnings.push(message),
  });
  assert.equal(result?.builder, "unsupported");
  assert.deepEqual(warnings, [
    "Unsupported Nuxt builder custom-builder; Vize lint checker is disabled.",
  ]);
});
