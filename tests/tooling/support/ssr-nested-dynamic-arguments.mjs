// Whole original nested arguments, script-provenance maps and actual pinned SSR.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const evidence = {
  versions: null,
  process: {
    execPath: process.execPath,
    version: process.version,
    versions: process.versions,
    pid: process.pid,
  },
  input: null,
  stock: [],
  maps: [],
  observations: [],
  stage: "startup",
  currentCase: null,
};
const hash = (source) => createHash("sha256").update(source).digest("hex");
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
  evidence.versions = {
    compiler: compiler.version,
    vue: vue.version,
    server: fromVue("@vue/server-renderer/package.json").version,
    plugin: fromUi("@vitejs/plugin-vue/package.json").version,
  };
  assert.deepEqual(evidence.versions, {
    compiler: "3.6.0-rc.9",
    vue: "3.6.0-rc.9",
    server: "3.6.0-rc.9",
    plugin: "6.0.7",
  });
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  evidence.input = input;
  evidence.stage = "input";
  assert.deepEqual(
    input.files.map(({ name }) => name),
    ["Child", "Longhand", "Shorthand"],
  );
  const originals = [
    [1005, "c4560f416b1ef6f8cb5306b605ce78a4f96800d80fa2c34ef83b81c886c4cd47"],
    [347, "89af421cdeb8adc60e5e412de64635e2bdd2c4dcfe8a134781448fe3f0cc4d41"],
    [337, "9d9da696776f57ca7b424b858f20480327d7ae5100d6d659fce30bc139cd59c0"],
  ];
  for (const [index, file] of input.files.entries()) {
    assert.equal(Buffer.byteLength(file.source), originals[index][0], file.name);
    assert.equal(hash(file.source), originals[index][1], file.name);
    assert.deepEqual(file.current, file.legacy, "complete compiler routes before runtime");
    assert.deepEqual(file.current.errors, []);
    assert.deepEqual(file.current.warnings, []);
    assert.equal(file.current.css, null);
    assert.deepEqual(file.current.macroArtifacts, []);
    assert.equal(typeof file.current.code, "string");
    assert(file.current.code.length > 0);
  }
  const keys = { "names]": ["first", "second"], "events]": ["update:first", "update:second"] };
  assert.deepEqual(input.states, [
    { keys, indices: [0, 1], index: 0, value: "initial", other: "untouched" },
    { keys, indices: [0, 1], index: 1, value: "second", other: "untouched" },
    { keys, indices: [1, 0], index: 0, value: "reordered", other: "other" },
  ]);
  const helperId = "\0plugin-vue:export-helper";
  const url = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
  const context = {
    error(error) {
      throw Error(typeof error === "string" ? error : error.message);
    },
    warn(warning) {
      throw Error(typeof warning === "string" ? warning : warning.message);
    },
    addWatchFile() {},
  };
  const plugin = pluginFactory({ compiler });
  plugin.configResolved({
    root: "/ssr-nested-dynamic-arguments",
    command: "build",
    isProduction: true,
    build: { sourcemap: true },
    define: {},
    logger: { warn: (warning) => context.warn(warning) },
  });
  plugin.buildStart.call(context);

  function mapGraph(code, map, source) {
    assert.equal(map.version, 3);
    assert(Array.isArray(map.names) && map.names.every((name) => typeof name === "string"));
    assert(Array.isArray(map.sources) && map.sources.length > 0);
    assert(map.sources.every((name) => typeof name === "string"));
    assert.deepEqual(
      map.sourcesContent,
      map.sources.map(() => source),
    );
    assert.equal(typeof map.mappings, "string");
    const decoded = codec.decode(map.mappings);
    assert.equal(codec.encode(decoded), map.mappings);
    const generated = code.split(/\r\n|[\r\n\u2028\u2029]/);
    const original = source.split(/\r\n|[\r\n\u2028\u2029]/);
    const boundary = (line, column) => {
      assert(typeof line === "string" && column >= 0 && column <= line.length);
      assert(
        !(
          column > 0 &&
          column < line.length &&
          line.charCodeAt(column - 1) >= 0xd800 &&
          line.charCodeAt(column - 1) <= 0xdbff &&
          line.charCodeAt(column) >= 0xdc00 &&
          line.charCodeAt(column) <= 0xdfff
        ),
        "map coordinate splits a UTF-16 surrogate pair",
      );
    };
    let mapped = 0;
    decoded.forEach((segments, line) => {
      let previous = -1;
      for (const segment of segments) {
        assert([1, 4, 5].includes(segment.length));
        assert(segment.every(Number.isSafeInteger));
        const [column, owner, sourceLine, sourceColumn, name] = segment;
        assert(column >= previous);
        boundary(generated[line], column);
        previous = column;
        if (segment.length === 1) continue;
        assert(owner >= 0 && owner < map.sources.length);
        assert(sourceLine >= 0 && sourceLine < original.length);
        boundary(original[sourceLine], sourceColumn);
        if (segment.length === 5) assert(name >= 0 && name < map.names.length);
        mapped += 1;
      }
    });
    assert(mapped > 0, "whole original script map has source segments");
    return { decoded, mapped };
  }

  for (const file of input.files) {
    evidence.stage = "compile";
    evidence.currentCase = { name: file.name, arm: "compile" };
    const parsed = compiler.parse(file.source, { filename: `${file.name}.vue` });
    assert.deepEqual(parsed.errors, []);
    assert(parsed.descriptor.scriptSetup && parsed.descriptor.template);
    const result = await plugin.transform.handler.call(
      context,
      file.source,
      `/ssr-nested-dynamic-arguments/${file.name}.vue`,
      { ssr: true },
    );
    const helper = plugin.load.handler.call(context, helperId, { ssr: true });
    evidence.stock.push({ name: file.name, source: file.source, result, helper });
    assert.equal(typeof result?.code, "string");
    assert.equal(typeof helper, "string");
    const maps = {
      name: file.name,
      official: null,
      current: null,
      currentDisposition: "script-provenance-only",
      completeOfficialMapParity: false,
      templateArgumentAnchorCoverage: false,
    };
    evidence.maps.push(maps);
    evidence.stage = "map";
    evidence.currentCase.arm = "official-map";
    maps.official = mapGraph(result.code, result.map, file.source);
    evidence.currentCase.arm = "current-map";
    assert.deepEqual(file.current.map.sources, [`${file.name}.vue`]);
    maps.current = mapGraph(file.current.code, file.current.map, file.source);
  }

  let sequence = 0;
  async function evaluate(code, dependencies, helper, arm) {
    const modules = {
      vue,
      "vue/server-renderer": server,
      "@vue/server-renderer": server,
      ...dependencies,
    };
    if (helper) modules[helperId] = await import(url(helper));
    const slot = `__ssrNestedArgumentModules${sequence++}`;
    globalThis[slot] = modules;
    try {
      const transformed = transformSync(code, {
        configFile: false,
        babelrc: false,
        plugins: [
          ({ types: t }) => ({
            visitor: {
              ImportDeclaration(path) {
                const source = path.node.source.value;
                assert(Object.hasOwn(modules, source), `unexpected original import ${source}`);
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
                  assert(Object.hasOwn(modules[source], key), `${source}/${key}`);
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
      const loaded = await import(url(`${transformed.code}\n// whole SSR graph ${sequence++}`));
      if (arm === "official") {
        assert.equal(loaded.default?.__ssrInlineRender, true);
        assert.equal(typeof loaded.default.setup, "function");
      } else {
        assert.equal(arm, "current");
        assert(loaded.default && typeof loaded.default.ssrRender === "function");
      }
      return loaded.default;
    } finally {
      delete globalThis[slot];
    }
  }

  const escape = (text) =>
    String(text).replace(
      /[&<>"']/g,
      (character) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          '"': "&quot;",
          "'": "&#39;",
        })[character],
    );
  function expectedHtml(state) {
    const selected = state.indices[state.index];
    const prop = state.keys["names]"][selected];
    assert(["first", "second"].includes(prop));
    assert.equal(state.keys["events]"][selected], `update:${prop}`);
    const snapshot = { [prop]: state.value };
    const html =
      `<section><div><pre>${escape(JSON.stringify(snapshot))}</pre>` +
      '<button class="first">first</button><button class="second">second</button>' +
      '<button class="default">default</button><button class="fixed">fixed</button></div>' +
      `<output>${escape(state.value)}|${escape(state.other)}</output></section>`;
    return { prop, snapshot, html };
  }
  evidence.stage = "render";
  for (const [stateIndex, originalState] of input.states.entries()) {
    for (const parent of input.files.slice(1)) {
      const expected = expectedHtml(originalState);
      const pair = [];
      for (const arm of ["official", "current"]) {
        evidence.currentCase = { name: parent.name, stateIndex, arm };
        const state = structuredClone(originalState);
        const row = (file) =>
          arm === "current"
            ? file.current
            : evidence.stock.find(({ name }) => name === file.name).result;
        const helper = arm === "official" ? evidence.stock[0].helper : undefined;
        const child = await evaluate(row(input.files[0]).code, {}, helper, arm);
        const component = await evaluate(
          row(parent).code,
          {
            "./Child.vue": { default: child },
            "./fixture": { state },
          },
          helper,
          arm,
        );
        const app = vue.createSSRApp(component);
        const diagnostics = [];
        app.config.warnHandler = (message) => diagnostics.push(message);
        app.config.errorHandler = (error) => diagnostics.push(String(error));
        evidence.currentCase.diagnostics = diagnostics;
        const html = await server.renderToString(app);
        const observation = {
          name: parent.name,
          stateIndex,
          arm,
          state,
          html,
          diagnostics,
          expected,
        };
        evidence.observations.push(observation);
        assert.deepEqual(diagnostics, []);
        assert.deepEqual(state, originalState, "SSR never executes the event handler");
        assert.equal(
          html,
          expected.html,
          `${parent.name}/${arm}/${stateIndex}: whole original HTML`,
        );
        pair.push(html);
      }
      assert.equal(pair[0], pair[1], "official/current whole SSR HTML");
    }
  }
  assert.equal(evidence.observations.length, 12);
  evidence.stage = "complete";
  evidence.currentCase = null;
} catch (error) {
  evidence.error = { name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
  process.stderr.write(`${error.stack ?? error}\n`);
} finally {
  await new Promise((resolve, reject) => {
    process.stdout.write(JSON.stringify(evidence), (error) => (error ? reject(error) : resolve()));
  });
}
