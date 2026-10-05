// Whole original #7891 defaults, source maps and real official server rendering.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const evidence = {
  versions: null,
  stock: [],
  maps: [],
  observations: [],
  stage: "startup",
  currentCase: null,
};
try {
  assert.equal(process.env.NODE_ENV, "production");
  const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
  const compiler = fromVue("vue/compiler-sfc");
  const vue = fromVue("vue");
  const server = fromVue("vue/server-renderer");
  const { transformSync } = fromUi("@babel/core");
  const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
  const codec = createRequire(fromSfc.resolve("magic-string/package.json"))(
    "@jridgewell/sourcemap-codec",
  );
  const pluginFactory = (await import(pathToFileURL(fromUi.resolve("@vitejs/plugin-vue")).href))
    .default;
  assert.equal(compiler.version, "3.6.0-rc.9");
  assert.equal(vue.version, compiler.version);
  assert.equal(fromVue("@vue/server-renderer/package.json").version, compiler.version);
  assert.equal(fromUi("@vitejs/plugin-vue/package.json").version, "6.0.7");
  evidence.versions = { vue: vue.version, plugin: "6.0.7" };
  const root = new URL("../../_fixtures/differential/compiler/ssr-builtins-7891/", import.meta.url);
  const custody = JSON.parse(readFileSync(new URL("custody.json", root)));
  const manifest = JSON.parse(readFileSync(new URL("../ssr-builtins-7891.manifest.json", root)));
  assert.equal(manifest.cases.length, 1);
  assert.equal(
    manifest.cases[0].custody.sha256,
    createHash("sha256")
      .update(readFileSync(new URL("custody.json", root)))
      .digest("hex"),
  );
  for (const file of manifest.cases[0].inputs.files) {
    assert.equal(custody.files[file.path], file.sha256);
  }
  assert.equal(manifest.cases[0].reference.sha256, custody.files["runtime.expected.json"]);
  for (const [name, hash] of Object.entries(custody.files)) {
    assert.equal(
      createHash("sha256")
        .update(readFileSync(new URL(name, root)))
        .digest("hex"),
      hash,
    );
  }
  const expected = JSON.parse(readFileSync(new URL("runtime.expected.json", root)));
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  assert.deepEqual(
    input.files.map(({ name }) => name),
    ["Async", ...custody.cases],
  );
  const helperId = "\0plugin-vue:export-helper";
  const url = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
  const context = {
    error(error) {
      throw new Error(typeof error === "string" ? error : error.message);
    },
    warn(warning) {
      throw new Error(typeof warning === "string" ? warning : warning.message);
    },
    addWatchFile() {},
  };
  const plugin = pluginFactory({ compiler });
  plugin.configResolved({
    root: "/ssr-builtins",
    command: "build",
    isProduction: true,
    build: { sourcemap: true },
    define: {},
    logger: { warn: (warning) => context.warn(warning) },
  });
  plugin.buildStart.call(context);
  function mapGraph(code, map, source) {
    assert.equal(map.version, 3);
    assert.ok(Array.isArray(map.names));
    assert.ok(Array.isArray(map.sources) && map.sources.length > 0);
    assert.deepEqual(
      map.sourcesContent,
      map.sources.map(() => source),
    );
    const decoded = codec.decode(map.mappings);
    assert.equal(codec.encode(decoded), map.mappings);
    const generated = code.split(/\r\n|[\r\n\u2028\u2029]/);
    const original = source.split(/\r\n|[\r\n\u2028\u2029]/);
    let mapped = 0;
    decoded.forEach((segments, line) => {
      let previous = -1;
      for (const segment of segments) {
        assert.ok([1, 4, 5].includes(segment.length));
        assert.ok(segment.every(Number.isSafeInteger));
        const [column, owner, sourceLine, sourceColumn, name] = segment;
        assert.ok(
          generated[line] !== undefined && column >= previous && column <= generated[line].length,
        );
        previous = column;
        if (segment.length === 1) continue;
        assert.ok(owner >= 0 && owner < map.sources.length);
        assert.ok(
          original[sourceLine] !== undefined &&
            sourceColumn >= 0 &&
            sourceColumn <= original[sourceLine].length,
        );
        if (segment.length === 5) assert.ok(name >= 0 && name < map.names.length);
        mapped += 1;
      }
    });
    assert.ok(mapped > 0);
    return decoded;
  }
  const stock = evidence.stock;
  const maps = evidence.maps;
  evidence.stage = "compile";
  for (const file of input.files) {
    evidence.currentCase = { name: file.name, arm: "compile" };
    assert.equal(file.source, readFileSync(new URL(`${file.name}.vue.txt`, root), "utf8"));
    const result = await plugin.transform.handler.call(
      context,
      file.source,
      `/ssr-builtins/${file.name}.vue`,
      { ssr: true },
    );
    assert.equal(typeof result?.code, "string");
    assert.ok(result.map && typeof result.map.mappings === "string");
    const helper = plugin.load.handler.call(context, helperId, {});
    assert.equal(typeof helper, "string");
    stock.push({
      name: file.name,
      source: file.source,
      code: result.code,
      map: result.map,
      helper,
    });
    assert.ok(file.current.map && typeof file.current.map.mappings === "string");
    assert.deepEqual(file.current.map.sources, [`${file.name}.vue`]);
    assert.deepEqual(file.current.map.sourcesContent, [file.source]);
    maps.push({
      name: file.name,
      official: mapGraph(result.code, result.map, file.source),
      current: mapGraph(file.current.code, file.current.map, file.source),
    });
  }
  let sequence = 0;
  async function evaluate(code, dependencies, helper) {
    const modules = {
      vue,
      "vue/server-renderer": server,
      "@vue/server-renderer": server,
      ...dependencies,
    };
    if (helper) modules[helperId] = await import(url(helper));
    const slot = `__ssrBuiltinModules${sequence++}`;
    globalThis[slot] = modules;
    try {
      const result = transformSync(code, {
        configFile: false,
        babelrc: false,
        plugins: [
          ({ types: t }) => ({
            visitor: {
              ImportDeclaration(path) {
                const source = path.node.source.value;
                assert.ok(Object.hasOwn(modules, source), source);
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
                    : specifier.imported.name;
                  assert.ok(Object.hasOwn(modules[source], key), `${source}/${key}`);
                  return t.variableDeclarator(
                    specifier.local,
                    t.memberExpression(owner, t.stringLiteral(key), true),
                  );
                });
                path.replaceWith(t.variableDeclaration("const", declarations));
              },
            },
          }),
        ],
      });
      const selected = await import(url(`${result.code}\n// whole SSR component ${sequence++}`));
      assert.ok(selected.default && typeof selected.default === "object");
      return selected.default;
    } finally {
      delete globalThis[slot];
    }
  }
  const observations = evidence.observations;
  evidence.stage = "render";
  for (const useCurrent of [false, true]) {
    const components = {};
    for (const file of input.files) {
      evidence.currentCase = { name: file.name, arm: useCurrent ? "current" : "official" };
      const reference = stock.find(({ name }) => name === file.name);
      const selected = useCurrent ? file.current : reference;
      components[file.name] = await evaluate(
        selected.code,
        { "./Async.vue": { default: components.Async } },
        selected.helper,
      );
      if (file.name === "Async") continue;
      const app = vue.createSSRApp(components[file.name]);
      const diagnostics = [];
      app.config.warnHandler = (message) => diagnostics.push(message);
      app.config.errorHandler = (error) => diagnostics.push(String(error));
      evidence.currentCase.diagnostics = diagnostics;
      const html = await server.renderToString(app);
      observations.push({
        name: file.name,
        arm: useCurrent ? "current" : "official",
        html,
        diagnostics,
      });
      assert.deepEqual(diagnostics, []);
      assert.equal(
        html,
        expected[file.name],
        `${useCurrent ? "current" : "official"}/${file.name}: whole HTML`,
      );
    }
  }
  assert.equal(observations.length, custody.cases.length * 2);
  evidence.stage = "complete";
  evidence.currentCase = null;
} catch (error) {
  evidence.error = { name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
  await new Promise((resolve, reject) => {
    process.stderr.write(`${error.stack ?? error}\n`, (failure) =>
      failure ? reject(failure) : resolve(),
    );
  });
} finally {
  await new Promise((resolve, reject) => {
    process.stdout.write(JSON.stringify(evidence), (failure) =>
      failure ? reject(failure) : resolve(),
    );
  });
}
