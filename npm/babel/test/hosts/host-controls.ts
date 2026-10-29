// Real host infrastructure controls only: this never loads the packed plugin or native compiler.
import assert from "node:assert/strict";
import { createRequire, registerHooks } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { hostSpecs, json } from "./environment.ts";
import { positiveFixtures } from "./fixtures.ts";
import { nodes, wholeError } from "./contracts.ts";
import type { BabelCore, WholeError } from "../host-types.ts";
import type { InputOptions, PluginAPI, PluginItem, PluginObject, PluginTarget } from "@babel/core";

interface HostControl {
  version: string;
  id: string;
  code: string | null;
  error: WholeError | null;
}

const here = path.dirname(fileURLToPath(import.meta.url));
const observations: HostControl[] = [];
const nativeGuardControls: { version: string; loader: string; nativeExecution: boolean }[] = [];
for (const spec of hostSpecs) {
  const root = path.join(here, "fixtures", "babel" + spec.major),
    requireHost = createRequire(path.join(root, "package.json"));
  const core: BabelCore = await import(pathToFileURL(requireHost.resolve("@babel/core")).href);
  assert.equal(core.version, spec.version);
  let guardAttempts = 0;
  const hook = registerHooks({
    resolve(specifier, context, next) {
      if (specifier === "@vizejs/native") {
        guardAttempts++;
        throw new Error("cold native guard observed " + specifier);
      }
      return next(specifier, context);
    },
  });
  try {
    assert.throws(() => requireHost("@vizejs/native"), {
      message: "cold native guard observed @vizejs/native",
    });
    assert.equal(guardAttempts, 1, "the actual require resolution reached this guard");
    nativeGuardControls.push({ version: core.version, loader: "require", nativeExecution: false });
    await assert.rejects(import("@vizejs/native"), {
      message: "cold native guard observed @vizejs/native",
    });
    assert.equal(guardAttempts, 2, "the actual import resolution reached this guard");
    nativeGuardControls.push({ version: core.version, loader: "import", nativeExecution: false });
  } finally {
    hook.deregister();
  }
  const load = async (name: string): Promise<PluginTarget> =>
    (await import(pathToFileURL(requireHost.resolve(name)).href)).default;
  const typescript = await load("@babel/plugin-transform-typescript");
  const modules = await load("@babel/plugin-transform-modules-commonjs");
  const decorators = await load("@babel/plugin-proposal-decorators");
  const properties = await load("@babel/plugin-transform-class-properties");
  for (const fixture of positiveFixtures.filter((f) => !f.error)) {
    const row: HostControl = { version: core.version, id: fixture.id, code: null, error: null };
    observations.push(row);
    try {
      const remove = (api: PluginAPI): PluginObject => ({
        pre(file) {
          assert.equal(typeof api.parseSync, "function");
          assert.equal(typeof api.transformFromAstSync, "function");
          for (const node of nodes(file.ast)) {
            if (
              node.type === "ArrowFunctionExpression" &&
              nodes(node.body).some((n) => n.type === "JSXElement")
            )
              node.body = core.types.numericLiteral(42);
          }
        },
      });
      const plugins: PluginItem[] = [remove];
      if (fixture.typescript) plugins.push([typescript, { isTSX: true }]);
      if (fixture.decorators) plugins.push([decorators, { version: "2023-11" }]);
      if (fixture.classProperties) plugins.push(properties);
      plugins.push(modules);
      const parserOpts: InputOptions["parserOpts"] = {
        plugins: fixture.typescript ? ["jsx", "typescript"] : ["jsx"],
      };
      const result = core.transformSync(fixture.source, {
        filename: path.join(root, fixture.id + ".tsx"),
        babelrc: false,
        configFile: false,
        ast: true,
        parserOpts,
        plugins,
      });
      assert(result && typeof result.code === "string");
      assert(!nodes(result.ast).some((n) => n.type === "JSXElement"));
      if (fixture.typescript) assert(!nodes(result.ast).some((n) => n.type.startsWith("TS")));
      if (fixture.decorators) assert(!nodes(result.ast).some((n) => n.type === "Decorator"));
      if (fixture.classProperties)
        assert(!nodes(result.ast).some((n) => n.type === "ClassProperty"));
      row.code = result.code;
    } catch (error) {
      row.error = wholeError(error);
    }
  }
}
const output = process.env.VIZE_BABEL_HOST_OUTPUT ?? path.join(here, ".artifacts/host-controls");
json(path.join(output, "host-controls.json"), {
  scope:
    "actual Babel host APIs/parser/ordinary plugins only; no packed-plugin or source-native credit",
  observations,
  nativeGuardControls,
});
assert(
  observations.every((row) => row.error === null),
  "actual host controls failed: inspect " + output,
);
process.stdout.write(
  JSON.stringify({
    scope: "host infrastructure only",
    actualHostControls: observations.length,
    nativeGuardControls: nativeGuardControls.length,
    packedOrNativeCredit: false,
  }) + "\n",
);
