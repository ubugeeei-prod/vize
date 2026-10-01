import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import type { HmrContext, ResolvedConfig } from "vite";
import { vize } from "./index.ts";
import type { ResolvedVizeConfig } from "../types.ts";
import { compilerConfigForFile, createFileCompilerOptions } from "./compiler-scopes.ts";
import { mergeCompilerOptions } from "./compiler-config.ts";
import { compileFile } from "../compiler.ts";
import { compileAll } from "./precompile-run.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import { toVirtualId } from "../virtual.ts";
import { loadHook } from "./load.ts";
import { getCompileOptionsForRequest } from "./state.ts";
import { makeProject, makeState } from "../test/precompile-cache-project.ts";

void test("compiler scopes retain ordered glob, base, ignores, and explicit option precedence", () => {
  const root = path.resolve("scope-test");
  const config: ResolvedVizeConfig = {
    compiler: { whitespace: "preserve", sourceMap: false },
    entries: [
      { files: ["src/**/*.vue", "!src/generated/**"], compiler: { whitespace: "condense" } },
      {
        basePath: "src\\condensed",
        files: ["**/*.vue"],
        ignores: ["skip/**", "!skip/keep.vue"],
        compiler: { sourceMap: true },
      },
      { files: ["src/{override,chosen}/**"], compiler: { whitespace: "vue2-line-breaks" } },
      { files: [], compiler: { sourceMap: true } },
    ],
  };
  assert.deepEqual(
    [
      "other.vue",
      "src/Foo.vue",
      "src/generated/G.vue",
      "src/condensed/Bar.vue",
      "src/condensed/skip/A.vue",
      "src/condensed/skip/keep.vue",
      "src/chosen/Z.vue",
    ].map((file) => compilerConfigForFile(config, root, path.join(root, file))),
    [
      { whitespace: "preserve", sourceMap: false },
      { whitespace: "condense", sourceMap: false, compatibility: {} },
      { whitespace: "preserve", sourceMap: false },
      { whitespace: "condense", sourceMap: true, compatibility: {} },
      { whitespace: "condense", sourceMap: false, compatibility: {} },
      { whitespace: "condense", sourceMap: true, compatibility: {} },
      { whitespace: "vue2-line-breaks", sourceMap: false, compatibility: {} },
    ],
  );
  const explicit = createFileCompilerOptions(config, root, {
    whitespace: "preserve",
    sourceMap: false,
  })!;
  assert.deepEqual(
    explicit(path.join(root, "src/condensed/Bar.vue")),
    mergeCompilerOptions({ whitespace: "preserve", sourceMap: false }, config),
  );
  assert.deepEqual(
    compilerConfigForFile(config, root, path.resolve(root, "../outside.vue")),
    config.compiler,
  );
});

void test("scope globs match hidden paths, literals, negative-only files, and absolute bases", () => {
  const root = path.resolve("glob-scope-test");
  const config: ResolvedVizeConfig = {
    compiler: { whitespace: "preserve" },
    entries: [
      { files: ["!skip/**"], compiler: { sourceMap: true } },
      { files: ["src/**/*.vue"], compiler: { whitespace: "condense" } },
      { files: [String.raw`src/literal\*.vue`], compiler: { vapor: true } },
      { basePath: path.join(root, "absolute"), compiler: { mode: "function" } },
    ],
  };
  assert.deepEqual(
    [
      "src/.hidden.vue",
      "src/literal*.vue",
      "src/literalOther.vue",
      "skip/A.vue",
      "absolute/A.vue",
    ].map((file) => compilerConfigForFile(config, root, path.join(root, file))),
    [
      { whitespace: "condense", sourceMap: true, compatibility: {} },
      { whitespace: "condense", sourceMap: true, vapor: true, compatibility: {} },
      { whitespace: "condense", sourceMap: true, compatibility: {} },
      { whitespace: "preserve" },
      { whitespace: "preserve", sourceMap: true, mode: "function", compatibility: {} },
    ],
  );
});

