import assert from "node:assert/strict";
import { spawn, spawnSync, type ChildProcessWithoutNullStreams } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type JsonRpcId = number;

type JsonRpcMessage = {
  jsonrpc: "2.0";
  id?: JsonRpcId;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code: number; message: string };
};

class LspSession {
  private readonly process: ChildProcessWithoutNullStreams;
  private readonly pending = new Map<
    JsonRpcId,
    {
      resolve: (value: unknown) => void;
      reject: (error: Error) => void;
      method: string;
      timeout: NodeJS.Timeout;
    }
  >();
  private readonly notificationBacklog: Array<{ method: string; params: unknown }> = [];
  private readonly notifications: Array<{
    method: string;
    predicate?: (params: unknown) => boolean;
    resolve: (params: unknown) => void;
    reject: (error: Error) => void;
    timeout: NodeJS.Timeout;
  }> = [];
  private buffer = Buffer.alloc(0);
  private nextId = 0;
  private stderr = "";

  constructor() {
    const [command, ...args] = resolveVizeLaunchCommand();
    this.process = spawn(command, args, {
      cwd: root,
      stdio: ["pipe", "pipe", "pipe"],
    });

    this.process.stdout.on("data", (chunk: Buffer) => {
      this.buffer = Buffer.concat([this.buffer, chunk]);
      this.drainMessages();
    });

    this.process.stderr.on("data", (chunk: Buffer) => {
      this.stderr += chunk.toString("utf8");
    });

    this.process.on("exit", (code, signal) => {
      const error = new Error(
        `vize lsp exited unexpectedly (code=${code ?? "null"}, signal=${signal ?? "null"})\n${this.stderr}`.trim(),
      );

      for (const pending of this.pending.values()) {
        clearTimeout(pending.timeout);
        pending.reject(error);
      }
      this.pending.clear();

      for (const notification of this.notifications) {
        clearTimeout(notification.timeout);
        notification.reject(error);
      }
      this.notifications.length = 0;
    });
  }

  async initialize(workspaceDir: string): Promise<unknown> {
    const result = await this.request("initialize", {
      processId: process.pid,
      rootUri: pathToFileURL(workspaceDir).href,
      capabilities: {
        textDocument: {
          completion: {
            completionItem: {
              documentationFormat: ["markdown", "plaintext"],
            },
          },
        },
      },
      initializationOptions: {
        editor: true,
        typecheck: true,
      },
      workspaceFolders: [
        {
          uri: pathToFileURL(workspaceDir).href,
          name: path.basename(workspaceDir),
        },
      ],
    });

    this.notify("initialized", {});
    return result;
  }

