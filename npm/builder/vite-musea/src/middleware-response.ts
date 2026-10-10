import type { ServerResponse } from "node:http";

/** Both transformed and fallback previews keep the same response contract. */
export function sendPreviewModule(response: ServerResponse, code: string): void {
  response.setHeader("Content-Type", "application/javascript");
  response.setHeader("Cache-Control", "no-cache");
  response.end(code);
}
