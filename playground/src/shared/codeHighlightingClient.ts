import type {
  CodeHighlightLanguage,
  HighlightRequest,
  HighlightResponse,
  ThemedCodeToken,
} from "./codeHighlighting";

const initializationBudgetMs = 5000;
const tokenizationBudgetMs = 1000;

export function createCodeHighlighter() {
  let worker: Worker | null = null;
  let ready = false;
  let nextId = 0;
  let deadline: ReturnType<typeof setTimeout> | undefined;
  let pending:
    | { request: HighlightRequest; complete: (lines: ThemedCodeToken[][]) => void }
    | undefined;

  function clearDeadline() {
    clearTimeout(deadline);
    deadline = undefined;
  }

  function cancel() {
    clearDeadline();
    pending = undefined;
    worker?.terminate();
    worker = null;
    ready = false;
  }

  function submit() {
    if (!worker || !pending) return;
    clearDeadline();
    deadline = setTimeout(cancel, tokenizationBudgetMs);
    worker.postMessage(pending.request);
  }

  function start() {
    const current = new Worker(new URL("./codeHighlighting.worker.ts", import.meta.url), {
      type: "module",
    });
    worker = current;
    deadline = setTimeout(cancel, initializationBudgetMs);
    current.onmessage = (event: MessageEvent<HighlightResponse>) => {
      if (worker !== current) return;
      const message = event.data;
      if (message.type === "ready") {
        ready = true;
        submit();
        return;
      }
      if (message.type === "initialization-error") {
        cancel();
        return;
      }
      if (!pending || pending.request.id !== message.id) return;
      if (message.type === "error") {
        cancel();
        return;
      }
      clearDeadline();
      const completed = pending;
      pending = undefined;
      completed.complete(message.lines);
    };
    current.onerror = (event) => {
      event.preventDefault();
      if (worker === current) cancel();
    };
  }

  function highlight(
    code: string,
    language: CodeHighlightLanguage,
    complete: (lines: ThemedCodeToken[][]) => void,
  ) {
    // A synchronous tokenizer cannot consume cancellation messages while it runs.
    if (pending) cancel();
    pending = { request: { id: ++nextId, code, language }, complete };
    try {
      if (!worker) start();
      else if (ready) submit();
    } catch {
      cancel();
    }
  }

  return { highlight, cancel };
}