  request(method: string, params: unknown, timeoutMs = 30000): Promise<unknown> {
    const id = ++this.nextId;

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Timed out waiting for ${method}\n${this.stderr}`.trim()));
      }, timeoutMs);

      this.pending.set(id, { resolve, reject, method, timeout });
      this.send({ jsonrpc: "2.0", id, method, params });
    });
  }

  notify(method: string, params: unknown): void {
    this.send({ jsonrpc: "2.0", method, params });
  }

  waitForNotification(
    method: string,
    predicate?: (params: unknown) => boolean,
    timeoutMs = 30000,
  ): Promise<unknown> {
    const backlogIndex = this.notificationBacklog.findIndex(
      (notification) =>
        notification.method === method && (predicate == null || predicate(notification.params)),
    );
    if (backlogIndex >= 0) {
      const [{ params }] = this.notificationBacklog.splice(backlogIndex, 1);
      return Promise.resolve(params);
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        const index = this.notifications.findIndex(
          (notification) => notification.resolve === resolve,
        );
        if (index >= 0) {
          this.notifications.splice(index, 1);
        }
        reject(new Error(`Timed out waiting for notification ${method}\n${this.stderr}`.trim()));
      }, timeoutMs);

      this.notifications.push({
        method,
        predicate,
        resolve,
        reject,
        timeout,
      });
    });
  }

  async shutdown(): Promise<void> {
    if (this.process.killed) {
      return;
    }

    try {
      await this.request("shutdown", undefined, 10000);
    } finally {
      this.notify("exit", undefined);
      this.process.stdin.end();
      await new Promise<void>((resolve) => {
        const timeout = setTimeout(() => {
          this.process.kill("SIGKILL");
        }, 5000);

        this.process.once("exit", () => {
          clearTimeout(timeout);
          resolve();
        });
      });
    }
  }

  private send(message: JsonRpcMessage): void {
    const payload = JSON.stringify(message);
    const frame = `Content-Length: ${Buffer.byteLength(payload, "utf8")}\r\n\r\n${payload}`;
    this.process.stdin.write(frame, "utf8");
  }

  private drainMessages(): void {
    while (true) {
      const headerEnd = this.buffer.indexOf("\r\n\r\n");
      if (headerEnd < 0) {
        return;
      }

      const header = this.buffer.subarray(0, headerEnd).toString("utf8");
      const lengthMatch = header.match(/Content-Length:\s*(\d+)/i);
      assert.ok(lengthMatch, `missing Content-Length header: ${header}`);

      const bodyLength = Number(lengthMatch[1]);
      const frameLength = headerEnd + 4 + bodyLength;
      if (this.buffer.length < frameLength) {
        return;
      }

      const body = this.buffer.subarray(headerEnd + 4, frameLength).toString("utf8");
      this.buffer = this.buffer.subarray(frameLength);

      const message = JSON.parse(body) as JsonRpcMessage;
      this.dispatch(message);
    }
  }

  private dispatch(message: JsonRpcMessage): void {
    if (typeof message.id === "number" && message.method == null) {
      const pending = this.pending.get(message.id);
      if (!pending) {
        return;
      }

      clearTimeout(pending.timeout);
      this.pending.delete(message.id);

      if (message.error) {
        pending.reject(new Error(`${pending.method}: ${message.error.message}`));
        return;
      }

      pending.resolve(message.result);
      return;
    }

    if (message.method != null && typeof message.id === "number") {
      this.send({
        jsonrpc: "2.0",
        id: message.id,
        error: {
          code: -32601,
          message: `client does not implement ${message.method}`,
        },
      });
      return;
    }

    if (message.method == null) {
      return;
    }

    const index = this.notifications.findIndex(
      (notification) =>
        notification.method === message.method &&
        (notification.predicate == null || notification.predicate(message.params)),
    );

    if (index < 0) {
      this.notificationBacklog.push({
        method: message.method,
        params: message.params,
      });
      return;
    }

    const [notification] = this.notifications.splice(index, 1);
    clearTimeout(notification.timeout);
    notification.resolve(message.params);
  }
}

test("vize lsp smoke-tests production editor flows", async (t) => {
  const workspaceDir = fs.mkdtempSync(path.join(root, ".tmp-lsp-smoke-"));
  const session = new LspSession();

  try {
    fs.writeFileSync(
      path.join(workspaceDir, "Child.vue"),
      `<script setup lang="ts"></script>
<template><button /></template>
`,
      "utf8",
    );

    const parentSource = `<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child />
</template>
`;
    const parentPath = path.join(workspaceDir, "Parent.vue");
    fs.writeFileSync(parentPath, parentSource, "utf8");

    const artSource = `<script setup lang="ts">
import { ref } from 'vue'

const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ primaryLabel }}</Child>
  </variant>
  <variant name="Secondary">
    <Child>{{ secondaryLabel }}</Child>
  </variant>
