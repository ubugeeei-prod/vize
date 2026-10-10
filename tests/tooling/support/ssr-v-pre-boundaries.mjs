// Separate v-pre source/code/map/HTML witnesses; original nested12 stays intact.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const evidence = {
  stage: "startup",
  process: {
    execPath: process.execPath,
    version: process.version,
    versions: process.versions,
    pid: process.pid,
  },
  input: null,
  stock: [],
  evaluation: [],
  observations: [],
  currentCase: null,
  completeOfficialMapParity: false,
};
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const errorValue = (error) =>
  Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]]));
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
  const root = new URL(
    "../../_fixtures/differential/compiler/v-pre-literal-boundary/",
    import.meta.url,
  );
  const corpus = JSON.parse(readFileSync(new URL("corpus.json", root), "utf8"));
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  evidence.input = input;
  assert.deepEqual(
    input.files.map(({ name }) => name),
    corpus.cases.map(({ name }) => name),
  );
  assert.equal(corpus.cases.length, 8);
  assert.deepEqual(input.state, {
    active: "active-id",
    keys: { "name]": "data-key" },
    value: "normal-value",
  });
  evidence.stage = "compile";
  for (const [index, file] of input.files.entries()) {
    const pinned = corpus.cases[index];
    const source = readFileSync(new URL(pinned.file, root), "utf8");
    assert.equal(file.source, source);
    assert.equal(Buffer.byteLength(source), pinned.bytes);
    assert.equal(hash(source), pinned.sha256);
    assert.deepEqual(file.current, file.legacy, `${file.name}: complete compiler routes`);
    assert.deepEqual(file.current.errors, []);
    assert.deepEqual(file.current.warnings, []);
    const filename = "format-v-pre-content.vue";
    const parsed = compiler.parse(source, { filename });
    const stock = { name: file.name, source, parsed, modules: [] };
    evidence.stock.push(stock);
    assert.deepEqual(parsed.errors, []);
    for (const ssr of [false, true]) {
      const options = {
        source: parsed.descriptor.template.content,
        filename,
        id: filename,
        ssr,
        isProd: false,
        inMap: parsed.descriptor.template.map,
        compilerOptions: { prefixIdentifiers: true },
      };
      const result = compiler.compileTemplate(options);
      stock.modules.push({ ssr, options, result });
      assert.deepEqual(result.errors, []);
    }
  }
  let sequence = 0;
  async function evaluate(code) {
    const dependencies = { vue, "vue/server-renderer": server, "@vue/server-renderer": server };
    const slot = `__vPreBoundaryModules${sequence++}`;
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
        bridge:
          "Babel import declarations only; retained source maps describe the original modules",
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
    const official = evidence.stock[index].modules.find(({ ssr }) => ssr).result;
    for (const arm of ["official", "current"]) {
      evidence.currentCase = { name: file.name, arm };
      const code = arm === "official" ? official.code : file.current.code;
      const state = structuredClone(input.state);
      const before = structuredClone(state);
      const warnings = [],
        errors = [];
      const observed = {
        name: file.name,
        arm,
        source: file.source,
        codeSha256: hash(code),
        html: null,
        outcomeError: null,
        before,
        after: state,
        warnings,
        errors,
      };
      evidence.observations.push(observed);
      try {
        const module = await evaluate(code);
        const component = arm === "official" ? { ssrRender: module.ssrRender } : module.default;
        assert.equal(typeof component?.ssrRender, "function");
        const app = vue.createSSRApp(component);
        Object.assign(app.config.globalProperties, state);
        for (const name of ["Child", "Row"]) {
          app.component(name, { render: () => vue.h("strong", { "data-component": name }, name) });
        }
        app.config.warnHandler = (message, _instance, trace) => warnings.push({ message, trace });
        app.config.errorHandler = (error, _instance, info) =>
          errors.push({ error: errorValue(error), info });
        observed.html = await server.renderToString(app);
      } catch (error) {
        observed.outcomeError = errorValue(error);
      }
    }
  }
  assert.equal(evidence.observations.length, 16);
  evidence.stage = "assert";
  for (const [index, file] of input.files.entries()) {
    const pair = evidence.observations.slice(index * 2, index * 2 + 2);
    for (const observed of pair) {
      assert.equal(observed.outcomeError, null, `${file.name}/${observed.arm}`);
      assert.deepEqual(observed.after, observed.before);
      assert.deepEqual(observed.warnings, []);
      assert.deepEqual(observed.errors, []);
    }
    assert.equal(pair[1].html, pair[0].html, `${file.name}: complete actual HTML`);
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
