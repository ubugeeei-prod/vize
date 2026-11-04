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
  const workers: {
    ready: boolean;
    terminated: boolean;
    initialized: Promise<void>;
    events: { type: string; at: number }[];
  }[] = [];
  const NativeWorker = Worker;
  vi.stubGlobal(
    "Worker",
    new Proxy(NativeWorker, {
      construct(target, args) {
        const worker = Reflect.construct(target, args) as Worker;
        let initialized!: () => void;
        let initializationFailed!: (error: Error) => void;
        const initialization = new Promise<void>((resolve, reject) => {
          initialized = resolve;
          initializationFailed = reject;
        });
        // A worker may be disposed before this test awaits its initialization.
        void initialization.catch(() => {});
        const state = {
          ready: false,
          terminated: false,
          initialized: initialization,
          events: [{ type: "created", at: performance.now() }],
        };
        workers.push(state);
        worker.addEventListener("message", ({ data }) => {
          state.events.push({ type: data.type, at: performance.now() });
          if (data.type === "ready") {
            state.ready = true;
            initialized();
          } else if (data.type === "initialization-error") {
            initializationFailed(new Error("the real highlighter worker failed to initialize"));
          }
        });
        worker.addEventListener("error", (event) => {
          state.events.push({ type: "worker-error", at: performance.now() });
          initializationFailed(new Error(event.message));
        });
        const terminate = worker.terminate.bind(worker);
        worker.terminate = () => {
          state.terminated = true;
          state.events.push({ type: "terminated", at: performance.now() });
          if (!state.ready) {
            initializationFailed(new Error("the real highlighter worker stopped before ready"));
          }
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
    expect(workers).toHaveLength(2);
    // Initialization has its own production deadline. Start the unchanged
    // color assertion only after the replacement tokenizer can receive work.
    await workers[1].initialized;
    expect(workers[1].terminated).toBe(false);
    await expect.poll(() => host.querySelector(".code-line span[style]")).not.toBeNull();
    expect(sourceText(host)).toBe(replacement);
    expect(host.querySelector("script")).toBeNull();
  } finally {
    app.unmount();
    host.remove();
    vi.unstubAllGlobals();
    console.info(
      "code-highlight-worker-lifecycle",
      JSON.stringify(
        workers.map(({ ready, terminated, events }) => ({ ready, terminated, events })),
      ),
    );
  }
  expect(workers.every((worker) => worker.terminated)).toBe(true);
});