</art>
`;
    const artPath = path.join(workspaceDir, "Button.art.vue");
    fs.writeFileSync(artPath, artSource, "utf8");

    const init = (await session.initialize(workspaceDir)) as {
      capabilities?: {
        completionProvider?: {
          triggerCharacters?: string[];
        };
        hoverProvider?: boolean;
        definitionProvider?: boolean;
      };
    };

    assert.equal(init.capabilities?.hoverProvider, true);
    assert.equal(init.capabilities?.definitionProvider, true);
    assert.ok(init.capabilities?.completionProvider?.triggerCharacters?.includes("."));

    const parentUri = pathToFileURL(parentPath).href;
    const artUri = pathToFileURL(artPath).href;

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: parentUri,
        languageId: "vue",
        version: 1,
        text: parentSource,
      },
    });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: artUri,
        languageId: "art-vue",
        version: 1,
        text: artSource,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics");

    await t.test("go-to-definition resolves component tags in Vue templates", async () => {
      const childUsageOffset = parentSource.indexOf("Child />") + "Child".length;
      const definition = (await session.request("textDocument/definition", {
        textDocument: { uri: parentUri },
        position: offsetToPosition(parentSource, childUsageOffset),
      })) as
        | Array<{
            uri: string;
            range: { start: { line: number; character: number } };
          }>
        | {
            uri: string;
            range: { start: { line: number; character: number } };
          };

      const location = firstLocation(definition);
      assert.equal(location.uri, pathToFileURL(path.join(workspaceDir, "Child.vue")).href);
      assert.deepEqual(location.range.start, { line: 0, character: 0 });
    });

    await t.test("hover and definition stay correct in non-default art variants", async () => {
      const secondaryLabelOffset =
        artSource.lastIndexOf("secondaryLabel") + "secondaryLabel".length;

      const hover = (await session.request("textDocument/hover", {
        textDocument: { uri: artUri },
        position: offsetToPosition(artSource, secondaryLabelOffset),
      })) as { contents?: unknown } | null;

      const hoverText = hoverToText(hover);
      assert.match(hoverText, /secondaryLabel/);
      assert.match(hoverText, /(Ref<string>|string)/);

      const definition = (await session.request("textDocument/definition", {
        textDocument: { uri: artUri },
        position: offsetToPosition(artSource, secondaryLabelOffset),
      })) as
        | Array<{
            uri: string;
            range: { start: { line: number; character: number } };
          }>
        | {
            uri: string;
            range: { start: { line: number; character: number } };
          };

      const location = firstLocation(definition);
      assert.equal(location.uri, artUri);
      assert.deepEqual(location.range.start, { line: 4, character: 6 });
    });

    await t.test("completion surfaces bindings and directives inside art variants", async () => {
      const completionOffset = artSource.lastIndexOf("secondaryLabel") + "secondaryLabel".length;

      const response = (await session.request("textDocument/completion", {
        textDocument: { uri: artUri },
        position: offsetToPosition(artSource, completionOffset),
      })) as Array<{ label: string }> | { items?: Array<{ label: string }> } | null;

      const labels = completionLabels(response);
      assert.ok(labels.includes("secondaryLabel"), labels.join(", "));
      assert.ok(labels.includes("primaryLabel"), labels.join(", "));
      assert.ok(labels.includes("v-if"), labels.join(", "));
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

function resolveVizeLaunchCommand(): string[] {
  const candidates = [
    [path.join(root, "target/release/vize"), "lsp"],
    [path.join(root, "target/debug/vize"), "lsp"],
    ["vize", "lsp"],
  ];

  for (const candidate of candidates) {
    const probe = spawnSync(candidate[0], ["--version"], {
      cwd: root,
      encoding: "utf8",
    });
    if (probe.status === 0) {
      return candidate;
    }
  }

  return ["cargo", "run", "-q", "-p", "vize", "--", "lsp"];
}

function offsetToPosition(source: string, offset: number): { line: number; character: number } {
  const lines = source.slice(0, offset).split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1)?.length ?? 0,
  };
}

function firstLocation(
  response:
    | Array<{ uri: string; range: { start: { line: number; character: number } } }>
    | { uri: string; range: { start: { line: number; character: number } } },
): { uri: string; range: { start: { line: number; character: number } } } {
  return Array.isArray(response) ? response[0] : response;
}

function hoverToText(hover: { contents?: unknown } | null): string {
  assert.ok(hover?.contents);

  const contents = hover.contents;
  if (typeof contents === "string") {
    return contents;
  }
  if (Array.isArray(contents)) {
    return contents.map(markedStringToText).join("\n\n");
  }
  if (typeof contents === "object" && contents != null && "value" in contents) {
    const value = (contents as { value?: unknown }).value;
    return typeof value === "string" ? value : JSON.stringify(contents);
  }

  return JSON.stringify(contents);
}

function markedStringToText(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "object" && value != null && "value" in value) {
    const text = (value as { value?: unknown }).value;
    return typeof text === "string" ? text : JSON.stringify(value);
  }
  return JSON.stringify(value);
}

function completionLabels(
  response: Array<{ label: string }> | { items?: Array<{ label: string }> } | null,
): string[] {
  if (response == null) {
    return [];
  }
  if (Array.isArray(response)) {
    return response.map((item) => item.label);
  }
  return (response.items ?? []).map((item) => item.label);
}
