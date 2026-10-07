// #7885: unchanged real parent/child sources, complete modules and mounted events.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";

const evidence = { stage: "startup", versions: null, rows: [], currentCase: null };
let window;
try {
  const root = new URL("../../_fixtures/differential/compiler/", import.meta.url);
  const manifest = JSON.parse(
    readFileSync(new URL("vapor-component-listeners.manifest.json", root)),
  );
  assert.equal(manifest.cases.length, 1);
  const pin = manifest.cases[0];
  assert.equal(pin.id, "compiler/sfc/vapor-component-listeners");
  const originals = new Map();
  for (const file of pin.inputs.files) {
    const bytes = readFileSync(new URL(`${pin.inputs.root}/${file.path}`, root));
    assert.equal(hash(bytes), file.sha256, file.path);
    originals.set(file.path, bytes.toString("utf8"));
  }
  const expectedBytes = readFileSync(new URL(pin.reference.path, root));
  assert.equal(hash(expectedBytes), pin.reference.sha256);
  const expected = JSON.parse(expectedBytes);
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  assert.equal(typeof input.production, "boolean");
  const profiles = {
    reported: ["App.vue.txt", "Child.vue.txt", true],
    vdom: ["Vdom.vue.txt", "VdomChild.vue.txt", false],
    callbacks: ["Callbacks.vue.txt", "Child.vue.txt", true],
    spread: ["Spread.vue.txt", "Child.vue.txt", true],
  };
  assert.deepEqual(
    input.cases.map(({ name, mode }) => `${name}:${mode}`).sort(),
    Object.keys(profiles)
      .flatMap((name) => [`${name}:inline`, `${name}:separate`])
      .sort(),
  );
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
    "MutationObserver",
  ])
    globalThis[name] = name === "window" ? window : window[name];
  const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
  const compiler = fromVue("vue/compiler-sfc");
  const vue = await loadRuntime({ production: input.production });
  assert.equal(compiler.version, pin.provenance.versions.vueRuntime);
  assert.equal(vue.version, compiler.version);
  const fromPlugin = createRequire(fromUi.resolve("@vitejs/plugin-vue/package.json"));
  const { transformWithOxc } = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
  const { transformSync } = fromUi("@babel/core");
  evidence.versions = {
    runtime: vue.version,
    compiler: compiler.version,
    production: input.production,
  };
  evidence.sourceCustody = input.sourceCustody;
  let sequence = 0;

  async function evaluate(code, child, capture) {
    const registry = { vue, "vue/vapor": vue, "./Child.vue": { default: child } };
    globalThis.__componentListenerModules = registry;
    const plain = await transformWithOxc(code, "Component.vue.ts", {
      lang: "ts",
      sourcemap: false,
    });
    capture.transpiledCode = plain.code;
    const normalized = transformSync(plain.code, {
      configFile: false,
      babelrc: false,
      plugins: [
        ({ types: t }) => ({
          visitor: {
            ImportDeclaration(path) {
              const source = path.node.source.value;
              assert.ok(Object.hasOwn(registry, source), source);
              const declarations = path.node.specifiers.map((specifier) => {
                const imported = t.isImportDefaultSpecifier(specifier)
                  ? "default"
                  : t.isImportSpecifier(specifier)
                    ? specifier.imported.name
                    : null;
                assert.ok(
                  imported && Object.hasOwn(registry[source], imported),
                  `${source}:${imported}`,
                );
                return t.variableDeclarator(
                  specifier.local,
                  t.memberExpression(
                    t.memberExpression(
                      t.memberExpression(
                        t.identifier("globalThis"),
                        t.identifier("__componentListenerModules"),
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
    capture.loadedCode = normalized;
    return (
      await import(
        `data:text/javascript;base64,${Buffer.from(
          `${normalized}\n// isolated component ${sequence++}`,
        ).toString("base64")}`
      )
    ).default;
  }

  function reference(file, vapor) {
    const parsed = compiler.parse(file.source, { filename: file.filename });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.source, file.source);
    parsed.descriptor.vapor = vapor;
    const script = compiler.compileScript(parsed.descriptor, {
      id: "probe",
      inlineTemplate: true,
      isProd: input.production,
      templateOptions: { isProd: input.production, compilerOptions: { comments: false } },
    });
    return {
      code: script.content,
      bindings: script.bindings,
      map: script.map,
      output: "official-inline-template",
      filename: file.filename,
      source: file.source,
    };
  }

  function validate(file) {
    assert.equal(file.outputTypeScript, false, "reported public default emits JavaScript");
    assert.deepEqual(file.plain.errors, []);
    assert.deepEqual(file.plain.warnings, []);
    assert.equal(file.plain.css, null);
    assert.deepEqual(file.plain.macroArtifacts, []);
    assert.equal(file.plain.map, null);
    assert.equal(file.mapped.map.version, 3);
    assert.deepEqual(file.mapped.map.sources, [file.filename]);
    assert.deepEqual(file.mapped.map.sourcesContent, [file.source]);
    assert.deepEqual(
      { ...file.mapped, map: null },
      file.plain,
      "mapping cannot alter any public result field",
    );
  }

  async function mounted(component, vapor, capture) {
    assert.equal(Boolean(component.__vapor), vapor);
    const app = (vapor ? vue.createVaporApp : vue.createApp)(component);
    const diagnostics = [];
    app.config.warnHandler = (message) => diagnostics.push(message);
    app.config.errorHandler = (error) => diagnostics.push(String(error));
    const host = window.document.createElement("main");
    window.document.body.append(host);
    capture.frames = [];
    capture.raw = [];
    let button;
    let paragraph;
    let active = false;
    const snapshot = (phase) => {
      capture.raw.push({ phase, html: host.innerHTML, diagnostics: [...diagnostics] });
      capture.frames.push({
        phase,
        tree: observeChildren(host),
        sameButton: Boolean(button && host.querySelector("button") === button),
        sameParagraph: Boolean(paragraph && host.querySelector("p") === paragraph),
        diagnostics: [...diagnostics],
      });
    };
    try {
      active = true;
      app.mount(host);
      await vue.nextTick();
      button = host.querySelector("button");
      paragraph = host.querySelector("p");
      snapshot("mount");
      assert.ok(button && paragraph, "whole original child and parent roots mounted");
      for (const phase of ["click", "click-again"]) {
        button.click();
        await vue.nextTick();
        snapshot(phase);
      }
      app.unmount();
      active = false;
      await vue.nextTick();
      snapshot("unmount");
      assert.deepEqual(capture.frames, expected.frames);
      assert.equal(host.childNodes.length, 0);
      assert.deepEqual(diagnostics, []);
    } finally {
      if (active) {
        capture.failureTreeBeforeCleanup = observeChildren(host);
        app.unmount();
      }
      host.remove();
    }
  }

  for (const fixture of input.cases) {
    const [appName, childName, vapor] = profiles[fixture.name];
    const row = {
      name: fixture.name,
      mode: fixture.mode,
      vapor,
      production: input.production,
      actual: { child: fixture.child, app: fixture.app },
      reference: {},
      observations: {},
      status: "RUNNING",
    };
    evidence.rows.push(row);
    evidence.currentCase = { name: fixture.name, mode: fixture.mode };
    evidence.stage = "validate-originals";
    assert.equal(fixture.vapor, vapor);
    assert.equal(fixture.app.source, originals.get(appName));
    assert.equal(fixture.child.source, originals.get(childName));
    assert.equal(fixture.app.filename, "App.vue");
    assert.equal(fixture.child.filename, "Child.vue");
    for (const file of [fixture.child, fixture.app]) validate(file);
    evidence.stage = "official-compile";
    row.reference.child = reference(fixture.child, vapor);
    row.reference.app = reference(fixture.app, vapor);
    for (const arm of ["reference", "actual"]) {
      evidence.stage = `load-${arm}`;
      const graph =
        arm === "actual" ? { child: fixture.child.plain, app: fixture.app.plain } : row.reference;
      const capture = { childModule: {}, appModule: {} };
      row.observations[arm] = capture;
      const child = await evaluate(graph.child.code, undefined, capture.childModule);
      const component = await evaluate(graph.app.code, child, capture.appModule);
      evidence.stage = `mount-${arm}`;
      await mounted(component, vapor, capture);
    }
    assert.deepEqual(row.observations.actual.frames, row.observations.reference.frames);
    row.status = "PASS";
  }
  evidence.stage = "complete";
  evidence.currentCase = null;
  evidence.nativeLevelCredit = false;
  evidence.ssrHydrationCredit = false;
} catch (error) {
  evidence.error = { name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  delete globalThis.__componentListenerModules;
  try {
    if (window) await window.happyDOM.close();
  } catch (error) {
    evidence.cleanupError = String(error);
    process.exitCode = 1;
  }
  await new Promise((resolve, reject) =>
    process.stdout.write(JSON.stringify(evidence), (error) => (error ? reject(error) : resolve())),
  );
}
function hash(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
