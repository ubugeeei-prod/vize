import assert from "node:assert/strict";

import { createSSRApp } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

/** Result of {@link renderAndHydrate}. */
export interface HydrationResult {
  /** Server markup, identical across the two isolated requests. */
  readonly html: string;

  /** Hydrated host element, still attached to the document. */
  readonly host: HTMLElement;

  /** Unmount the hydrated app and detach the host. */
  readonly dispose: () => void;
}

/**
 * Render a probe in two isolated SSR requests, assert byte-identical markup,
 * then hydrate the markup in the document and assert that hydration keeps the
 * server root node and emits no Vue warnings or errors.
 */
export async function renderAndHydrate(probe: Component): Promise<HydrationResult> {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe)),
    renderToString(createSSRApp(probe)),
  ]);
  const html = outputs[0] ?? "";
  assert.equal(html, outputs[1], "server markup must be identical across requests");

  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(probe);
  try {
    app.mount(host);
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
  }
  assert.deepEqual(diagnostics, [], "hydration must not emit diagnostics");
  assert.ok(host.firstElementChild === serverRoot, "hydration must reuse the server root");

  return {
    html,
    host,
    dispose: () => {
      app.unmount();
      host.remove();
    },
  };
}
