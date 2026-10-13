import { expect, it } from "vite-plus/test";
import { createApp, createVaporApp, nextTick } from "virtual:vize-svg-namespace-8624/runtime";
import Vapor, { emittedSha256 as vaporHash } from "virtual:vize-svg-namespace-8624/vapor";
import Vdom, { emittedSha256 as vdomHash } from "virtual:vize-svg-namespace-8624/vdom";
import { state as vaporState } from "virtual:vize-svg-namespace-8624/state-vapor";
import { state as vdomState } from "virtual:vize-svg-namespace-8624/state-vdom";
import row, { custody } from "virtual:vize-svg-namespace-8624/controls";

type Tree = string | { tag: string; attributes: Record<string, string>; children: Tree[] };
function tree(parent: Node): Tree[] {
  const children: Tree[] = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === Node.TEXT_NODE && node.textContent) {
      const previous = children.at(-1);
      if (typeof previous === "string") children[children.length - 1] = previous + node.textContent;
      else children.push(node.textContent);
    } else if (node instanceof Element) {
      children.push({
        tag: node.localName,
        attributes: Object.fromEntries(
          [...node.attributes].map(({ name, value }) => [name, value]),
        ),
        children: tree(node),
      });
    }
  }
  return children;
}

it("executes the whole inherited SVG corpus against independent VDOM in Chromium", async () => {
  expect(custody).toMatchObject({
    runtimeVersion: "3.6.0-rc.9",
    productProvider: "fresh-source-napi",
    oracleProvider: "independent-stock-vdom",
    publicationQualification: false,
  });
  expect(custody.sourceSha256).toBe(
    "3302b4bdfbe306a97822e360314ea018aca6cb3e95de85426d30d9ec3baf3abe",
  );
  expect(custody.controlsSha256).toBe(
    "de63dc5c776f6286350015b64524e17ef9d91af77446e3e6f46af21884e36b01",
  );
  expect(vaporHash).toMatch(/^[a-f0-9]{64}$/u);
  expect(vdomHash).toMatch(/^[a-f0-9]{64}$/u);
  expect(vaporHash).not.toBe(vdomHash);
  expect(custody.checkout).toMatch(/^[a-f0-9]{40}$/u);
  expect(custody.bindingSha256).toMatch(/^[a-f0-9]{64}$/u);
  console.info(
    "namespace-8624 source custody",
    JSON.stringify({ ...custody, vaporHash, vdomHash }),
  );
  expect(Vapor).toHaveProperty("__vapor", true);
  expect(Vdom).not.toHaveProperty("__vapor", true);
  const traces = [];
  for (const backend of ["vapor", "vdom"] as const) {
    const host = document.createElement("div");
    document.body.append(host);
    const state = backend === "vapor" ? vaporState : vdomState;
    Object.assign(state, structuredClone(row.context));
    const app = backend === "vapor" ? createVaporApp(Vapor) : createApp(Vdom);
    const diagnostics: string[] = [];
    app.config.warnHandler = (message) => diagnostics.push(message);
    app.config.errorHandler = (error) => diagnostics.push(String(error));
    const trace = [];
    const observe = (phase: number) => {
      const nodes = row.boundIds.map((id) => {
        const node = host.querySelector<HTMLElement | SVGElement>(`#${id}`);
        expect(node, id).not.toBeNull();
        return node!;
      });
      const snapshot = {
        tree: tree(host),
        namespaces: [...host.querySelectorAll("*")].map((node) => node.namespaceURI),
        classes: nodes.map((node) => node.getAttribute("class")),
        colors: phase < 2 ? nodes.map((node) => getComputedStyle(node).color) : [],
        classNameKinds: nodes.map((node) =>
          node.className instanceof SVGAnimatedString ? "SVGAnimatedString" : typeof node.className,
        ),
      };
      expect(snapshot.tree).toEqual(row.expectedTrees[phase]);
      expect(snapshot.namespaces).toEqual(row.expectedNamespacePhases[phase]);
      expect(snapshot.classes).toEqual(row.expectedClassPhases[phase]);
      expect(snapshot.classNameKinds).toEqual([
        ...Array(5).fill("SVGAnimatedString"),
        "string",
        "string",
      ]);
      if (phase < 2) expect(snapshot.colors).toEqual(row.expectedColorPhases[phase]);
      else
        for (const node of nodes) {
          expect(node.hasAttribute("style")).toBe(false);
          expect(node.style.color).toBe("");
        }
      expect(diagnostics).toEqual([]);
      trace.push(snapshot);
    };
    try {
      app.mount(host);
      await nextTick();
      observe(0);
      expect(host.querySelector("svg")!.getBoundingClientRect().toJSON()).toMatchObject({
        width: 64,
        height: 64,
      });
      expect(host.querySelector("rect")!.getBoundingClientRect().toJSON()).toMatchObject({
        width: 32,
        height: 32,
      });
      for (const [index, step] of row.steps.entries()) {
        Object.assign(state, structuredClone(step.patch));
        await nextTick();
        observe(index + 1);
      }
      const retained = row.boundIds.map((id) => host.querySelector(`#${id}`)!);
      const settled = retained.map((node) => node.outerHTML);
      app.unmount();
      await nextTick();
      expect(host.childNodes.length).toBe(0);
      expect(tree(host)).toEqual(row.expectedTrees[3]);
      expect([...host.querySelectorAll("*")]).toEqual(row.expectedNamespacePhases[3]);
      Object.assign(state, structuredClone(row.context));
      await nextTick();
      expect(retained.map((node) => node.outerHTML)).toEqual(settled);
      expect(diagnostics).toEqual([]);
      trace.push({ tree: tree(host), namespaces: [], classes: [], colors: [], classNameKinds: [] });
      traces.push(trace);
    } finally {
      if (host.childNodes.length) app.unmount();
      host.remove();
    }
  }
  expect(traces[0]).toEqual(traces[1]);
});
