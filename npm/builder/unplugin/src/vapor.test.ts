import { test } from "node:test";
import { vizeUnplugin } from "./unplugin.ts";
import { packageRoot } from "./test/helpers.ts";

interface TransformResult {
  code: string;
  map?: unknown;
}

function createPlugin(vapor: boolean) {
  return vizeUnplugin.raw(
    {
      isProduction: true,
      root: packageRoot,
      vapor,
    },
    {
      framework: "rollup",
    },
  );
}

async function runTransform(
  vapor: boolean,
  source: string,
  id: string,
): Promise<{ result: TransformResult | null; warnings: string[] }> {
  const plugin = createPlugin(vapor);
  const warnings: string[] = [];
  const transform = plugin.transform;
  if (typeof transform !== "function") {
    throw new Error("plugin.transform is not a function");
  }
  const result = (await transform.call(
    {
      warn(message: string) {
        warnings.push(message);
      },
    } as never,
    source,
    id,
  )) as TransformResult | null;
  return { result, warnings };
}

const VAPOR_SFC =
  "<template><div>{{ n }}</div></template>\n" + "<script setup>\nconst n = 1\n</script>";

void test("vapor:true flows through the transform and emits non-empty code without warnings", async (t) => {
  const { result, warnings } = await runTransform(true, VAPOR_SFC, "/proj/Vapor.vue");

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.equal(typeof result.code, "string");
  t.assert.ok(result.code.length > 0, "emitted code is non-empty");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");

  // Concrete vapor markers observed in the compiled output: the vapor runtime
  // entry helper and the static template helper.
  t.assert.match(
    result.code,
    /defineVaporComponent/,
    "vapor output imports the vapor component helper",
  );
  t.assert.match(result.code, /_template\(/, "vapor output uses the static template helper");
});

void test("vapor codegen differs from vdom codegen for the same SFC", async (t) => {
  const vapor = await runTransform(true, VAPOR_SFC, "/proj/Vapor.vue");
  const vdom = await runTransform(false, VAPOR_SFC, "/proj/Vapor.vue");

  t.assert.ok(vapor.result && typeof vapor.result === "object");
  t.assert.ok(vdom.result && typeof vdom.result === "object");

  const vaporCode = vapor.result.code;
  const vdomCode = vdom.result.code;

  // The two distinct codegen backends must not produce identical output.
  t.assert.notEqual(vaporCode, vdomCode, "vapor and vdom outputs differ");

  // Vapor-specific marker present only in the vapor output.
  t.assert.match(vaporCode, /defineVaporComponent/, "vapor output has defineVaporComponent");
  t.assert.doesNotMatch(
    vdomCode,
    /defineVaporComponent/,
    "vdom output does not have defineVaporComponent",
  );

  // vdom-specific marker present only in the vdom output: the block-based
  // element factory from the virtual-DOM runtime.
  t.assert.match(vdomCode, /createElementBlock/, "vdom output uses createElementBlock");
  t.assert.doesNotMatch(
    vaporCode,
    /createElementBlock/,
    "vapor output does not use createElementBlock",
  );
});

void test('vapor SFC with <script setup lang="ts"> compiles and strips TS', async (t) => {
  const tsSource =
    "<template><div>{{ n }}</div></template>\n" +
    '<script setup lang="ts">\n' +
    "const n: number = 1\n" +
    "const label = (x: number): string => String(x)\n" +
    "</script>";

  const { result, warnings } = await runTransform(true, tsSource, "/proj/VaporTs.vue");

  t.assert.ok(result && typeof result === "object", "typed vapor SFC compiles");
  t.assert.ok(result.code.length > 0, "emitted code is non-empty");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");

  // Still a vapor build.
  t.assert.match(result.code, /defineVaporComponent/, "typed vapor output is vapor codegen");

  // TS type annotations are stripped from the emitted JS.
  t.assert.doesNotMatch(result.code, /:\s*number/, "type annotation on const is stripped");
  t.assert.doesNotMatch(result.code, /:\s*string/, "return type annotation is stripped");
  t.assert.doesNotMatch(result.code, /lang="ts"/, "lang attribute does not leak into output");

  // The runtime bindings survive the strip.
  t.assert.match(result.code, /const n = 1/, "value binding is preserved after stripping types");
  t.assert.match(result.code, /String\(x\)/, "arrow body is preserved after stripping types");
});
