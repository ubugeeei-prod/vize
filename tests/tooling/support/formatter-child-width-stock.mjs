import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { compileFunction } from "node:vm";
import { pathToFileURL } from "node:url";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const serializeError = (error) =>
  typeof error === "string"
    ? error
    : { code: error.code ?? null, message: error.message, loc: error.loc ?? null };

// These are the four independently authored states sealed in the reference.
export const childWidthStockStates = [
  { id: "hidden", state: { visible: false, value: "A&<", selectedItems: ["a", "b"] } },
  { id: "shown", state: { visible: true, value: "A&<", selectedItems: ["a", "b"] } },
  { id: "unicode", state: { visible: true, value: "状態", selectedItems: ["状態", "β"] } },
  { id: "empty", state: { visible: true, value: "", selectedItems: [] } },
];

// Pass the checkout root. Dependencies resolve through the existing UI package,
// including its coherent stock Vue/compiler/renderer family, on macOS or CI.
export async function createChildWidthStockObserver(repositoryRoot) {
  const uiRequire = createRequire(path.join(repositoryRoot, "npm/ui/package.json"));
  const vueEntry = uiRequire.resolve("vue");
  const vueRequire = createRequire(vueEntry);
  const rendererEntry = uiRequire.resolve("vue/server-renderer");
  const rendererRequire = createRequire(rendererEntry);
  const entries = {
    vue: { require: vueRequire, name: "vue", packageName: "vue" },
    compilerDom: {
      require: vueRequire,
      name: "@vue/compiler-dom",
      packageName: "@vue/compiler-dom",
    },
    compilerSfc: {
      require: uiRequire,
      name: "vue/compiler-sfc",
      packageName: "@vue/compiler-sfc",
    },
    compilerSsr: {
      require: rendererRequire,
      name: "@vue/compiler-ssr",
      packageName: "@vue/compiler-ssr",
    },
    renderer: {
      require: uiRequire,
      name: "vue/server-renderer",
      packageName: "@vue/server-renderer",
    },
  };
  const identities = Object.fromEntries(
    Object.entries(entries).map(([id, entry]) => {
      const resolved = entry.require.resolve(entry.name);
      const packageRequire = id === "compilerSfc" ? vueRequire : createRequire(resolved);
      const version = packageRequire(`${entry.packageName}/package.json`).version;
      assert.equal(version, "3.5.35", `${id}: stock provider version`);
      return [
        id,
        {
          entry: resolved,
          realEntry: fs.realpathSync(resolved),
          version,
          entrySha256: sha256(fs.readFileSync(resolved)),
        },
      ];
    }),
  );
  assert.equal(fs.realpathSync(rendererRequire.resolve("vue")), identities.vue.realEntry);
  const happyEntry = uiRequire.resolve("happy-dom");
  const happyManifest = JSON.parse(
    fs.readFileSync(new URL("../package.json", pathToFileURL(happyEntry))),
  );
  assert.equal(happyManifest.name, "happy-dom");
  assert.equal(happyManifest.version, "20.11.2");
  identities.happyDom = {
    entry: happyEntry,
    realEntry: fs.realpathSync(happyEntry),
    version: happyManifest.version,
    entrySha256: sha256(fs.readFileSync(happyEntry)),
  };
  const { Window } = await import(pathToFileURL(happyEntry));
  const window = new Window();
  for (const name of ["window", "document", "Element", "HTMLElement", "SVGElement", "Node"])
    Object.defineProperty(globalThis, name, {
      configurable: true,
      value: name === "window" ? window : window[name],
    });
  // DOM globals must exist before runtime-dom captures its document.
  const Vue = vueRequire(identities.vue.entry);
  const { renderToString } = rendererRequire(identities.renderer.entry);
  const { compile } = vueRequire(identities.compilerDom.entry);
  const { compile: compileSsr } = rendererRequire(identities.compilerSsr.entry);
  const { parse } = uiRequire(identities.compilerSfc.entry);
  assert.equal(Vue.version, "3.5.35");

  function templateOf(source, id) {
    const parsed = parse(source, { filename: `${id}.vue` });
    const errors = parsed.errors.map(serializeError);
    assert.ok(parsed.descriptor.template, `${id}: complete SFC has a template`);
    return { template: parsed.descriptor.template.content, errors };
  }
  function compileTemplatePacket(template, errors) {
    const options = { mode: "function", onError: (error) => errors.push(serializeError(error)) };
    const dom = compile(template, options);
    const ssr = compileSsr(template, options);
    return {
      template,
      templateSha256: sha256(template),
      parseErrors: errors,
      dom: { code: dom.code, codeSha256: sha256(dom.code) },
      ssr: { code: ssr.code, codeSha256: sha256(ssr.code) },
    };
  }
  // Actual execution always compiles the descriptor's complete original bytes,
  // including its first LF/CRLF. No source or compiler code is normalized.
  function compilePacket(source, id = "case") {
    const { template, errors } = templateOf(source, id);
    return compileTemplatePacket(template, errors);
  }
  // Separate provenance check for the immutable author's declared recipe. This
  // packet is never used to load the DOM or SSR functions executed below.
  function compileAuthorPacket(source, id) {
    const { template, errors } = templateOf(source, id);
    return compileTemplatePacket(template.replace(/^\r?\n/, ""), errors);
  }

  // The real reporter component was unavailable. This exact deterministic
  // observer is declared in the sealed reference, never treated as that module.
  const observer = {
    name: "IndependentSlotObserver",
    emits: ["click"],
    render() {
      return Vue.h("button", { onClick: () => this.$emit("click") }, this.$slots.default?.());
    },
  };
  async function observePacket(packet, state, id = "case") {
    assert.deepEqual(packet.parseErrors, [], `${id}: complete stock parse diagnostics`);
    const render = compileFunction(packet.dom.code, ["Vue"])(Vue);
    // Match Vue's own runtime-compiled flag for this mode:function output.
    render._rc = true;
    const ssrRender = compileFunction(packet.ssr.code, ["require"])((name) => {
      assert.ok(["vue", "vue/server-renderer"].includes(name), `${id}: stock SSR import`);
      return name === "vue" ? Vue : rendererRequire(identities.renderer.entry);
    });
    const parent = {
      emits: ["submit"],
      components: { MyButton: observer, VeryLongComponentName: observer },
      data: () => structuredClone(state),
      methods: { describeEverySelectedItemInTheList: (items) => items.join(", ") },
      render,
    };
    const host = document.createElement("div");
    document.body.append(host);
    let submits = 0;
    const app = Vue.createApp(parent, { onSubmit: () => submits++ });
    const warnings = [];
    app.config.globalProperties.$t = (key) => `translated:${key}`;
    app.config.warnHandler = (message) => warnings.push(message);
    const result = {
      domHtml: null,
      textContent: null,
      submitsBefore: null,
      submitsAfter: null,
      ssrHtml: null,
      warnings,
    };
    let mounted = false;
    try {
      app.mount(host);
      mounted = true;
      await Vue.nextTick();
      result.domHtml = host.innerHTML;
      result.textContent = host.textContent;
      result.submitsBefore = submits;
      host.querySelector("button")?.click();
      await Vue.nextTick();
      result.submitsAfter = submits;
    } finally {
      if (mounted) app.unmount();
      host.remove();
    }
    const ssrApp = Vue.createSSRApp({ ...parent, render: undefined, ssrRender });
    ssrApp.config.globalProperties.$t = (key) => `translated:${key}`;
    ssrApp.config.warnHandler = (message) => warnings.push(message);
    result.ssrHtml = await renderToString(ssrApp);
    assert.deepEqual(warnings, [], `${id}: stock observer warnings`);
    return result;
  }

  // reference is an immutable row from sole-child-width-independent.json.
  // phase is original for initial source, expected for every successful write.
  // Persist captures before judging each full packet; failure state is retained.
  async function qualify(source, reference, phase, observed, persist = () => {}) {
    assert.ok(["original", "expected"].includes(phase));
    observed.identities = identities;
    observed.source = source;
    observed.sourceSha256 = sha256(source);
    observed.packet = compilePacket(source, reference.id);
    observed.authorRecipe = {
      name: "sealed author capture excludes the first template LF/CRLF",
      execution: false,
      packet: compileAuthorPacket(source, reference.id),
    };
    observed.runtime = [];
    persist();
    assert.equal(
      source,
      phase === "original" ? reference.input : reference.expected,
      `${reference.id}: complete sealed ${phase} SFC bytes`,
    );
    assert.deepEqual(
      observed.authorRecipe.packet,
      reference.stock[phase],
      `${reference.id}: exact sealed ${phase} author packet provenance`,
    );
    assert.deepEqual(
      observed.packet.parseErrors,
      [],
      `${reference.id}: full-content SFC/DOM/SSR diagnostic packet`,
    );
    // The complete content's generated functions must independently remain
    // byte-identical to the sealed author functions. Only complete-content
    // functions, retained in observed.packet, are used for actual execution.
    assert.deepEqual(
      observed.packet.dom,
      reference.stock[phase].dom,
      `${reference.id}: full-content DOM compiler bytes`,
    );
    assert.deepEqual(
      observed.packet.ssr,
      reference.stock[phase].ssr,
      `${reference.id}: full-content SSR compiler bytes`,
    );
    for (const control of reference.stock.runtime) {
      const row = { id: control.id, state: control.state, result: null, error: null };
      observed.runtime.push(row);
      persist();
      try {
        row.result = await observePacket(observed.packet, control.state, reference.id);
        persist();
        assert.deepEqual(
          row.result,
          control[phase],
          `${reference.id}: complete sealed ${phase}/${control.id} stock state`,
        );
      } catch (error) {
        row.error = { message: error.message, stack: error.stack };
        persist();
        throw error;
      }
    }
    assert.equal(observed.runtime.length, 4);
    observed.qualified = true;
    persist();
    return observed;
  }
  return { identities, compilePacket, observePacket, qualify };
}
