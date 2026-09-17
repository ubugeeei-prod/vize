import { expect, it } from "vite-plus/test";
import { createVaporApp, nextTick, reactive } from "vue";
import ScopedNamespaces from "./fixtures/ScopedNamespaces.vue";

it("preserves scoped HTML, SVG and MathML namespaces in Chromium", async () => {
  // Happy DOM's template parser assigns HTML namespaces to MathML elements.
  const host = document.createElement("div");
  document.body.append(host);
  const state = reactive({ label: "initial" });
  const diagnostics: string[] = [];
  expect(ScopedNamespaces).toHaveProperty("__vapor", true);
  const app = createVaporApp(ScopedNamespaces, state);
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const expected = [
    ["section", "http://www.w3.org/1999/xhtml"],
    ["svg", "http://www.w3.org/2000/svg"],
    ["g", "http://www.w3.org/2000/svg"],
    ["circle", "http://www.w3.org/2000/svg"],
    ["math", "http://www.w3.org/1998/Math/MathML"],
    ["mrow", "http://www.w3.org/1998/Math/MathML"],
    ["mi", "http://www.w3.org/1998/Math/MathML"],
    ["textarea", "http://www.w3.org/1999/xhtml"],
  ];
  try {
    app.mount(host);
    await nextTick();
    const scope = host
      .firstElementChild!.getAttributeNames()
      .find((name) => name.startsWith("data-v-"));
    expect(scope).toBeDefined();
    for (const label of ["initial", "updated"]) {
      state.label = label;
      await nextTick();
      const nodes = [...host.querySelectorAll("*")];
      expect(nodes.map((node) => [node.localName, node.namespaceURI])).toEqual(expected);
      expect(nodes.every((node) => node.hasAttribute(scope!))).toBe(true);
      expect(host.querySelector("mi")!.textContent).toBe(label);
      expect(host.querySelector("textarea")!.value).toBe("<span>");
      expect(diagnostics).toEqual([]);
    }
    app.unmount();
    expect(host.childNodes.length).toBe(0);
    expect(diagnostics).toEqual([]);
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
  }
});
