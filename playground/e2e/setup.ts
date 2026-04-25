import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const originalFetch = globalThis.fetch.bind(globalThis);
const wasmDir = resolve(import.meta.dirname, "../src/wasm");

Object.defineProperty(WebAssembly, "instantiateStreaming", {
  configurable: true,
  value: undefined,
});

function fetchUrl(input: RequestInfo | URL): string | null {
  if (typeof input === "string") {
    return input;
  }
  if (input instanceof URL) {
    return input.href;
  }
  if (typeof input === "object" && input && "url" in input) {
    return String(input.url);
  }
  return null;
}

globalThis.fetch = async (input, init) => {
  const url = fetchUrl(input);
  const pathname = url ? new URL(url, "http://localhost").pathname : "";
  if (pathname === "/src/wasm/vize_vitrine_bg.wasm") {
    const bytes = await readFile(resolve(wasmDir, "vize_vitrine_bg.wasm"));
    return new Response(new Uint8Array(bytes), {
      headers: { "Content-Type": "application/wasm" },
    });
  }
  return originalFetch(input, init);
};
