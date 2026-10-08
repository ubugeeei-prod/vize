// #7894: execute whole source-built and independent official Vapor SFCs.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { transformSync } from "@babel/core";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";
import { vueVaporVersion } from "./vue-vapor-release.mjs";

const fixtureRoot = new URL("../../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(
  readFileSync(new URL("vapor-trailing-line-comments.manifest.json", fixtureRoot)),
);
const pin = manifest.cases[0];
for (const artifact of [
  ...pin.inputs.files.map((file) => ({ ...file, path: pin.inputs.root + "/" + file.path })),
  pin.reference,
]) {
  const bytes = readFileSync(new URL(artifact.path, fixtureRoot));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), artifact.sha256, artifact.path);
}
const reported = readFileSync(
  new URL("vapor-trailing-line-comments/Reported.vue.txt", fixtureRoot),
  "utf8",
);
const controls = JSON.parse(
  readFileSync(new URL("vapor-trailing-line-comments/cases.json", fixtureRoot)),
);
const expectedSources = new Map([
  ["reported", reported],
  ...controls.map((c) => [c.name, c.source]),
]);
assert.equal(expectedSources.size, 16);

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(typeof input.production, "boolean");
assert.equal(input.cases.length, expectedSources.size);
const window = new Window();
for (const key of [
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
  globalThis[key] = key === "window" ? window : window[key];
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const fromPlugin = createRequire(fromUi.resolve("@vitejs/plugin-vue/package.json"));
const { transformWithOxc } = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
const compiler = fromVue("vue/compiler-sfc");
const vue = await loadRuntime({ production: input.production });
assert.equal(vue.version, "3.6.0-rc.9");
assert.equal(vueVaporVersion, vue.version);
assert.equal(compiler.version, vue.version);
globalThis.__lineCommentModules = { vue };
let sequence = 0;

async function component(code) {
  // Parse the complete module; resolve only real import declarations.
  const transformed = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            assert.equal(path.node.source.value, "vue");
            const declarations = path.node.specifiers.map((specifier) => {
              assert.ok(t.isImportSpecifier(specifier), "only actual named Vue imports");
              assert.ok(Object.hasOwn(vue, specifier.imported.name), specifier.imported.name);
              return t.variableDeclarator(
                specifier.local,
                t.memberExpression(
                  t.memberExpression(
                    t.identifier("globalThis"),
                    t.identifier("__lineCommentModules"),
                  ),
                  t.identifier("vue"),
                ),
              );
            });
            for (let i = 0; i < declarations.length; i++)
              declarations[i].init = t.memberExpression(
                declarations[i].init,
                t.stringLiteral(path.node.specifiers[i].imported.name),
                true,
              );
            path.replaceWith(t.variableDeclaration("const", declarations));
          },
        },
      }),
    ],
  });
  const body = transformed.code + "\n// isolated line-comment module " + sequence++;
  const loaded = await import(
    "data:text/javascript;base64," + Buffer.from(body).toString("base64")
  );
  assert.equal(loaded.default?.__vapor, true, "complete actual default Vapor component");
  return loaded.default;
}

async function observe(code, name) {
  delete globalThis.__lineCommentState;
  const app = vue.createVaporApp(await component(code));
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  try {
    app.mount(host);
    await vue.nextTick();
    const frames = [{ tree: observeChildren(host), diagnostics: [...diagnostics] }];
    const state = globalThis.__lineCommentState;
    if (name === "reported") assert.equal(state, undefined);
    else assert.ok(vue.isReactive(state), "the original setup creates genuine reactive state");
    for (const patch of [
      { ok: false, label: "second", classes: { off: true }, html: "<i>second</i>" },
      { ok: true, label: "last", classes: { last: true }, html: "<em>last</em>" },
    ]) {
      if (state) Object.assign(state, patch);
      if (name === "event-control" || name === "async-spread-regex-control") {
        const buttons = host.querySelectorAll("button");
        assert.equal(buttons.length, 1);
        buttons[0].click();
        if (name === "async-spread-regex-control") {
          assert.ok(state.pending instanceof Promise);
          await state.pending;
          assert.equal(state.label, "/", "the unchanged authored async regex actually executes");
        }
      }
      await vue.nextTick();
      frames.push({ tree: observeChildren(host), diagnostics: [...diagnostics] });
    }
    assert.deepEqual(diagnostics, []);
    app.unmount();
    await vue.nextTick();
    assert.equal(host.childNodes.length, 0, "unmount removes the entire actual component");
    return { frames, afterUnmount: [] };
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
  }
}

const seen = new Set();
const results = [];
try {
  for (const fixture of input.cases) {
    assert.ok(expectedSources.has(fixture.name));
    assert.ok(!seen.has(fixture.name));
    seen.add(fixture.name);
    assert.equal(fixture.source, expectedSources.get(fixture.name), "original complete source");
    const current = fixture.current;
    assert.deepEqual(Object.keys(current).sort(), [
      "bindings",
      "code",
      "css",
      "errors",
      "macroArtifacts",
      "map",
      "warnings",
    ]);
    assert.deepEqual(
      {
        css: current.css,
        map: current.map,
        errors: current.errors,
        warnings: current.warnings,
        macroArtifacts: current.macroArtifacts,
      },
      { css: null, map: null, errors: [], warnings: [], macroArtifacts: [] },
    );
    const parsed = compiler.parse(fixture.source, { filename: "Reported.vue" });
    assert.deepEqual(parsed.errors, []);
    // Vize's public option requests Vapor for the unchanged ordinary template.
    parsed.descriptor.vapor = true;
    const official = compiler.compileScript(parsed.descriptor, {
      id: "probe",
      inlineTemplate: true,
      isProd: input.production,
      templateOptions: { isProd: input.production, compilerOptions: { comments: false } },
    });
    assert.deepEqual(
      current.bindings.bindings,
      JSON.parse(JSON.stringify(official.bindings)),
      "complete serialized setup binding map",
    );
    assert.deepEqual(current.bindings.propsAliases, {});
    assert.equal(current.bindings.isScriptSetup, true);
    const officialJs =
      parsed.descriptor.scriptSetup.lang === "ts"
        ? (
            await transformWithOxc(official.content, "Reported.vue.ts", {
              lang: "ts",
              sourcemap: false,
            })
          ).code
        : official.content;
    const expected = await observe(officialJs, fixture.name);
    const actual = await observe(current.code, fixture.name);
    assert.deepEqual(actual, expected, fixture.name + "/production=" + input.production);
    results.push({
      name: fixture.name,
      current,
      officialCode: official.content,
      officialJs,
      expected,
      actual,
    });
  }
  assert.deepEqual(seen, new Set(expectedSources.keys()));
  process.stdout.write(
    JSON.stringify({ production: input.production, version: vue.version, results }),
  );
} finally {
  delete globalThis.__lineCommentState;
  delete globalThis.__lineCommentModules;
  await window.happyDOM.close();
}
