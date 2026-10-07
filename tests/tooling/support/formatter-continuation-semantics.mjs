import fs from "node:fs";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";
const sha = (value) => createHash("sha256").update(value).digest("hex");
const uiRequire = createRequire(
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url)),
);
const vueEntry = uiRequire.resolve("vue");
const vueRequire = createRequire(vueEntry);
const rendererEntry = uiRequire.resolve("vue/server-renderer");
const rendererRequire = createRequire(rendererEntry);
const packageEntries = {
  vue: [vueRequire, "vue", "vue"],
  compilerDom: [vueRequire, "@vue/compiler-dom", "@vue/compiler-dom"],
  compilerSfc: [uiRequire, "vue/compiler-sfc", "@vue/compiler-sfc"],
  compilerSsr: [rendererRequire, "@vue/compiler-ssr", "@vue/compiler-ssr"],
  renderer: [uiRequire, "vue/server-renderer", "@vue/server-renderer"],
};
const identities = Object.fromEntries(
  Object.entries(packageEntries).map(([id, [require, name, packageName]]) => {
    const entry = require.resolve(name);
    const packageRequire = id === "compilerSfc" ? vueRequire : createRequire(entry);
    const version = packageRequire(`${packageName}/package.json`).version;
    assert.equal(version, "3.5.35");
    return [
      id,
      {
        entry,
        realEntry: fs.realpathSync(entry),
        version,
        entrySha256: sha(fs.readFileSync(entry)),
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
  entrySha256: sha(fs.readFileSync(happyEntry)),
};
const { Window } = await import(pathToFileURL(happyEntry));
const window = new Window();
for (const name of [
  "window",
  "document",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Node",
  "Event",
  "MouseEvent",
  "navigator",
]) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    value: name === "window" ? window : window[name],
  });
}
const req = vueRequire;
const Vue = req(identities.vue.entry);
const { compile } = req(identities.compilerDom.entry);
const { compile: compileSsr } = req(identities.compilerSsr.entry);
const { parse } = req(identities.compilerSfc.entry);
const { renderToString } = req(identities.renderer.entry);
assert.equal(Vue.version, "3.5.35");

const MyList = Vue.defineComponent({
  name: "MyList",
  props: ["itemsGroupedByPageForTheCurrentView", "selectedIndex"],
  emits: ["click"],
  setup(props, { emit }) {
    return () =>
      Vue.h(
        "button",
        {
          type: "button",
          "data-pages": JSON.stringify(props.itemsGroupedByPageForTheCurrentView),
          "data-selected": String(props.selectedIndex),
          onClick: () => emit("click"),
        },
        "MyList",
      );
  },
});
function templateOf(source, id) {
  const parsed = parse(source, { filename: id + ".vue" });
  assert.deepEqual(parsed.errors, []);
  assert.ok(parsed.descriptor.template);
  return parsed.descriptor.template.content;
}
function makeContext(state, onClick) {
  return {
    settingsPanelState: {
      pagination: { pages: state.pages },
      selectedPageIndex: { value: state.page },
    },
    state: { pages: state.pages, page: state.page },
    pages: state.pages,
    page: state.page,
    visible: state.page > 0,
    n: state.pages.length,
    openSettingsDialogForTheCurrentlySelectedItem: onClick,
  };
}
export async function observeSource(source, id, states) {
  const template = templateOf(source, id);
  const domCode = compile(template, {
    mode: "function",
    whitespace: "condense",
    prefixIdentifiers: true,
  }).code;
  const ssrCode = compileSsr(template, { mode: "function", whitespace: "condense" }).code;
  const render = new Function("Vue", domCode)(Vue);
  const ssrRender = new Function("require", ssrCode)((name) => {
    assert.ok(["vue", "vue/server-renderer"].includes(name), name);
    return name === "vue" ? Vue : req(identities.renderer.entry);
  });
  let clicks = 0;
  const ctx = Vue.reactive(
    makeContext(states[0], () => {
      clicks++;
    }),
  );
  const app = Vue.createApp({ render, setup: () => ctx });
  app.component("MyList", MyList);
  const host = document.createElement("div");
  document.body.appendChild(host);
  app.mount(host);
  const observations = [];
  for (const state of states) {
    Object.assign(
      ctx,
      makeContext(state, () => {
        clicks++;
      }),
    );
    await Vue.nextTick();
    const button = host.querySelector("button");
    const before = clicks;
    if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    const ssrApp = Vue.createSSRApp({ ssrRender, setup: () => makeContext(state, () => {}) });
    ssrApp.component("MyList", MyList);
    const ssr = await renderToString(ssrApp);
    observations.push({
      state,
      domHtml: host.innerHTML,
      domText: host.textContent,
      ssrHtml: ssr,
      emittedClickCalls: clicks - before,
    });
  }
  app.unmount();
  host.remove();
  return {
    id,
    sourceSha256: sha(source),
    templateSha256: sha(template),
    domCode,
    ssrCode,
    domCodeSha256: sha(domCode),
    ssrCodeSha256: sha(ssrCode),
    observations,
  };
}

export { identities as providerIdentities };
