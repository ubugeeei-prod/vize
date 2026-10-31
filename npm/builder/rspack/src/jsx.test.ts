import { test } from "node:test";
import fs from "node:fs";
import path from "node:path";
import { rspack } from "@rspack/core";
import { compileJsxModule } from "./shared/compiler.ts";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.ts";
import {
  normalizeSnapshot,
  packageLoaderAliases,
  prepareOutputDir,
  resolveFixturePath,
} from "./test/helpers.ts";

const ORIGINAL_PRIVATE_VAPOR_FIXTURE = `// Standalone Vapor renderers currently support roots without captured bindings.
const _App = () => (
  <div class="jsx-vapor-app">
    <p>hello vapor jsx</p>
    <span>{1 + 1}</span>
  </div>
);
`;

function runCompiler(compiler: ReturnType<typeof rspack>) {
  return new Promise<NonNullable<Parameters<Parameters<typeof compiler.run>[0]>[1]>>(
    (resolve, reject) => {
      compiler.run((error, stats) => {
        compiler.close((closeError) => {
          if (error || closeError) {
            reject(error ?? closeError);
            return;
          }
          if (!stats) {
            reject(new Error("Rspack did not return stats"));
            return;
          }
          resolve(stats);
        });
      });
    },
  );
}

/** Build a fixture routing `.jsx`/`.tsx` through the dist JSX loader. */
function createJsxCompiler(
  fixtureName: string,
  outputName: string,
  vapor: boolean,
): ReturnType<typeof rspack> {
  return rspack({
    mode: "development",
    devtool: false,
    context: resolveFixturePath(fixtureName, "."),
    entry: {
      main: resolveFixturePath(fixtureName, "entry.ts"),
    },
    output: {
      path: prepareOutputDir(outputName),
      filename: "bundle.js",
      clean: true,
    },
    externals: {
      vue: "vue",
    },
    infrastructureLogging: {
      level: "error",
    },
    resolve: {
      extensions: ["...", ".ts", ".js", ".jsx", ".tsx"],
    },
    resolveLoader: {
      alias: packageLoaderAliases,
    },
    module: {
      rules: [
        {
          test: /\.ts$/,
          loader: "builtin:swc-loader",
          options: { jsc: { parser: { syntax: "typescript" } } },
        },
        // Route `.jsx`/`.tsx` Vue components through the Vize JSX loader.
        {
          test: /\.[jt]sx$/,
          use: [
            {
              loader: "@vizejs/rspack-plugin/jsx-loader",
              options: { vapor },
            },
          ],
        },
      ],
    },
    plugins: [new VizePlugin({ vapor })],
  });
}

function bundleSource(stats: Awaited<ReturnType<typeof runCompiler>>): string {
  return Object.values(stats.compilation.assets)
    .map((asset) => asset.source().toString())
    .join("\n");
}

void test("jsx: a .jsx Vue component compiles to VDOM render code", async (t) => {
  const compiler = createJsxCompiler("jsx-vdom", "jsx-vdom", false);
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  t.assert.snapshot(normalizeSnapshot(bundleSource(stats)));
});

void test("jsx: a .jsx <style scoped> block emits scope-rewritten CSS into the bundle", async (t) => {
  // The JSX loader emits the component's `<style scoped>` CSS through the same
  // inline-style injection the integrations use for plain SFC CSS, so the
  // rewritten CSS (scope id baked into the selector) lands in the bundle and the
  // `data-v-<hash>` attribute is injected at runtime (#1495, #1533).
  const compiler = createJsxCompiler("jsx-scoped", "jsx-scoped", false);
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  t.assert.snapshot(normalizeSnapshot(bundleSource(stats)));
});

void test("jsx: a vapor .jsx component compiles to template render code", async (t) => {
  const compiler = createJsxCompiler("jsx-vapor", "jsx-vapor", true);
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  t.assert.equal(stats.hasWarnings(), false);
  const filename = resolveFixturePath("jsx-vapor", "App.jsx");
  const source = fs.readFileSync(filename, "utf8");
  t.assert.equal(
    source,
    ORIGINAL_PRIVATE_VAPOR_FIXTURE + "\nexport default _App;\n",
    "the new exported fixture adds exactly one authored export to the preserved private input",
  );
  const entry = fs.readFileSync(resolveFixturePath("jsx-vapor", "entry.ts"), "utf8");
  t.assert.equal(
    entry,
    '// Consume the authored component export through the JSX loader.\nimport App from "./App.jsx";\n\nexport default App;\n',
  );
  const reference = compileJsxModule(filename, source, { vapor: true, sourceMap: false });
  t.assert.deepEqual(reference.warnings, []);
  t.assert.equal(reference.map, null);
  const module = Array.from(stats.compilation.modules).find(
    (module) => "resource" in module && module.resource === filename,
  );
  t.assert.ok(module, "the actual source fixture has one compiled Rspack module");
  t.assert.equal(
    module.originalSource()?.source().toString(),
    reference.code,
    "the full dist-loader module equals the source native adapter reference",
  );
  const info = stats.toJson({ all: false, modules: true, providedExports: true });
  const exports = info.modules?.filter((module) => module.name === "./App.jsx");
  t.assert.equal(exports?.length, 1);
  t.assert.deepEqual(
    exports?.[0].providedExports,
    ["default"],
    "the bundle exposes the authored component export",
  );
  const captureDirectory =
    process.env.VIZE_RSPACK_JSX_CAPTURE ?? stats.compilation.outputOptions.path;
  if (!captureDirectory) throw new Error("Rspack must expose its real compiled output directory");
  fs.mkdirSync(captureDirectory, { recursive: true });
  fs.writeFileSync(
    path.join(captureDirectory, "authored-vapor-bundle.json"),
    JSON.stringify(
      {
        source,
        entry,
        reference,
        loadedModule: module.originalSource()?.source().toString(),
        stats: info,
        bundle: bundleSource(stats),
      },
      null,
      2,
    ) + "\n",
  );
  t.assert.snapshot(normalizeSnapshot(bundleSource(stats)));
});
