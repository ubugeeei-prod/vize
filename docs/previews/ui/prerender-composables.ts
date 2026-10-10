/** Compile and server-render the same SFCs before their client hydration checks. */
import assert from "node:assert/strict";
import type { Component } from "vue";
import { previewComposableExamples, uiRequire, type previewBuildConfig } from "./build-config.ts";

export async function prerenderComposables(
  config: Awaited<ReturnType<typeof previewBuildConfig>>,
): Promise<void> {
  const { createServer } = (await import(uiRequire.resolve("vite"))) as typeof import("vite");
  const server = await createServer({
    ...config,
    appType: "custom",
    server: { middlewareMode: true },
    optimizeDeps: { noDiscovery: true },
  });
  try {
    const { createSSRApp } = (await server.ssrLoadModule("vue")) as typeof import("vue");
    const { renderToString } = (await server.ssrLoadModule(
      "vue/server-renderer",
    )) as typeof import("vue/server-renderer");
    for (const example of previewComposableExamples) {
      const module = (await server.ssrLoadModule(
        `/virtual-vize-composable-examples/${example.name}.vue`,
      )) as { default: Component };
      example.ssrHtml = await renderToString(createSSRApp(module.default));
      assert.match(
        example.ssrHtml,
        /class="composable-example"/,
        `${example.name}: actual SSR output`,
      );
    }
  } finally {
    await server.close();
  }
}
