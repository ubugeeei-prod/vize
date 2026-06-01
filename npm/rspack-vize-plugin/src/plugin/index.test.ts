import { test } from "node:test";
import assert from "node:assert/strict";
import "./../test/setup.ts";
import { VizePlugin } from "./index.ts";

function createMockCompiler(
  existingDefinitions?: Record<string, unknown>,
  config: {
    experiments?: { css?: boolean };
    rules?: unknown[];
    warnings?: string[];
  } = {},
) {
  let capturedDefinitions: Record<string, string> | null = null;

  class DefinePluginMock {
    definitions: Record<string, string>;

    constructor(definitions: Record<string, string>) {
      this.definitions = definitions;
      capturedDefinitions = definitions;
    }

    apply() {}
  }

  const compiler = {
    options: {
      mode: "development",
      plugins: existingDefinitions
        ? [{ definitions: existingDefinitions }]
        : [],
      experiments: config.experiments,
      module: config.rules ? { rules: config.rules } : undefined,
    },
    webpack: {
      DefinePlugin: DefinePluginMock,
    },
    hooks: {
      watchRun: {
        tap() {},
      },
    },
    getInfrastructureLogger() {
      return {
        warn(message: string) {
          config.warnings?.push(message);
        },
        debug() {},
      };
    },
  };

  return {
    compiler,
    getCapturedDefinitions: () => capturedDefinitions,
  };
}

void test("injects the default Vue compile-time flags", (t) => {
  const { compiler, getCapturedDefinitions } = createMockCompiler();
  new VizePlugin().apply(compiler as never);

  t.assert.snapshot(JSON.stringify(getCapturedDefinitions(), null, 2));
});

void test("does not override Vue flags that are already defined", (t) => {
  const { compiler, getCapturedDefinitions } = createMockCompiler({
    __VUE_OPTIONS_API__: JSON.stringify(false),
  });

  new VizePlugin().apply(compiler as never);

  t.assert.snapshot(JSON.stringify(getCapturedDefinitions(), null, 2));
});

void test("does not warn when native CSS is explicit and legacy experiments.css is unavailable", () => {
  const warnings: string[] = [];
  const { compiler } = createMockCompiler(undefined, { warnings });

  new VizePlugin({ css: { native: true } }).apply(compiler as never);

  assert.deepEqual(warnings, []);
});

void test("warns when native CSS is explicit and legacy experiments.css is disabled", () => {
  const warnings: string[] = [];
  const { compiler } = createMockCompiler(undefined, {
    experiments: { css: false },
    warnings,
  });

  new VizePlugin({ css: { native: true } }).apply(compiler as never);

  assert.deepEqual(warnings, [
    "`css.native: true` is set but `experiments.css` is not enabled in rspack config.",
  ]);
});

void test("passes explicit native CSS mode to the auto-cloned vue loader", () => {
  const rules = [
    { test: /\.css$/, use: ["css-loader"] },
    { test: /\.vue$/, use: ["@vizejs/rspack-plugin/loader"] },
  ];
  const { compiler } = createMockCompiler(undefined, { rules });

  new VizePlugin({ css: { native: true } }).apply(compiler as never);

  const vueRule = rules[1] as Record<string, unknown>;
  const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;
  const mainBranch = oneOf[oneOf.length - 1];
  const use = mainBranch.use as Array<Record<string, unknown>>;

  assert.deepEqual(use[0], {
    loader: "@vizejs/rspack-plugin/loader",
    options: { css: { native: true } },
  });
});
