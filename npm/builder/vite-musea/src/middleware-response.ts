import type { ServerResponse } from "node:http";

/** Both transformed and fallback previews keep the same response contract. */
export function sendPreviewModule(response: ServerResponse, code: string): void {
  response.setHeader("Content-Type", "application/javascript");
  response.setHeader("Cache-Control", "no-cache");
  response.end(code);
}

export function sendPreviewError(response: ServerResponse, error: unknown): void {
  response.statusCode = 500;
  response.end(error instanceof Error ? error.message : String(error));
}

/** Art fallbacks retain their existing headers and bound generation failures. */
export function sendArtModuleFallback(response: ServerResponse, generate: () => string): void {
  try {
    const code = generate();
    response.setHeader("Content-Type", "application/javascript");
    response.end(code);
  } catch (error) {
    sendPreviewError(response, error);
  }
}
