import * as Vue from "vue";
import type { StyleDiagnostics, StyleDomTree, StyleRuntimeModule } from "./types.ts";
const input = globalThis.__styleGuardInput;
const diagnostics: StyleDiagnostics = { warnings: [], errors: [] };
const metrics = (globalThis.__styleGuardMetrics = { setup: 0, clicks: 0, events: [] });
try {
  const module: StyleRuntimeModule = await import(input.module);
  const app = Vue.createVaporApp(module.App);
  app.config.warnHandler = (message, _instance, trace) =>
    diagnostics.warnings.push({ message, trace });
  app.config.errorHandler = (error, _instance, info) =>
    diagnostics.errors.push({
      name: (error as Error).name,
      message: (error as Error).message,
      info,
    });
  const host = document.querySelector<HTMLElement>("#app")!;
  app.mount(host);
  await Vue.nextTick();
  const allNodes = (node: Node): Node[] =>
    [...node.childNodes].flatMap((child) => [child, ...allNodes(child)]);
  const original = allNodes(host);
  const tree = (node: Node): StyleDomTree => ({
    nodeType: node.nodeType,
    nodeName: node.nodeName,
    data: node.nodeType === 3 || node.nodeType === 8 ? (node as CharacterData).data : null,
    attributes:
      node.nodeType === 1
        ? [...(node as Element).attributes]
            .map((a): [string, string] => [a.name, a.value])
            .sort((a, b) => a[0].localeCompare(b[0]))
        : [],
    children: [...node.childNodes].map(tree),
  });
  globalThis.__styleGuard = {
    ready: true,
    snapshot() {
      const button = host.querySelector("button")!,
        computed = getComputedStyle(button);
      return {
        html: host.innerHTML,
        tree: [...host.childNodes].map(tree),
        metrics: structuredClone(metrics),
        diagnostics: structuredClone(diagnostics),
        sameNodes:
          allNodes(host).length === original.length &&
          original.every((node, index) => allNodes(host)[index] === node),
        styles: {
          color: computed.color,
          backgroundColor: computed.backgroundColor,
          fontWeight: computed.fontWeight,
          fontFamily: computed.fontFamily,
          float: computed.float,
          colorPriority: button.style.getPropertyPriority("color"),
          weightPriority: button.style.getPropertyPriority("font-weight"),
          customBrand: button.style.getPropertyValue("--Brand"),
          customPriority: button.style.getPropertyPriority("--Brand"),
        },
      };
    },
    async restore() {
      module.controls.reset();
      await Vue.nextTick();
    },
    async dispose() {
      app.unmount();
      await Vue.nextTick();
      return { html: host.innerHTML, diagnostics: structuredClone(diagnostics) };
    },
  };
} catch (error) {
  globalThis.__styleGuardFailure = {
    name: (error as Error).name,
    message: (error as Error).message,
    stack: (error as Error).stack,
  };
}
