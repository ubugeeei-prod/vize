// Finite SSR component camel keys; separate from original8/nested12 populations.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const errorValue = (error) =>
  Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]]));
const evidence = {
  stage: "startup",
  input: null,
  stock: [],
  evaluation: [],
  observations: [],
  currentCase: null,
  process: {
    pid: process.pid,
    execPath: process.execPath,
    version: process.version,
    versions: process.versions,
  },
  completeOfficialMapParity: false,
};
try {
  assert.equal(process.env.NODE_ENV, "production");
  evidence.process.executableSha256 = hash(readFileSync(process.execPath));
  const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
  const compiler = fromVue("vue/compiler-sfc"),
    vue = fromVue("vue"),
    server = fromVue("vue/server-renderer");
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
  const corpus = JSON.parse(
    readFileSync(
      new URL(
        "../../_fixtures/differential/compiler/dynamic-component-camel/corpus.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  evidence.input = input;
  assert.deepEqual(input.corpus, corpus);
  assert.equal(corpus.cases.length, 12);
  assert.deepEqual(
    input.files.map(({ name }) => name),
    corpus.cases.map(({ name }) => name),
  );
  evidence.stage = "compile";
  for (const [index, file] of input.files.entries()) {
    const expected = corpus.cases[index];
    assert.equal(file.source, expected.source);
    assert.equal(Buffer.byteLength(file.source), expected.bytes);
    assert.equal(hash(file.source), expected.sha256);
    assert.deepEqual(file.current, file.legacy, `${file.name}: complete compiler routes`);
    assert.deepEqual(file.current.errors, []);
    assert.deepEqual(file.current.warnings, []);
    const filename = "format-v-pre-content.vue";
    const parsed = compiler.parse(file.source, { filename });
    const options = {
      source: parsed.descriptor.template.content,
      filename,
      id: filename,
      ssr: true,
      isProd: false,
      inMap: parsed.descriptor.template.map,
      compilerOptions: { prefixIdentifiers: true },
    };
    const result = compiler.compileTemplate(options);
    evidence.stock.push({ name: file.name, source: file.source, parsed, options, result });
    assert.deepEqual(parsed.errors, []);
    assert.deepEqual(result.errors, []);
  }
  let sequence = 0;
  async function evaluate(code) {
    const dependencies = { vue, "vue/server-renderer": server, "@vue/server-renderer": server };
    const slot = `__camelBoundaryModules${sequence++}`;
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
      const evaluatedCode = `${result.code}\n// instance ${sequence++}`;
      evidence.evaluation.push({
        currentCase: evidence.currentCase,
        originalCode: code,
        originalCodeSha256: hash(code),
        evaluatedCode,
        evaluatedCodeSha256: hash(evaluatedCode),
        bridge: "Babel import declarations only; raw maps describe original modules",
      });
      return await import(
        `data:text/javascript;base64,${Buffer.from(evaluatedCode).toString("base64")}`
      );
    } finally {
      delete globalThis[slot];
    }
  }
  evidence.stage = "render";
  for (const [index, file] of input.files.entries()) {
    const expected = corpus.cases[index];
    for (const arm of ["official", "current"]) {
      evidence.currentCase = { name: file.name, arm };
      const code = arm === "official" ? evidence.stock[index].result.code : file.current.code;
      const state = { ...structuredClone(corpus.state), ...structuredClone(expected.overrides) };
      const before = structuredClone(state),
        trace = [],
        received = [],
        warnings = [],
        errors = [];
      const observed = {
        name: file.name,
        arm,
        source: file.source,
        code,
        codeSha256: hash(code),
        camelImport: code.includes("camelize as _camelize"),
        before,
        after: state,
        trace,
        received,
        warnings,
        errors,
        html: null,
        outcomeError: null,
      };
      evidence.observations.push(observed);
      try {
        const module = await evaluate(code);
        const component = arm === "official" ? { ssrRender: module.ssrRender } : module.default;
        assert.equal(typeof component?.ssrRender, "function");
        const app = vue.createSSRApp(component);
        Object.assign(app.config.globalProperties, state, {
          nextKey: () => {
            trace.push("key");
            return state.key;
          },
          nextValue: () => {
            trace.push("value");
            return state.value;
          },
          spreadProps: () => {
            trace.push("spread");
            return { dataKey: "spread-value", b: "middle" };
          },
          handler: () => {
            trace.push("handler");
          },
        });
        app.component("Child", {
          inheritAttrs: false,
          setup(_props, context) {
            return () => {
              const attrs = Object.fromEntries(
                Object.entries(context.attrs).map(([key, value]) => [
                  key,
                  typeof value === "function" ? "[function]" : value,
                ]),
              );
              received.push(attrs);
              return vue.h("pre", JSON.stringify(attrs));
            };
          },
        });
        app.component("Host", {
          inheritAttrs: false,
          setup(_props, context) {
            return () => vue.h("section", context.slots.default({ row: state.slotRow }));
          },
        });
        app.config.warnHandler = (message, _instance, messageTrace) =>
          warnings.push({ message, trace: messageTrace });
        app.config.errorHandler = (error, _instance, info) =>
          errors.push({ error: errorValue(error), info });
        observed.html = await server.renderToString(app);
      } catch (error) {
        observed.outcomeError = errorValue(error);
      }
    }
  }
  // Every completed attempted render is retained before any runtime oracle.
  assert.equal(evidence.observations.length, 24);
  evidence.stage = "assert";
  for (const [index, expected] of corpus.cases.entries()) {
    const pair = evidence.observations.slice(index * 2, index * 2 + 2);
    for (const observed of pair) {
      assert.equal(observed.outcomeError, null, `${expected.name}/${observed.arm}`);
      assert.deepEqual(observed.after, observed.before);
      assert.deepEqual(observed.warnings, []);
      assert.deepEqual(observed.errors, []);
      assert.equal(observed.camelImport, expected.camelImport, `${expected.name}: helper import`);
      assert.deepEqual(
        observed.trace,
        expected.expectedTrace,
        `${expected.name}: evaluation count/order`,
      );
      assert.deepEqual(
        observed.received,
        expected.expectedProps,
        `${expected.name}: actual component keys`,
      );
    }
    assert.equal(pair[1].html, pair[0].html, `${expected.name}: complete actual HTML`);
  }
  evidence.stage = "complete";
  evidence.currentCase = null;
  process.stdout.write(JSON.stringify(evidence));
} catch (error) {
  evidence.error = errorValue(error);
  process.stdout.write(JSON.stringify(evidence));
  process.stderr.write(`${error.stack}\n`);
  process.exitCode = 1;
}
