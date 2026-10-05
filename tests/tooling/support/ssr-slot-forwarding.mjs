// #7890: unchanged Forward.vue, complete default components and real slot branches.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { Window } from "happy-dom";
import { loadRuntime } from "./davinci-mounted-trace.mjs";

assert.equal(process.env.NODE_ENV, "production");
const window = new Window();
for (const key of [
  "window",
  "document",
  "Document",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Node",
  "Text",
  "Comment",
  "Event",
  "ShadowRoot",
])
  globalThis[key] = key === "window" ? window : window[key];
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const compiler = fromVue("vue/compiler-sfc");
const vue = fromVue("vue");
const server = fromVue("vue/server-renderer");
const vapor = await loadRuntime({ production: true });
const pluginFactory = (await import(pathToFileURL(fromUi.resolve("@vitejs/plugin-vue")).href))
  .default;
const { transformSync } = fromUi("@babel/core");
assert.equal(compiler.version, "3.6.0-rc.9");
assert.equal(vue.version, compiler.version);
assert.equal(vapor.version, compiler.version);
assert.equal(fromVue("@vue/server-renderer/package.json").version, compiler.version);
assert.equal(fromUi("@vitejs/plugin-vue/package.json").version, "6.0.7");
const root = new URL("../../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(readFileSync(new URL("ssr-slot-forwarding.manifest.json", root)));
const pinned = manifest.cases[0];
for (const file of [...pinned.inputs.files, pinned.reference])
  assert.equal(
    createHash("sha256")
      .update(readFileSync(new URL(file.path, root)))
      .digest("hex"),
    file.sha256,
    file.path,
  );
const expected = JSON.parse(readFileSync(new URL(pinned.reference.path, root)));
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const helperId = "\0plugin-vue:export-helper";
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
let sequence = 0;
const context = {
  error(error) {
    throw new Error(typeof error === "string" ? error : error.message);
  },
  warn(warning) {
    throw new Error(typeof warning === "string" ? warning : warning.message);
  },
  addWatchFile() {},
};

async function reference(source, name, target) {
  const selected = {
    ...compiler,
    parse(...args) {
      const result = compiler.parse(...args);
      assert.equal(result.descriptor.source, args[0]);
      // compiler-sfc caches descriptors: select this target on an owned copy.
      return { ...result, descriptor: { ...result.descriptor, vapor: target === "vapor" } };
    },
  };
  const plugin = pluginFactory({ compiler: selected });
  plugin.configResolved({
    root: "/slot-forwarding",
    command: "build",
    isProduction: true,
    build: { sourcemap: true },
    define: {},
    logger: { warn: (warning) => context.warn(warning) },
  });
  plugin.buildStart.call(context);
  const result = await plugin.transform.handler.call(
    context,
    source,
    `/slot-forwarding/${name}.vue`,
    { ssr: target.endsWith("ssr") },
  );
  assert.equal(typeof result?.code, "string");
  const helper = plugin.load.handler.call(context, helperId, {});
  assert.equal(typeof helper, "string");
  return { code: result.code, helper };
}

async function evaluate(code, runtime, dependencies, helper) {
  const modules = {
    vue: runtime,
    "vue/vapor": runtime,
    "vue/server-renderer": server,
    "@vue/server-renderer": server,
    ...dependencies,
  };
  if (helper) modules[helperId] = await import(dataUrl(helper));
  const slot = `__slotForwardModules${sequence++}`;
  globalThis[slot] = modules;
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
  const module = await import(dataUrl(`${result.code}\n// whole component ${sequence++}`));
  assert.ok(
    module.default && typeof module.default === "object",
    "authored whole-SFC default component",
  );
  delete globalThis[slot];
  return module.default;
}

async function observe(target, variant, mode, useActual) {
  const runtime = target === "vapor" ? vapor : vue;
  const serverTarget = target.endsWith("ssr");
  const innerName = variant === "reported" ? "Inner" : "InnerNamed";
  const originals = input.cases.find((item) => item.target === target).files;
  const controlInner = mode === "vnode" || target === "dom";
  const components = {};
  const codes = {};
  for (const name of [innerName, "Forward", "App"]) {
    const fixture = originals.find((file) => file.name === name);
    assert.equal(
      fixture.source,
      readFileSync(new URL(`ssr-slot-forwarding/${name}.vue.txt`, root), "utf8"),
    );
    const stock = await reference(
      fixture.source,
      name,
      name === innerName && controlInner ? "dom" : target,
    );
    const selected =
      useActual && !(name === innerName && controlInner) ? { code: fixture.current.code } : stock;
    const dependencies = {
      "./Inner.vue": { default: components[innerName] },
      "./Forward.vue": { default: components.Forward },
    };
    components[name] = await evaluate(selected.code, runtime, dependencies, selected.helper);
    codes[name] = selected.code;
    if (name !== innerName) assert.equal(typeof components[name].setup, "function");
  }
  const app = (
    target === "vapor"
      ? runtime.createVaporApp
      : serverTarget
        ? runtime.createSSRApp
        : runtime.createApp
  )(components.App);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  let html;
  if (serverTarget) html = await server.renderToString(app);
  else {
    const host = document.createElement("div");
    document.body.append(host);
    try {
      app.mount(host);
      await runtime.nextTick();
      html = host.innerHTML;
    } finally {
      app.unmount();
      host.remove();
    }
  }
  assert.deepEqual(diagnostics, []);
  const decoded = document.createElement("div");
  decoded.innerHTML = html;
  const rendered = [...decoded.querySelectorAll("div > b, div > i")]
    .map((node) => node.outerHTML)
    .join("");
  const wholeDom = `<div>${rendered}</div>`;
  assert.equal(wholeDom, expected[`${variant}Dom`]);
  return {
    html,
    diagnostics,
    codes,
    controlInner: controlInner ? "whole-stock-original-dom-default" : null,
  };
}

const observations = [];
try {
  for (const fixture of input.cases) {
    for (const variant of ["reported", "named"]) {
      for (const mode of fixture.target.endsWith("ssr") ? ["push", "vnode"] : ["mounted"]) {
        const reference = await observe(fixture.target, variant, mode, false);
        const actual = await observe(fixture.target, variant, mode, true);
        assert.equal(
          actual.html,
          reference.html,
          `${fixture.target}/${variant}/${mode}: complete HTML`,
        );
        observations.push({ target: fixture.target, variant, mode, reference, actual });
      }
    }
  }
  process.stdout.write(
    JSON.stringify({ versions: { vue: vue.version, plugin: "6.0.7" }, observations }),
  );
} finally {
  await window.happyDOM.close();
}
