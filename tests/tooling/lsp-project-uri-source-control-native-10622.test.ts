import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { frameMessage } from "../differential/lsp-wire.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import {
  auditSourceControlNative,
  type SourceControlNativeProducer,
} from "./support/lsp/project-uri-source-control-native-10622.ts";
import { sha256 } from "./support/project-uri-membership-10622.ts";
import { prepareProjectUriProvider } from "./support/project-uri-membership-provider-10622.ts";

const provider = prepareProjectUriProvider();
const base = path.join(testOutputRoot, "project-uri-source-control-10622");
fs.mkdirSync(base, { recursive: true });
const producer: SourceControlNativeProducer = {
  role: "candidate",
  sourceRevision: "6573b92e74c93883303e34e9f12869868f567207",
  cliReceiptRoot: process.cwd(),
};
const spawn = {
  kind: "spawn",
  pid: 1,
  argv: ["--lsp", "--stdio"],
  binary: process.execPath,
  binarySha256: sha256(fs.readFileSync(process.execPath)),
};
const packets = (direction: string, value: unknown) => {
  const frame = frameMessage(value);
  return [
    { kind: "chunk", pid: 1, direction, chunkBase64: frame.toString("base64") },
    { kind: "frame", pid: 1, direction, frameBase64: frame.toString("base64"), packet: value },
  ];
};
async function refuses(events: unknown[], pattern: RegExp) {
  const output = fs.mkdtempSync(path.join(base, "native-refusal-"));
  const file = path.join(output, "native.jsonl");
  fs.writeFileSync(file, events.map((event) => JSON.stringify(event)).join("\n") + "\n");
  await assert.rejects(auditSourceControlNative(file, output, process.execPath, producer), pattern);
}

test("native source-control wire refuses foreign ownership and physical provider substitution", async () => {
  await refuses(
    [{ kind: "chunk", pid: 2, direction: "client", chunkBase64: "" }],
    /original observed owner/u,
  );
  await refuses([{ ...spawn, binarySha256: "0".repeat(64) }], /actual native binary SHA/u);
  await refuses([{ ...spawn, binary: import.meta.filename }], /actual native physical binary/u);
});

test("native source-control framing preserves unknown packet fields and the existing byte bound", async () => {
  const packet = { jsonrpc: "2.0", method: "test/control", params: { opaque: "retained" } };
  const rows = packets("client", packet);
  await refuses(
    [spawn, rows[0], { ...rows[1], packet: { ...packet, params: { opaque: "changed" } } }],
    /reported native packet retains every field/u,
  );
  for (const [header, reason] of [
    ["Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}", /one native Content-Length/u],
    [`Content-Length: ${16 * 1024 * 1024 + 1}\r\n\r\n`, /existing wire bound/u],
  ] as const)
    await refuses(
      [
        spawn,
        {
          kind: "chunk",
          pid: 1,
          direction: "client",
          chunkBase64: Buffer.from(header).toString("base64"),
        },
      ],
      reason,
    );
});

test(
  "native source-control refuses changed complete code before any source denominator admission",
  { skip: !provider.vue },
  async () => {
    const nativeRoot = path.join(base, "native-project");
    const uri = pathToFileURL(nativeRoot).href;
    await refuses(
      [
        spawn,
        ...packets("client", {
          jsonrpc: "2.0",
          id: 1,
          method: "initialize",
          params: {
            rootPath: nativeRoot,
            rootUri: uri,
            workspaceFolders: [{ name: "native-project", uri }],
          },
        }),
        ...packets("server", {
          jsonrpc: "2.0",
          id: 1,
          result: { capabilities: { positionEncoding: "utf-16" } },
        }),
        ...packets("client", { jsonrpc: "2.0", method: "initialized", params: {} }),
        ...packets("client", {
          jsonrpc: "2.0",
          method: "textDocument/didOpen",
          params: {
            textDocument: {
              uri: pathToFileURL(path.join(nativeRoot, "src/c/Comp0.vue.ts")).href,
              languageId: "typescript",
              version: 1,
              text: "export const changed = true;",
            },
          },
        }),
      ],
      /src\/c\/Comp0\.vue\.ts complete native code/u,
    );
  },
);
