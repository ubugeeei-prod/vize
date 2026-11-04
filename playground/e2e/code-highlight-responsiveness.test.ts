import { expect, it, vi } from "vite-plus/test";
import { createApp, nextTick } from "vue";
import CodeHighlight from "../src/shared/CodeHighlight.vue";
import CodeHighlightHarness from "./fixtures/CodeHighlightHarness.vue";

function sourceText(host: HTMLElement) {
  return Array.from(host.querySelectorAll(".code-line"), (line) =>
    line.textContent === "\u00a0" ? "" : line.textContent,
  ).join("\n");
}

it("keeps a complete 100k SFC script responsive and replaces it with the latest source", async () => {
  const host = document.createElement("div");
  document.body.appendChild(host);
  const warm = createApp(CodeHighlight, {
    code: "const warm = 'ready';",
    language: "typescript",
  });
  warm.mount(host);
  try {
    await expect.poll(() => host.querySelector(".code-line span[style]")).not.toBeNull();
  } finally {
    warm.unmount();
  }

  const large = `import { ref } from 'vue'; const publicLarge679 = ref<string>(${JSON.stringify("x".repeat(100000))});`;
  const replacement = 'const current = "日本語😀<&</script>";\n\n// complete current source';
  const app = createApp(CodeHighlightHarness, { initial: large, replacement });
  app.mount(host);
  try {
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    expect(sourceText(host)).toBe(large);
    host.querySelector("button")!.click();
    await nextTick();
    expect(sourceText(host)).toBe(replacement);
    await expect.poll(() => host.querySelector(".code-line span[style]")).not.toBeNull();
    expect(sourceText(host)).toBe(replacement);
    expect(host.querySelector("script")).toBeNull();
    expect(host.querySelectorAll(".line-number")).toHaveLength(3);
    expect(host.textContent).not.toContain("publicLarge679");
  } finally {
    app.unmount();
    host.remove();
  }
});

it("cancels an actual busy tokenizer while keeping complete source selection and editing available", async () => {
  const workers: { ready: boolean; terminated: boolean }[] = [];
  const NativeWorker = Worker;
  vi.stubGlobal(
    "Worker",
    new Proxy(NativeWorker, {
      construct(target, args) {
        const worker = Reflect.construct(target, args) as Worker;
        const state = { ready: false, terminated: false };
        workers.push(state);
        worker.addEventListener("message", ({ data }) => {
          if (data.type === "ready") state.ready = true;
        });
        const terminate = worker.terminate.bind(worker);
        worker.terminate = () => {
          state.terminated = true;
          terminate();
        };
        return worker;
      },
    }),
  );
  const host = document.createElement("div");
  document.body.appendChild(host);
  const large = `import { ref } from 'vue'; const publicLarge679 = ref<string>(${JSON.stringify("x".repeat(100000))});`;
  const replacement = 'const current = "日本語😀<&</script>";';
  const app = createApp(CodeHighlightHarness, { initial: large, replacement });
  app.mount(host);
  try {
    await expect.poll(() => workers[0]?.ready).toBe(true);
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    expect(workers[0].terminated).toBe(false);
    const range = document.createRange();
    range.selectNodeContents(host.querySelector(".code-content")!);
    const selection = window.getSelection()!;
    selection.removeAllRanges();
    selection.addRange(range);
    expect(selection.toString()).toBe(large);
    selection.removeAllRanges();
    host.querySelector("button")!.click();
    await nextTick();
    expect(workers[0].terminated).toBe(true);
    expect(sourceText(host)).toBe(replacement);
    await expect.poll(() => host.querySelector(".code-line span[style]")).not.toBeNull();
    expect(sourceText(host)).toBe(replacement);
    expect(host.querySelector("script")).toBeNull();
  } finally {
    app.unmount();
    host.remove();
    vi.unstubAllGlobals();
  }
  expect(workers.every((worker) => worker.terminated)).toBe(true);
});
