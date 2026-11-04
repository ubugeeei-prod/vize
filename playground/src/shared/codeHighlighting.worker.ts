import { initCodeHighlighter, codeToThemedTokenLines } from "./codeHighlightingEngine";
import type { HighlightRequest, HighlightResponse } from "./codeHighlighting";

function respond(message: HighlightResponse) {
  self.postMessage(message);
}

async function initialize() {
  try {
    await initCodeHighlighter();
  } catch {
    respond({ type: "initialization-error" });
    return;
  }
  self.onmessage = async (event: MessageEvent<HighlightRequest>) => {
    const { id, code, language } = event.data;
    try {
      respond({ type: "result", id, lines: await codeToThemedTokenLines(code, language) });
    } catch {
      respond({ type: "error", id });
    }
  };
  respond({ type: "ready" });
}

void initialize();