void test("scoped compiler output survives batches, cold caches, client/SSR loads, and HMR", async () => {
  const source = fs.readFileSync(
    new URL("../test/fixtures/scoped-compiler.vue", import.meta.url),
    "utf8",
  );
  for (const isProduction of [false, true]) {
    const { root, file } = makeProject(source);
    const scoped = path.join(root, "src/condensed/Bar.vue");
    fs.mkdirSync(path.dirname(scoped), { recursive: true });
    fs.writeFileSync(scoped, source);
    const config: ResolvedVizeConfig = {
      compiler: { whitespace: "preserve" },
      entries: [
        { basePath: "src/condensed", files: ["**/*.vue"], compiler: { whitespace: "condense" } },
      ],
    };
    const makeScopedState = (settings = config) => {
      const state = makeState(root, mergeCompilerOptions({}, settings));
      state.isProduction = isProduction;
      state.fileCompilerOptions = createFileCompilerOptions(settings, root, {});
      state.compilerScopeIdentity = settings.entries;
      return state;
    };
    const expectedCode = (
      filename: string,
      whitespace: "preserve" | "condense",
      ssr = false,
      input = source,
    ) =>
      compileFile(
        filename,
        new Map(),
        {
          sourceMap: !isProduction,
          ssr,
          vapor: false,
          customRenderer: false,
          mode: "module",
          templateSyntax: "standard",
          templateComments: true,
          styleTrim: true,
          runtimeModuleName: "vue",
          runtimeGlobalName: "Vue",
          vueVersion: 3,
          whitespace,
          ...(isProduction ? { isProd: true, ...(!ssr ? { inlineTemplate: true } : {}) } : {}),
        },
        input,
      ).code;
    const first = makeScopedState();
    await compileAll(first);
    assert.equal(first.cache.get(file)?.code, expectedCode(file, "preserve"));
    assert.equal(first.cache.get(scoped)?.code, expectedCode(scoped, "condense"));
    const cold = makeScopedState();
    await compileAll(cold);
    assert.equal(cold.cache.get(scoped)?.code, expectedCode(scoped, "condense"));
    const changed = makeScopedState({ ...config, entries: [] });
    await compileAll(changed);
    assert.equal(changed.cache.get(scoped)?.code, expectedCode(scoped, "preserve"));
    const onDemand = makeScopedState();
    loadHook(onDemand, toVirtualId(scoped), { ssr: false });
    loadHook(onDemand, toVirtualId(scoped, true), { ssr: true });
    assert.equal(onDemand.cache.get(scoped)?.code, expectedCode(scoped, "condense"));
    assert.equal(onDemand.ssrCache.get(scoped)?.code, expectedCode(scoped, "condense", true));
    const edited = source.replace("a</span>", "updated</span>");
    const ctx = {
      file: scoped,
      read: async () => edited,
      modules: [],
      server: {
        moduleGraph: {
          getModuleById: () => undefined,
          getModulesByFile: () => undefined,
          invalidateModule() {},
        },
      },
    } as unknown as HmrContext;
    await handleHotUpdateHook(onDemand, ctx);
    assert.equal(onDemand.cache.get(scoped)?.code, expectedCode(scoped, "condense", false, edited));
    assert.equal(onDemand.ssrCache.size, 0);
    assert.equal(getCompileOptionsForRequest(onDemand, true, scoped).whitespace, "condense");
  }
});

void test("public plugin config resolution applies compiler entries to production client and SSR loads", async () => {
  const source = fs.readFileSync(
    new URL("../test/fixtures/scoped-compiler.vue", import.meta.url),
    "utf8",
  );
  const { root, file } = makeProject(source);
  const scoped = path.join(root, "src/condensed/Bar.vue");
  fs.mkdirSync(path.dirname(scoped), { recursive: true });
  fs.writeFileSync(scoped, source);
  const resolved = {
    root,
    base: "/",
    mode: "production",
    command: "build",
    isProduction: true,
    build: { assetsDir: "assets", ssr: false, sourcemap: false },
    define: {},
    plugins: [],
    resolve: { alias: [] },
  } as unknown as ResolvedConfig;
  async function output(config: ResolvedVizeConfig, filename: string, ssr: boolean) {
    fs.writeFileSync(path.join(root, "vize.config.json"), JSON.stringify(config));
    const plugin = vize({ root }).find((candidate) => candidate.name === "vite-plugin-vize")!;
    const resolve =
      typeof plugin.configResolved === "function"
        ? plugin.configResolved
        : plugin.configResolved!.handler;
    await resolve.call({} as never, resolved);
    const load = typeof plugin.load === "function" ? plugin.load : plugin.load!.handler;
    const result = await load.call({} as never, toVirtualId(filename, ssr), { ssr });
    assert.ok(result);
    return result;
  }
  const mixed: ResolvedVizeConfig = {
    compiler: { whitespace: "preserve" },
    entries: [{ files: ["src/condensed/**/*.vue"], compiler: { whitespace: "condense" } }],
  };
  for (const ssr of [false, true]) {
    assert.deepEqual(
      await output(mixed, file, ssr),
      await output({ compiler: { whitespace: "preserve" }, entries: [] }, file, ssr),
    );
    assert.deepEqual(
      await output(mixed, scoped, ssr),
      await output({ compiler: { whitespace: "condense" }, entries: [] }, scoped, ssr),
    );
  }
});
