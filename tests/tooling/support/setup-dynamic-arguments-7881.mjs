// Full original #7881 modules and actual pinned Vue DOM listener ownership.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const evidence = {
  versions: null,
  stock: [],
  maps: [],
  observations: [],
  stage: "startup",
  currentCase: null,
};
let window;
try {
  assert.equal(process.env.NODE_ENV, "production");
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
  const url = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
  const runtimePath = fromUi.resolve("vue-vapor-runtime/dist/vue.runtime.esm-browser.prod.js");
  const runtimeCode = readFileSync(runtimePath, "utf8");
  const vue = await import(url(runtimeCode));
  assert.equal(vue.version, compiler.version);
  evidence.versions = {
    vue: vue.version,
    compiler: compiler.version,
    happyDom: fromUi("happy-dom/package.json").version,
    runtimeSha256: hash(runtimeCode),
  };
  const root = new URL(
    "../../_fixtures/differential/compiler/setup-dynamic-args-7881/",
    import.meta.url,
  );
  const custody = JSON.parse(readFileSync(new URL("custody.json", root)));
  const manifest = JSON.parse(
    readFileSync(new URL("../setup-dynamic-args-7881.manifest.json", root)),
  );
  assert.equal(manifest.cases.length, 1);
  assert.equal(manifest.cases[0].custody.sha256, hash(readFileSync(new URL("custody.json", root))));
  for (const [name, digest] of Object.entries(custody.files))
    assert.equal(hash(readFileSync(new URL(name, root))), digest, name);
  for (const row of manifest.cases[0].inputs.files)
    assert.equal(custody.files[row.path], row.sha256);
  assert.equal(manifest.cases[0].reference.sha256, custody.files["runtime.expected.json"]);
  const paths = JSON.parse(readFileSync(new URL("runtime.expected.json", root)));
  assert.deepEqual(Object.keys(paths), custody.cases);
  const expected = {};
  for (const name of custody.cases) {
    assert.equal(paths[name], `expected/${name}.json`);
    expected[name] = JSON.parse(readFileSync(new URL(paths[name], root)));
  }
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  assert.deepEqual(
    input.modes.map((row) => row.mode),
    ["inline", "separate"],
  );
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
  async function evaluate(code, child) {
    const registry = { vue, "./Child.vue": { default: child } };
    globalThis.__setupDynamicArgumentModules = registry;
    const transformed = transformSync(code, {
      configFile: false,
      babelrc: false,
      sourceMaps: false,
      plugins: [
        ({ types: t }) => ({
          visitor: {
            ImportDeclaration(path) {
              const source = path.node.source.value;
              assert.ok(source === "vue" || source === "./Child.vue", source);
              const declarations = path.node.specifiers.map((specifier) => {
                const imported = t.isImportSpecifier(specifier)
                  ? specifier.imported.name
                  : t.isImportDefaultSpecifier(specifier)
                    ? "default"
                    : null;
                assert.ok(
                  imported && Object.hasOwn(registry[source], imported),
                  `${source}:${imported}`,
                );
                return t.variableDeclarator(
                  t.identifier(specifier.local.name),
                  t.memberExpression(
                    t.memberExpression(
                      t.memberExpression(
                        t.identifier("globalThis"),
                        t.identifier("__setupDynamicArgumentModules"),
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
    return (await import(url(transformed))).default;
  }
  function observeChildren(parent) {
    return [...parent.childNodes].map((node) => {
      if (node.nodeType === 3 || node.nodeType === 8)
        return { type: node.nodeType, data: node.data };
      assert.equal(node.nodeType, 1);
      return {
        type: 1,
        tag: node.localName,
        namespace: node.namespaceURI,
        attributes: [...node.attributes].map((attr) => [attr.name, attr.value]),
        children: observeChildren(node),
      };
    });
  }
  for (const { mode, files } of input.modes) {
    assert.deepEqual(
      files.map((file) => file.name),
      custody.modules,
    );
    const official = new Map();
    const current = new Map();
    for (const file of files) {
      evidence.stage = "compile-reference";
      evidence.currentCase = { mode, name: file.name, stage: evidence.stage };
      const original = readFileSync(new URL(`${file.name}.vue.txt`, root), "utf8");
      assert.equal(file.source, original);
      assert.deepEqual(file.current.errors, []);
      assert.deepEqual(file.current.warnings, []);
      assert.equal(file.current.css, null);
      assert.deepEqual(file.current.macroArtifacts, []);
      const parsed = compiler.parse(original, { filename: `${file.name}.vue` });
      assert.deepEqual(parsed.errors, []);
      const { descriptor } = parsed;
      assert.ok(descriptor.scriptSetup && descriptor.template);
      const script = compiler.compileScript(descriptor, {
        id: file.name,
        inlineTemplate: mode === "inline",
        genDefaultAs: "__component",
        templateOptions: { isProd: true },
      });
      let code = script.content;
      let template = null;
      if (mode === "separate") {
        template = compiler.compileTemplate({
          source: descriptor.template.content,
          filename: `${file.name}.vue`,
          id: file.name,
          isProd: true,
          compilerOptions: { bindingMetadata: script.bindings },
        });
        assert.deepEqual(template.errors, []);
        assert.deepEqual(template.tips, []);
        code += `\n${template.code}\n__component.render = render;\n`;
      }
      code += "\nexport default __component;\n";
      const row = {
        mode,
        name: file.name,
        source: original,
        code,
        map: script.map,
        script: { code: script.content, bindings: script.bindings, map: script.map },
        template,
      };
      evidence.stock.push(row);
      evidence.maps.push({
        mode,
        name: file.name,
        current: mapGraph(file.current.code, file.current.map, original),
        officialScript: mapGraph(script.content, script.map, original),
        officialTemplate: template
          ? mapGraph(template.code, template.map, descriptor.template.content)
          : null,
        disposition: "current-script-provenance;official-raw-script-and-template",
      });
      const currentChild = current.get("Child");
      const officialChild = official.get("Child");
      current.set(file.name, await evaluate(file.current.code, currentChild));
      official.set(file.name, await evaluate(code, officialChild));
    }
    for (const [arm, components] of [
      ["official", official],
      ["current", current],
    ]) {
      for (const name of custody.cases) {
        evidence.stage = "mount";
        const diagnostics = [];
        const row = { mode, arm, name, phases: [], diagnostics };
        evidence.observations.push(row);
        evidence.currentCase = row;
        const host = window.document.createElement("main");
        window.document.body.append(host);
        const app = vue.createApp(components.get(name));
        app.config.warnHandler = (message) => diagnostics.push(message);
        app.config.errorHandler = (error) => diagnostics.push(String(error));
        let button;
        let mounted = false;
        const snapshot = (phase) => {
          const currentButton = host.querySelector("button");
          if (!button) button = currentButton;
          const observed = {
            phase,
            tree: observeChildren(host),
            diagnostics: [...diagnostics],
            sameButton: button !== null && currentButton === button,
          };
          row.phases.push(observed);
          assert.ok(button);
          assert.deepEqual(observed, expected[name][row.phases.length - 1]);
        };
        try {
          mounted = true;
          app.mount(host);
          await vue.nextTick();
          snapshot("initial");
          button.click();
          await vue.nextTick();
          snapshot("click");
          button.click();
          await vue.nextTick();
          snapshot(name === "Mutable" ? "stale-click" : "click-again");
          if (name === "Mutable") {
            button.dispatchEvent(new window.Event("mouseup", { bubbles: true }));
            await vue.nextTick();
            snapshot("mouseup");
          }
          app.unmount();
          mounted = false;
          await vue.nextTick();
          row.phases.push({
            phase: "unmount",
            tree: observeChildren(host),
            diagnostics: [...diagnostics],
            detachedButton: button.isConnected === false,
          });
          assert.deepEqual(row.phases, expected[name]);
          assert.equal(host.childNodes.length, 0);
          assert.deepEqual(diagnostics, []);
        } finally {
          if (mounted) {
            row.failureTreeBeforeCleanup = observeChildren(host);
            app.unmount();
          }
          host.remove();
        }
      }
    }
  }
  assert.equal(evidence.stock.length, 18);
  assert.equal(evidence.maps.length, 18);
  assert.equal(evidence.observations.length, 32);
  assert.equal(
    evidence.observations.reduce((n, row) => n + row.phases.length, 0),
    132,
  );
  evidence.stage = "complete";
  evidence.currentCase = null;
  evidence.nativeLevelCredit = false;
  evidence.browserHydrationCredit = false;
} catch (error) {
  evidence.error = { name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  try {
    if (window) await window.happyDOM.close();
  } catch (error) {
    evidence.cleanupError = String(error);
    process.exitCode = 1;
  }
  delete globalThis.__setupDynamicArgumentModules;
  await new Promise((resolve, reject) =>
    process.stdout.write(JSON.stringify(evidence), (error) => (error ? reject(error) : resolve())),
  );
}
function hash(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
