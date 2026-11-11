/**
 * Absolute-floor content mapper: every SFC projects to $FLOOR_STUB_TEXT
 * (default `export default {} as any;`). The host still starts a mapper,
 * reads every SFC and loads every .ts file, but checks no projection.
 */

import { bodyReader, writeMessage } from "./frames.ts";

const text = process.env.FLOOR_STUB_TEXT || "export default {} as any;\n";
const results: Record<string, unknown> = {
  initialize: { diagnosticSource: "vize", positionEncoding: "utf-8" },
  openProject: { configIdentity: "floor" },
  transform: { diagnostics: [], extension: ".ts", mappings: [], text },
};
const read = bodyReader();
process.stdin.on("data", (chunk: Buffer) => {
  for (const body of read(chunk)) {
    const message = JSON.parse(body.toString("utf8")) as { id?: unknown; method?: string };
    if (message.id == null) continue;
    const result = results[message.method ?? ""] ?? null;
    writeMessage(process.stdout, { id: message.id, jsonrpc: "2.0", result });
  }
});
process.stdin.on("end", () => process.exit(0));
