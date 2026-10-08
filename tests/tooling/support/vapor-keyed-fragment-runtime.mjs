// Original #7883 source modules, complete maps, replacement and scope disposal.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";
import { keyedCases, keyedExpectations } from "./vapor-keyed-fragment-expectations.mjs";

const evidence = { versions: null, stock: [], maps: [], observations: [], stage: "startup" };
let window;
try {
  const root = new URL(
    "../../_fixtures/differential/compiler/vapor-keyed-fragment/",
    import.meta.url,
  );
  const custody = JSON.parse(readFileSync(new URL("custody.json", root)));
  const originals = new Map();
  for (const file of custody.files) {
    const bytes = readFileSync(new URL(file.path, root));
    assert.equal(bytes.length, file.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
    originals.set(file.path, bytes.toString("utf8"));
  }
  const expected = keyedExpectations(originals);
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  assert.equal(typeof input.production, "boolean");
  assert.equal(typeof input.inline, "boolean");
  assert.deepEqual(
    input.cases.map(({ name }) => name),
    keyedCases,
  );
  const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
  const compiler = fromVue("vue/compiler-sfc");
  const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
  const codec = createRequire(fromSfc.resolve("magic-string/package.json"))(
    "@jridgewell/sourcemap-codec",
  );
  const { transformSync } = fromUi("@babel/core");
  const { Window } = await import(fromUi.resolve("happy-dom"));
  assert.equal(compiler.version, "3.6.0-rc.9");
  window = new Window();
  for (const name of [
    "window",
    "document",
    "Document",
    "Node",
    "Text",
    "Comment",
    "Element",
    "HTMLElement",
    "SVGElement",
    "Event",
    "ShadowRoot",
  ])
    globalThis[name] = name === "window" ? window : window[name];
  const vue = await loadRuntime({ production: input.production });
  assert.equal(vue.version, compiler.version);
  evidence.versions = {
    vue: vue.version,
    compiler: compiler.version,
    happyDom: fromUi("happy-dom/package.json").version,
  };
  function mapGraph(code, map, source) {
    assert.equal(map.version, 3);
    assert.ok(Array.isArray(map.names));
    assert.ok(Array.isArray(map.sources) && map.sources.length > 0);
    assert.deepEqual(
      map.sourcesContent,
      map.sources.map(() => source),
    );
    const graph = codec.decode(map.mappings);
    assert.equal(codec.encode(graph), map.mappings);
    const generated = code.split(/\r\n|[\r\n\u2028\u2029]/);
    const original = source.split(/\r\n|[\r\n\u2028\u2029]/);
    let mapped = 0;
    graph.forEach((segments, line) => {
      let previous = -1;
      for (const segment of segments) {
        assert.ok([1, 4, 5].includes(segment.length));
        assert.ok(segment.every(Number.isSafeInteger));
        const [column, owner, sourceLine, sourceColumn, name] = segment;
        assert.ok(
          generated[line] !== undefined &&
            column >= 0 &&
            column >= previous &&
            column <= generated[line].length,
        );
        previous = column;
        if (segment.length === 1) continue;
        assert.ok(owner >= 0 && owner < map.sources.length);
        assert.ok(sourceLine >= 0 && sourceLine < original.length);
        assert.ok(sourceColumn >= 0 && sourceColumn <= original[sourceLine].length);
        if (segment.length === 5) assert.ok(name >= 0 && name < map.names.length);
        mapped++;
      }
    });
    assert.ok(mapped > 0);
    return graph;
  }
  function official(fixture) {
    const parsed = compiler.parse(fixture.source, { filename: fixture.filename });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.source, fixture.source);
    const script = compiler.compileScript(parsed.descriptor, {
      id: fixture.filename,
      isProd: input.production,
      inlineTemplate: true,
      genDefaultAs: "__reference",
    });
    return { code: `${script.content}\nexport default __reference;`, map: script.map };
  }
  let sequence = 0;
  async function evaluate(code, child) {
    globalThis.__keyedFragmentModules = {
      vue,
      "vue/vapor": vue,
      "./Counter.vue": { default: child },
    };
    const transformed = transformSync(code, {
      configFile: false,
      babelrc: false,
      plugins: [
        ({ types: t }) => ({
          visitor: {
            ImportDeclaration(path) {
              const source = path.node.source.value;
              assert.ok(Object.hasOwn(globalThis.__keyedFragmentModules, source), source);
              const declarations = path.node.specifiers.map((specifier) => {
                const imported = t.isImportDefaultSpecifier(specifier)
                  ? "default"
                  : specifier.imported.name;
                assert.ok(
                  Object.hasOwn(globalThis.__keyedFragmentModules[source], imported),
                  imported,
                );
                return t.variableDeclarator(
                  specifier.local,
                  t.memberExpression(
                    t.memberExpression(
                      t.memberExpression(
                        t.identifier("globalThis"),
                        t.identifier("__keyedFragmentModules"),
                      ),
                      t.stringLiteral(source),
                      true,
                    ),
                    t.stringLiteral(imported),
                    true,
                  ),
                );
              });
              path.replaceWith(t.variableDeclaration("const", declarations));
            },
          },
        }),
      ],
    }).code;
    return (
      await import(
        `data:text/javascript;base64,${Buffer.from(`${transformed}\n// ${sequence++}`).toString("base64")}`
      )
    ).default;
  }
  async function mounted(name, module, counterModule, capture) {
    const counter = await evaluate(counterModule.code, null);
    let setups = 0;
    let disposed = 0;
    const setup = counter.setup;
    assert.equal(typeof setup, "function");
    counter.setup = function (...args) {
      setups++;
      vue.onUnmounted(() => disposed++);
      return setup.apply(this, args);
    };
    const component = await evaluate(module.code, counter);
    const nestedFallthrough = ["nested-attrs", "nested-if-attrs"].includes(name);
    const fallthrough = nestedFallthrough || ["root-attrs", "if-attrs"].includes(name);
    const app = fallthrough
      ? vue.createVaporApp(component, { title: "outer" })
      : vue.createVaporApp(component);
    const diagnostics = [];
    app.config.warnHandler = (message) => diagnostics.push(message);
    app.config.errorHandler = (error) => diagnostics.push(String(error));
    const host = window.document.createElement("div");
    window.document.body.append(host);
    const rows = [];
    const raw = [];
    Object.assign(capture, { rows, raw, diagnostics });
    const selector = name === "element" || fallthrough ? "input" : "i";
    let previous;
    const snapshot = () => {
      const node = host.querySelector(selector);
      rows.push({
        tree: observeChildren(host),
        same: Boolean(node && node === previous),
        setups,
        disposed,
        diagnostics: [...diagnostics],
      });
      raw.push({ html: host.innerHTML, targetConnected: node?.isConnected ?? false });
      if (node) previous = node;
    };
    const click = async (selector) => {
      const node = host.querySelector(selector);
      assert.ok(node, selector);
      node.click();
      await vue.nextTick();
      snapshot();
    };
    try {
      app.mount(host);
      await vue.nextTick();
      previous = host.querySelector(selector);
      assert.ok(previous);
      snapshot();
      if (fallthrough) {
        if (nestedFallthrough) await click("button:nth-of-type(2)");
        else {
          previous.dispatchEvent(new window.Event("dblclick", { bubbles: true }));
          await vue.nextTick();
          snapshot();
        }
        const old = previous;
        await click(nestedFallthrough ? "button" : "input");
        assert.equal(old.isConnected, false);
      } else if (name === "element") {
        previous.value = "typed";
        snapshot();
        const old = previous;
        await click("button");
        assert.equal(old.isConnected, false);
        await click("button:nth-of-type(2)");
      } else {
        await click("i");
        const old = previous;
        await click("button");
        assert.equal(old.isConnected, name === "stable");
        if (name === "component") await click("i");
      }
      app.unmount();
      await vue.nextTick();
      assert.equal(host.childNodes.length, 0);
      snapshot();
      assert.deepEqual(rows, expected[name]);
    } finally {
      if (host.childNodes.length) app.unmount();
      host.remove();
    }
  }
  assert.equal(input.counter.source, originals.get("Counter.vue.txt"));
  assert.equal(input.counter.filename, "Counter.vue");
  const counterReference = official(input.counter);
  evidence.stock.push({ name: "Counter", ...counterReference });
  evidence.maps.push({
    name: "Counter",
    kind: "official",
    graph: mapGraph(counterReference.code, counterReference.map, input.counter.source),
  });
  for (const fixture of [input.counter, ...input.cases]) {
    const graph = mapGraph(
      fixture.compiled.mapped.code,
      fixture.compiled.mapped.map,
      fixture.source,
    );
    evidence.maps.push({ name: fixture.filename, kind: "current-script-module", graph });
  }
  for (const fixture of input.cases) {
    assert.equal(
      fixture.source,
      originals.get(
        {
          component: "App.vue.txt",
          element: "element.vue.txt",
          stable: "stable.vue.txt",
          "nested-attrs": "nested-attrs.vue.txt",
          "root-attrs": "root-attrs.vue.txt",
          "if-attrs": "if-attrs.vue.txt",
          "nested-if-attrs": "nested-if-attrs.vue.txt",
        }[fixture.name],
      ),
    );
    const reference = official(fixture);
    evidence.stock.push({ name: fixture.name, ...reference });
    evidence.maps.push({
      name: fixture.name,
      kind: "official",
      graph: mapGraph(reference.code, reference.map, fixture.source),
    });
    const row = {
      name: fixture.name,
      production: input.production,
      inline: input.inline,
      current: {},
      reference: {},
      status: "RUNNING",
    };
    evidence.observations.push(row);
    await mounted(fixture.name, reference, counterReference, row.reference);
    await mounted(fixture.name, fixture.compiled.plain, input.counter.compiled.plain, row.current);
    assert.deepEqual(row.current.rows, row.reference.rows);
    row.status = "PASS";
  }
  evidence.stage = "PASS";
} catch (error) {
  evidence.stage = "FAIL";
  evidence.error = { name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  if (window) {
    try {
      await window.happyDOM.close();
    } catch (error) {
      evidence.cleanupError = { name: error.name, message: error.message };
      evidence.stage = "FAIL";
      process.exitCode = 1;
    }
  }
  delete globalThis.__keyedFragmentModules;
  await new Promise((resolve) => process.stdout.write(JSON.stringify(evidence), resolve));
}
