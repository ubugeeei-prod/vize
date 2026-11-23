import { readFileSync } from "node:fs";

export const resourceCorsaProxySource = readFileSync(
  new URL("./lsp-resource-corsa-proxy.cjs", import.meta.url),
  "utf8",
);
