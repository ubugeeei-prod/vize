import assert from "node:assert/strict";
import fs from "node:fs";
import { registerHooks } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { activeNative, hash, json } from "./environment.mjs";
import { coldFixtures, positiveFixtures } from "./fixtures.mjs";
import {
  assertCompiled,
  assertUpstream,
  editJsx,
  hasJsx,
  removeJsx,
  upstreamMap,
  wholeError,
} from "./contracts.mjs";

const factory = (module) => (typeof module === "function" ? module : module.default);
const astPacket = (value) =>
  JSON.stringify(value, (_, item) => (typeof item === "bigint" ? { bigint: String(item) } : item));
// Syntax plugins may append parser entries; each actual host invocation owns its options.
const freshParserOptions = (options) => ({
  ...options,
  parserOpts: {
    ...options.parserOpts,
    plugins: options.parserOpts.plugins.map((plugin) =>
      Array.isArray(plugin) ? [...plugin] : plugin,
    ),
  },
});
function outputPayload(result) {
  // Babel also returns its input configuration. That contains the deliberately different
  // plugin list and hook functions; retain it in the raw report, compare every output field.
  const { options: _configuration, ...output } = result;
  return output;
}
function addonImages(requireHost) {
  return Object.keys(requireHost.cache)
    .filter((file) => file.endsWith(".node"))
    .map((file) => {
      const image = fs.realpathSync(file);
      return { path: image, sha256: hash(fs.readFileSync(image)) };
    });
}

