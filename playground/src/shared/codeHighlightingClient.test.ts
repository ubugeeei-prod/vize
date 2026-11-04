import { afterEach, beforeEach, expect, it, vi } from "vite-plus/test";
import { createCodeHighlighter } from "./codeHighlightingClient";
import type { HighlightResponse, ThemedCodeToken } from "./codeHighlighting";

class HighlightWorker {
  static instances: HighlightWorker[] = [];
  onmessage: ((event: MessageEvent<HighlightResponse>) => void) | null = null;
  onerror: ((event: ErrorEvent) => void) | null = null;
  postMessage = vi.fn();
  terminate = vi.fn();
  constructor() {
    HighlightWorker.instances.push(this);
  }
  emit(data: HighlightResponse) {
    this.onmessage?.({ data } as MessageEvent<HighlightResponse>);
  }
}

const tokens: ThemedCodeToken[][] = [
  [{ content: "日本語😀<&", darkColor: "#000000", lightColor: "#ffffff" }],
];

beforeEach(() => {
  HighlightWorker.instances = [];
  vi.useFakeTimers();
  vi.stubGlobal("Worker", HighlightWorker);
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

it("loads lazily and reuses an idle worker without altering source or tokens", () => {
  const highlighter = createCodeHighlighter();
  const complete = vi.fn();
  expect(HighlightWorker.instances).toHaveLength(0);
  highlighter.highlight("日本語😀<&", "typescript", complete);
  const worker = HighlightWorker.instances[0];
  expect(worker.postMessage).not.toHaveBeenCalled();
  worker.emit({ type: "ready" });
  expect(worker.postMessage).toHaveBeenLastCalledWith({
    id: 1,
    code: "日本語😀<&",
    language: "typescript",
  });
  worker.emit({ type: "result", id: 1, lines: tokens });
  expect(complete).toHaveBeenCalledExactlyOnceWith(tokens);
  highlighter.highlight("new", "html", complete);
  expect(HighlightWorker.instances).toHaveLength(1);
  expect(worker.postMessage).toHaveBeenLastCalledWith({ id: 2, code: "new", language: "html" });
  highlighter.cancel();
});

it("terminates an in-flight generation and rejects every late event from its worker", () => {
  const highlighter = createCodeHighlighter();
  const oldComplete = vi.fn();
  const currentComplete = vi.fn();
  highlighter.highlight("old", "typescript", oldComplete);
  const old = HighlightWorker.instances[0];
  old.emit({ type: "ready" });
  highlighter.highlight("current", "json", currentComplete);
  expect(old.terminate).toHaveBeenCalledOnce();
  old.emit({ type: "result", id: 1, lines: tokens });
  old.emit({ type: "ready" });
  old.emit({ type: "error", id: 1 });
  const current = HighlightWorker.instances[1];
  current.emit({ type: "ready" });
  current.emit({ type: "result", id: 1, lines: tokens });
  expect(currentComplete).not.toHaveBeenCalled();
  current.emit({ type: "result", id: 2, lines: tokens });
  expect(currentComplete).toHaveBeenCalledExactlyOnceWith(tokens);
  expect(oldComplete).not.toHaveBeenCalled();
  expect(current.terminate).not.toHaveBeenCalled();
  highlighter.cancel();
});

it("bounds initialization and ignores a worker that becomes ready after cancellation", async () => {
  const highlighter = createCodeHighlighter();
  const complete = vi.fn();
  highlighter.highlight("current", "html", complete);
  const worker = HighlightWorker.instances[0];
  await vi.advanceTimersByTimeAsync(4999);
  expect(worker.terminate).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1);
  expect(worker.terminate).toHaveBeenCalledOnce();
  worker.emit({ type: "ready" });
  worker.emit({ type: "result", id: 1, lines: tokens });
  expect(worker.postMessage).not.toHaveBeenCalled();
  expect(complete).not.toHaveBeenCalled();
});

it("gives a replacement worker its own initialization phase before the tokenizer deadline", async () => {
  const highlighter = createCodeHighlighter();
  const complete = vi.fn();
  highlighter.highlight("old", "typescript", complete);
  const old = HighlightWorker.instances[0];
  old.emit({ type: "ready" });
  highlighter.highlight("current", "typescript", complete);
  const current = HighlightWorker.instances[1];
  expect(old.terminate).toHaveBeenCalledOnce();
  await vi.advanceTimersByTimeAsync(2000);
  expect(current.terminate).not.toHaveBeenCalled();
  expect(current.postMessage).not.toHaveBeenCalled();
  expect(complete).not.toHaveBeenCalled();
  current.emit({ type: "ready" });
  expect(current.postMessage).toHaveBeenLastCalledWith({
    id: 2,
    code: "current",
    language: "typescript",
  });
  await vi.advanceTimersByTimeAsync(999);
  expect(current.terminate).not.toHaveBeenCalled();
  current.emit({ type: "result", id: 2, lines: tokens });
  expect(complete).toHaveBeenCalledExactlyOnceWith(tokens);
  await vi.advanceTimersByTimeAsync(1);
  expect(current.terminate).not.toHaveBeenCalled();
  highlighter.cancel();
});

it("hard-cancels a non-returning tokenizer and can highlight a later source", async () => {
  const highlighter = createCodeHighlighter();
  const complete = vi.fn();
  highlighter.highlight("long source", "typescript", complete);
  const blocked = HighlightWorker.instances[0];
  blocked.emit({ type: "ready" });
  await vi.advanceTimersByTimeAsync(999);
  expect(blocked.terminate).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1);
  expect(blocked.terminate).toHaveBeenCalledOnce();
  blocked.emit({ type: "result", id: 1, lines: tokens });
  expect(complete).not.toHaveBeenCalled();
  highlighter.highlight("current", "css", complete);
  const current = HighlightWorker.instances[1];
  current.emit({ type: "ready" });
  current.emit({ type: "result", id: 2, lines: tokens });
  expect(complete).toHaveBeenCalledExactlyOnceWith(tokens);
  highlighter.cancel();
});

it.each(["initialization-error", "error"] as const)("retains plain output on %s", (type) => {
  const highlighter = createCodeHighlighter();
  const complete = vi.fn();
  highlighter.highlight("current", "typescript", complete);
  const worker = HighlightWorker.instances[0];
  if (type === "error") worker.emit({ type: "ready" });
  worker.emit(type === "error" ? { type, id: 1 } : { type });
  expect(worker.terminate).toHaveBeenCalledOnce();
  expect(complete).not.toHaveBeenCalled();
});

it("terminates on disposal without publishing a late result", () => {
  const highlighter = createCodeHighlighter();
  const complete = vi.fn();
  highlighter.highlight("current", "typescript", complete);
  const worker = HighlightWorker.instances[0];
  worker.emit({ type: "ready" });
  highlighter.cancel();
  worker.emit({ type: "result", id: 1, lines: tokens });
  expect(worker.terminate).toHaveBeenCalledOnce();
  expect(complete).not.toHaveBeenCalled();
});
