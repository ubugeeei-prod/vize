import { expect, it, vi } from "vite-plus/test";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import CodeHighlight from "../src/shared/CodeHighlight.vue";

it("renders on the server without creating a browser worker", async () => {
  const worker = vi.fn(() => {
    throw new Error("server rendering cannot create a browser worker");
  });
  vi.stubGlobal("Worker", worker);
  try {
    const html = await renderToString(
      createSSRApp(CodeHighlight, {
        code: 'const source = "日本語😀<&";',
        language: "typescript",
        showLineNumbers: true,
      }),
    );
    expect(html).toContain('class="code-highlight');
    expect(html).toContain('class="code-content"');
    expect(worker).not.toHaveBeenCalled();
  } finally {
    vi.unstubAllGlobals();
  }
});
