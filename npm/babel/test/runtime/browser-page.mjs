import * as Vue from "vue";

const input = globalThis.__vizeRuntimeInput;
const check = (value, message) => {
  if (!value) throw new Error(message);
};
function allNodes(host) {
  const walker = document.createTreeWalker(host, NodeFilter.SHOW_ALL),
    result = [];
  for (let node = walker.nextNode(); node; node = walker.nextNode()) result.push(node);
  return result;
}
function tree(node) {
  const result = { nodeType: node.nodeType, nodeName: node.nodeName };
  if (node.nodeType === Node.ELEMENT_NODE) {
    result.namespace = node.namespaceURI;
    result.attributes = [...node.attributes]
      .map((attribute) => [attribute.name, attribute.value])
      .sort(([a], [b]) => a.localeCompare(b));
    if (node instanceof HTMLButtonElement) {
      result.disabled = node.disabled;
      result.style = { color: node.style.color, fontWeight: node.style.fontWeight };
    }
  }
  if (node.nodeType === Node.TEXT_NODE || node.nodeType === Node.COMMENT_NODE)
    result.data = node.data;
  result.children = [...node.childNodes].map(tree);
  return result;
}
async function boot() {
  check(Vue.version === input.version, "actual browser Vue version");
  globalThis.__vizeBabelRuntime = {
    setup: { App: 0, Counter: 0 },
    childEvents: 0,
    parentEvents: 0,
    events: [],
  };
  const module = await import(input.module);
  check(module.App && typeof module.App === "object", "actual block component object required");
  const host = document.querySelector("#app");
  if (input.mode === "hydrate") host.innerHTML = input.html;
  const before = input.mode === "hydrate" ? allNodes(host) : [];
  const beforeHtml = host.innerHTML,
    beforeTree = [...host.childNodes].map(tree);
  const create =
    input.target === "vapor"
      ? input.mode === "hydrate"
        ? Vue.createVaporSSRApp
        : Vue.createVaporApp
      : input.mode === "hydrate"
        ? Vue.createSSRApp
        : Vue.createApp;
  check(typeof create === "function", "the selected real mount/hydration API exists");
  const diagnostics = { warnings: [], errors: [] };
  const app = create(module.App);
  app.config.warnHandler = (message) => diagnostics.warnings.push(message);
  app.config.errorHandler = (error) =>
    diagnostics.errors.push({ name: error.name, message: error.message, stack: error.stack });
  app.mount(host);
  await Vue.nextTick();
  const after = allNodes(host);
  const hydration = {
    mode: input.mode,
    beforeHtml,
    beforeTree,
    originalCount: before.length,
    afterCount: after.length,
    retainedInOrder: before.map((node, index) => node === after[index]),
    sameCount: before.length === after.length,
  };
  globalThis.__vizeHydrationObservation = hydration;
  if (input.mode === "hydrate") {
    check(before.length > 0, "hydration starts with genuine server nodes");
    check(
      hydration.sameCount && hydration.retainedInOrder.every(Boolean),
      "every original server element/comment/text node is adopted in place",
    );
  }
  check(
    typeof module.controls.update === "function" && typeof module.controls.restore === "function",
    "actual authored update/restore controls",
  );
  const snapshot = async () => {
    await Vue.nextTick();
    return {
      html: host.innerHTML,
      tree: [...host.childNodes].map(tree),
      metrics: structuredClone(globalThis.__vizeBabelRuntime),
      diagnostics: structuredClone(diagnostics),
    };
  };
  globalThis.__vizeRuntime = {
    ready: true,
    version: Vue.version,
    hydration,
    snapshot,
    update: async () => {
      module.controls.update();
      await Vue.nextTick();
    },
    restore: async () => {
      module.controls.restore();
      await Vue.nextTick();
    },
    dispose: () => {
      app.unmount();
      return { html: host.innerHTML, diagnostics: structuredClone(diagnostics) };
    },
  };
}
boot().catch((error) => {
  globalThis.__vizeRuntimeFailure = {
    name: error.name,
    message: error.message,
    stack: error.stack,
  };
});
