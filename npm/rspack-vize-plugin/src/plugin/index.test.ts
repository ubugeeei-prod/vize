import { test } from "node:test";
import "./../test/setup.ts";
import { VizePlugin } from "./index.ts";

function createMockCompiler(existingDefinitions?: Record<string, unknown>) {
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
      plugins: existingDefinitions ? [{ definitions: existingDefinitions }] : [],
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
        warn() {},
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
