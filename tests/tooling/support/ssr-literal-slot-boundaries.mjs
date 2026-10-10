// The same four public slot sources: complete current/stock SSR module execution.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const hash = (value) => createHash("sha256").update(value).digest("hex");
const errorValue = (error) =>
  Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]]));
const cases = [
  ["own-empty", "<template><slot v-pre></slot></template>", "<slot></slot>"],
  [
    "inherited-empty",
    "<template><div v-pre><slot></slot></div></template>",
    "<div><slot></slot></div>",
  ],
  ["normal-outlet", "<template><slot></slot></template>", "<!--[--><!--]-->"],
  [
    "original-nonempty",
    "<template><slot v-pre>{{ not }} an interpolation</slot></template>",
    "<slot>{{ not }} an interpolation</slot>",
  ],
];
const evidence = {
  stage: "startup",
  process: {
    pid: process.pid,
    execPath: process.execPath,
    version: process.version,
    versions: process.versions,
  },
  input: null,
  stock: [],
  evaluation: [],
  observations: [],
  currentCase: null,
  completeOfficialMapParity: false,
};
try {
  assert.equal(process.env.NODE_ENV, "production");
  evidence.process.executableSha256 = hash(readFileSync(process.execPath));
  const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
  const compiler = fromVue("vue/compiler-sfc");
  const vue = fromVue("vue");
  const server = fromVue("vue/server-renderer");
  const { transformSync } = fromUi("@babel/core");
  evidence.versions = {
    compiler: compiler.version,
    vue: vue.version,
    server: fromVue("@vue/server-renderer/package.json").version,
  };
  assert.deepEqual(evidence.versions, {
    compiler: "3.6.0-rc.9",
    vue: "3.6.0-rc.9",
    server: "3.6.0-rc.9",
  });
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const inputBytes = Buffer.concat(chunks);
  const input = JSON.parse(inputBytes.toString("utf8"));
  evidence.input = input;
  evidence.inputBytes = inputBytes.length;
  evidence.inputSha256 = hash(inputBytes);
  assert.deepEqual(input.files.map(({ name, source }) => [name, source]), cases.map((row) => row.slice(0, 2)));
  evidence.stage = "compile";
  for (const file of input.files) {
    assert.deepEqual(file.current, file.legacy, `${file.name}: all public fields`);
    assert.deepEqual(file.current.errors, []);
    assert.deepEqual(file.current.warnings, []);
    const filename = "SlotBoundary.vue";
    const parsed = compiler.parse(file.source, { filename });
    const stock = { name: file.name, source: file.source, sourceSha256: hash(file.source), parsed };
    evidence.stock.push(stock);
    assert.deepEqual(parsed.errors, []);
    const options = {
      source: parsed.descriptor.template.content,
      filename,
      id: filename,
      ssr: true,
      isProd: false,
      inMap: parsed.descriptor.template.map,
      compilerOptions: { prefixIdentifiers: true },
    };
    stock.options = options;
    stock.result = compiler.compileTemplate(options);
    assert.deepEqual(stock.result.errors, []);
  }
  let sequence = 0;
  async function evaluate(code, name, route) {
    const slot = `__literalSlotModules${sequence++}`;
    const dependencies = { vue, "vue/server-renderer": server, "@vue/server-renderer": server };
    globalThis[slot] = dependencies;
    try {
      const result = transformSync(code, {
        configFile: false,
        babelrc: false,
        plugins: [
          ({ types: t }) => ({
            visitor: {
              ImportDeclaration(path) {
                const source = path.node.source.value;
                assert(Object.hasOwn(dependencies, source), `unexpected import ${source}`);
                const declarations = path.node.specifiers.map((specifier) => {
                  const owner = t.memberExpression(
                    t.memberExpression(t.identifier("globalThis"), t.stringLiteral(slot), true),
                    t.stringLiteral(source),
                    true,
                  );
                  if (t.isImportNamespaceSpecifier(specifier))
                    return t.variableDeclarator(specifier.local, owner);
                  const key = t.isImportDefaultSpecifier(specifier)
                    ? "default"
                    : (specifier.imported.name ?? specifier.imported.value);
                  assert(Object.hasOwn(dependencies[source], key), `${source}/${key}`);
                  return t.variableDeclarator(
                    specifier.local,
                    t.memberExpression(owner, t.stringLiteral(key), true),
                  );
                });
                if (declarations.length)
                  path.replaceWith(t.variableDeclaration("const", declarations));
                else path.remove();
              },
            },
          }),
        ],
      });
      evidence.evaluation.push({
        name, route, originalCode: code, originalCodeSha256: hash(code),
        evaluatedCode: result.code, evaluatedCodeSha256: hash(result.code),
        bridge: "Babel ImportDeclaration-only; original modules and raw maps retained separately",
      });
      return await import(`data:text/javascript;base64,${Buffer.from(result.code).toString("base64")}`);
    } finally {
      delete globalThis[slot];
    }
  }
  evidence.stage = "evaluate";
  for (const [index, file] of input.files.entries()) {
    for (const route of ["current", "official"]) {
      evidence.currentCase = { name: file.name, route };
      const result = route === "current" ? file.current : evidence.stock[index].result;
      const warnings = [], errors = [], context = {};
      const observation = {
        name: file.name, route, source: file.source, sourceSha256: hash(file.source),
        html: null, warnings, errors, context, outcomeError: null,
      };
      evidence.observations.push(observation);
      try {
        const module = await evaluate(result.code, file.name, route);
        let component;
        if (route === "current") {
          assert.deepEqual(Object.keys(module), ["default"]);
          assert.equal(typeof module.default, "object");
          assert.equal(typeof module.default.ssrRender, "function");
          component = module.default;
          observation.contract = "whole public SFC default export with ssrRender";
        } else {
          assert.deepEqual(Object.keys(module), ["ssrRender"]);
          assert.equal(typeof module.ssrRender, "function");
          component = { ssrRender: module.ssrRender };
          observation.contract = "official compileTemplate named ssrRender export";
        }
        const app = vue.createSSRApp(component);
        app.config.warnHandler = (message, _instance, trace) => warnings.push({ message, trace });
        app.config.errorHandler = (error, _instance, info) => errors.push({ error: errorValue(error), info });
        observation.html = await server.renderToString(app, context);
      } catch (error) {
        observation.outcomeError = errorValue(error);
      }
    }
  }
  evidence.stage = "complete";
  const wholeOutcome = ({ name, source, sourceSha256, html, context, warnings, errors, outcomeError }) =>
    ({ name, source, sourceSha256, html, context, warnings, errors, outcomeError });
  evidence.judges = evidence.observations.map((row, index) => ({
    name: row.name, route: row.route, expectedHtml: cases[Math.floor(index / 2)][2],
    expectedContext: { __instanceScopes: [] },
    pass: row.html === cases[Math.floor(index / 2)][2]
      && JSON.stringify(row.context) === JSON.stringify({ __instanceScopes: [] })
      && JSON.stringify(wholeOutcome(row)) === JSON.stringify(wholeOutcome(evidence.observations[index ^ 1]))
      && row.warnings.length === 0 && row.errors.length === 0 && row.outcomeError === null,
  }));
  assert.equal(evidence.observations.length, 8);
  if (!evidence.judges.every((row) => row.pass)) process.exitCode = 1;
} catch (error) {
  evidence.error = errorValue(error);
  process.stderr.write(`${error.stack}\n`);
  process.exitCode = 1;
}
process.stdout.write(JSON.stringify(evidence));