export async function runHost(input, load) {
  const report = {
    schema: "vize.babel.packed-host",
    version: 1,
    source: input.source,
    host: input.host,
    format: input.format,
    phase: input.phase,
    tarballSha256: input.tarballSha256,
    scope:
      input.phase === "cold"
        ? "packed no-JSX cold host only; zero native source execution"
        : "active source-prepared addon + actual packed consumer",
    observations: [],
    resolutions: [],
    fatal: null,
  };
  let hook;
  try {
    assert.equal(addonImages(load.requireHost).length, 0, "the worker must begin without an addon");
    hook = registerHooks({
      resolve(specifier, context, next) {
        if (specifier.startsWith("@vizejs/native") || specifier.endsWith(".node")) {
          report.resolutions.push({ specifier, parentURL: context.parentURL });
          assert.notEqual(
            input.phase,
            "cold",
            "a no-JSX transform must never attempt native resolution",
          );
        }
        return next(specifier, context);
      },
    });
    const core = await load.core();
    assert.equal(core.version, input.host.version);
    const vize = factory(await load.vize());
    assert.equal(typeof vize, "function", "actual packed default factory");
    const typescript = factory(await load.typescript());
    const modules = factory(await load.modules());
    const decorators = factory(await load.decorators());
    const classProperties = factory(await load.classProperties());
    const react = factory(await load.react());
    const preparation = input.phase === "positive" ? await activeNative(input) : null;
    report.preparation = preparation;
    const cases = input.phase === "cold" ? coldFixtures : positiveFixtures;
    for (const fixture of cases) {
      const item = { id: fixture.id, input: fixture, result: null, error: null, assertion: null };
      report.observations.push(item);
      try {
        if (preparation)
          assert.deepEqual(
            await activeNative(input),
            preparation,
            "source preparation lease remains current",
          );
        const filename = path.join(
          input.host.root,
          "src",
          fixture.id + (fixture.filenameSuffix ?? (fixture.typescript ? ".tsx" : ".jsx")),
        );
        const parserOpts = { plugins: fixture.typescript ? ["jsx", "typescript"] : ["jsx"] };
        const options = {
          filename,
          babelrc: false,
          configFile: false,
          ast: true,
          sourceMaps: Boolean(fixture.upstream),
          comments: fixture.comments ?? true,
          parserOpts,
        };
        const prePlugin = () => ({
          name: "authored-host-pre-" + fixture.id,
          pre(file) {
            if (fixture.pre === "remove") removeJsx(core, file.ast);
            if (fixture.pre === "edit") {
              editJsx(file.ast, "pre-edited");
              file.metadata.vize = { predecessor: "preserved" };
              const extra = core.parseSync("export const preAdded = 17;", {
                babelrc: false,
                configFile: false,
              });
              file.path.pushContainer("body", extra.program.body);
            }
            if (fixture.pre === "create") {
              const extra = core.parseSync(
                "export const render = () => <p>created-before-native</p>;",
                { babelrc: false, configFile: false, parserOpts: { plugins: ["jsx"] } },
              );
              file.path.pushContainer("body", extra.program.body);
            }
          },
        });
        const latePlugin = () => ({
          name: "late-jsx-host-pre",
          pre(file) {
            const extra = core.parseSync("<aside />;", {
              babelrc: false,
              configFile: false,
              parserOpts: { plugins: ["jsx"] },
            });
            file.path.pushContainer("body", extra.program.body);
          },
        });
        const ordinary = () => ({
          name: "ordinary-host-number-edit",
          visitor: {
            NumericLiteral(p) {
              if (p.node.value === 17) p.node.value = 19;
            },
          },
        });
        const normal = [];
        if (fixture.latePost)
          normal.push(() => ({
            name: "late-jsx-host-post",
            post(file) {
              const extra = core.parseSync("<aside />;", {
                babelrc: false,
                configFile: false,
                parserOpts: { plugins: ["jsx"] },
              });
              file.path.pushContainer("body", extra.program.body);
            },
          }));
        if (fixture.generatorOverrides) {
          item.generatorCalls = [];
          for (let index = 0; index < fixture.generatorOverrides; index++)
            normal.push(() => ({
              name: "real-host-generator-" + index,
              generatorOverride(ast, generatorOptions, code, generate) {
                item.generatorCalls.push(index);
                return generate(ast, generatorOptions, code);
              },
            }));
        }
        if (fixture.typescript) normal.push([typescript, { isTSX: true }]);
        if (fixture.decorators) normal.push([decorators, { version: "2023-11" }]);
        if (fixture.classProperties) normal.push(classProperties);
        if (fixture.ordinaryEdit) normal.push(ordinary);
        if (fixture.reactTransform) normal.push([react, { runtime: "automatic" }]);
        if (input.phase === "positive" && fixture.modules !== false) normal.push(modules);
        const plugin = [vize, { jsxMode: "vdom", jsxCompat: "native" }];
        options.plugins = fixture.order === "last" ? [...normal, plugin] : [plugin, ...normal];
        if (fixture.overrideScope) {
          options.plugins = normal;
          options.overrides = [
            { test: /\.vue\.jsx$/, plugins: [plugin] },
            { test: /\.react\.jsx$/, plugins: [[react, { runtime: "automatic" }]] },
          ];
        }
        if (fixture.uidReservation) {
          item.reservations = [];
          const allocator = (label) => (api) => ({
            name: "deferred-host-uid-" + label,
            pre(file) {
              this.deferred = file.scope.generateUidIdentifier("openBlock");
              assert(
                !item.reservations.some((r) => r.name === this.deferred.name),
                "two actual pre allocators must receive distinct UIDs",
              );
              item.reservations.push({ label, name: this.deferred.name });
            },
            visitor: {
              Program: {
                exit(p) {
                  p.pushContainer(
                    "body",
                    api.types.variableDeclaration("const", [
                      api.types.variableDeclarator(
                        api.types.cloneNode(this.deferred),
                        api.types.stringLiteral("mine" + label),
                      ),
                    ]),
                  );
                  p.pushContainer(
                    "body",
                    api.types.exportNamedDeclaration(
                      api.types.variableDeclaration("const", [
                        api.types.variableDeclarator(
                          api.types.identifier("reserved" + label),
                          api.types.cloneNode(this.deferred),
                        ),
                      ]),
                    ),
                  );
                },
              },
            },
          });
          const a = allocator("A"),
            b = allocator("B");
          options.plugins =
            fixture.uidReservation === "split"
              ? [a, plugin, b, ...normal]
              : [a, b, ...normal, plugin];
        }
        if (fixture.pre) options.plugins.unshift(prePlugin);
        if (fixture.latePre) options.plugins.splice(1, 0, latePlugin);
        if (fixture.upstream) {
          const mapping = await load.mapping();
          options.sourceFileName = "intermediate.jsx";
          const upstream = upstreamMap(mapping, fixture.source, options.sourceFileName);
          options.inputSourceMap = upstream.map;
          item.upstream = upstream;
        }
        let ast;
        if (fixture.fromAst) {
          ast = core.parseSync(fixture.source, { ...freshParserOptions(options), plugins: [] });
          if (fixture.fromAst === "remove") removeJsx(core, ast);
          else if (fixture.fromAst === "edit") editJsx(ast, "ast-edited");
        }
        const beforeAst = ast ? astPacket(ast) : null;
        let actualError;
        try {
          item.result = ast
            ? core.transformFromAstSync(ast, fixture.source, {
                ...freshParserOptions(options),
                cloneInputAst: fixture.cloneInputAst ?? true,
              })
            : core.transformSync(fixture.source, freshParserOptions(options));
        } catch (error) {
          actualError = error;
          item.error = wholeError(error);
        }
        if (fixture.error) {
          assert(actualError, "the declared invalid host combination must fail");
          assert.equal(
            actualError.message,
            filename + ": " + fixture.error,
            "complete host error message",
          );
          continue;
        }
        if (actualError) throw actualError;
        assert(item.result);
        if (fixture.generatorOverrides) assert.deepEqual(item.generatorCalls, [0]);
        if (fixture.upstream) assertUpstream(await load.trace(), item.result, fixture.source);
        if (beforeAst && fixture.cloneInputAst !== false)
          assert.equal(
            astPacket(ast),
            beforeAst,
            "Babel cloneInputAst keeps every caller AST field unchanged",
          );
        if (beforeAst && fixture.cloneInputAst === false)
          assert.equal(
            hasJsx(ast),
            false,
            "the supplied File is the actual un-cloned transformed AST",
          );
        if (input.phase === "cold") {
          const baselineOptions = {
            ...options,
            plugins: options.plugins.filter((p) => p !== plugin),
          };
          if (options.overrides)
            baselineOptions.overrides = options.overrides.filter(
              (entry) => !entry.plugins.includes(plugin),
            );
          let baselineAst;
          if (beforeAst) {
            baselineAst = core.parseSync(fixture.source, {
              ...freshParserOptions(options),
              plugins: [],
            });
            if (fixture.fromAst === "remove") removeJsx(core, baselineAst);
            else if (fixture.fromAst === "edit") editJsx(baselineAst, "ast-edited");
          }
          const baseline = beforeAst
            ? core.transformFromAstSync(baselineAst, fixture.source, {
                ...freshParserOptions(baselineOptions),
                cloneInputAst: fixture.cloneInputAst ?? true,
              })
            : core.transformSync(fixture.source, freshParserOptions(baselineOptions));
          item.baseline = baseline;
          assert.deepEqual(
            outputPayload(item.result),
            outputPayload(baseline),
            "every Babel output field is unchanged; only the selected input plugin configuration differs",
          );
          assert.equal(report.resolutions.length, 0);
          assert.equal(addonImages(load.requireHost).length, 0);
          continue;
        }
        assertCompiled(item.result, fixture);
        core.parseSync(item.result.code, {
          babelrc: false,
          configFile: false,
          sourceType: "unambiguous",
        });
        if (fixture.uidReservation) {
          assert.equal(item.reservations.length, 2);
          assert.equal(
            item.reservations[0].name,
            "_openBlock",
            "the original pre reservation creates the real compiler-alias collision",
          );
          assert.equal(new Set(item.reservations.map((r) => r.name)).size, 2);
        }
        const runtimeDir = path.join(input.host.root, "node_modules/.vize-babel-runtime");
        fs.mkdirSync(runtimeDir, { recursive: true });
        const runtimeFile = path.join(
          runtimeDir,
          input.format + "-" + fixture.id + (fixture.modules === false ? ".mjs" : ".cjs"),
        );
        fs.writeFileSync(runtimeFile, item.result.code);
        const exports =
          fixture.modules === false
            ? await import(pathToFileURL(runtimeFile).href)
            : load.requireHost(runtimeFile);
        for (const [key, value] of Object.entries(fixture.exported ?? {}))
          assert.deepEqual(exports[key], value);
        const vue = await load.vue(),
          renderer = await load.renderer();
        const html = await renderer.renderToString(
          vue.createSSRApp({ render: () => exports.render(fixture.renderProps) }),
        );
        item.html = html;
        assert.equal(
          html,
          fixture.html,
          "whole actual Vue-rendered HTML after the host transforms",
        );
      } catch (error) {
        item.assertion = wholeError(error);
      }
    }
    report.loadedAddons = addonImages(load.requireHost);
    if (preparation)
      assert.deepEqual(
        report.loadedAddons,
        [{ path: preparation.image, sha256: preparation.preparation.sha256 }],
        "only the prepared physical source addon was loaded",
      );
    else {
      assert.deepEqual(report.loadedAddons, []);
      assert.deepEqual(
        report.resolutions,
        [],
        "all cold controls must avoid every native resolution attempt",
      );
    }
    assert.equal(report.observations.length, cases.length);
  } catch (error) {
    report.fatal = wholeError(error);
  } finally {
    hook?.deregister();
    json(input.report, report);
  }
  const failed = report.fatal || report.observations.some((item) => item.assertion);
  process.stdout.write(
    JSON.stringify({
      report: input.report,
      cases: report.observations.length,
      failed: Boolean(failed),
    }) + "\n",
  );
  if (failed) process.exitCode = 1;
}
