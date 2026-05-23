#!/usr/bin/env node
const fs = require("node:fs");

const args = process.argv.slice(2);
const logPath = process.env.VIZE_TEST_SERVER_LOG;
const version = process.env.VIZE_TEST_SERVER_VERSION ?? "0.0.0";

appendLog({
  args,
  event: "process-start",
  pid: process.pid,
});

process.on("exit", (code) => {
  appendLog({
    code,
    event: "process-exit",
    pid: process.pid,
  });
});

if (args.includes("--version")) {
  appendLog({ event: "version" });
  console.log(`vize ${version}`);
  process.exit(0);
}

if (args[0] !== "lsp") {
  appendLog({
    event: "unexpected-args",
  });
  process.exit(2);
}

let input = Buffer.alloc(0);

process.stdin.on("data", (chunk) => {
  input = Buffer.concat([input, chunk]);
  consumeInput();
});

process.stdin.on("end", () => {
  appendLog({ event: "stdin-end" });
});

function consumeInput() {
  while (true) {
    const headerEnd = input.indexOf("\r\n\r\n");
    if (headerEnd === -1) {
      return;
    }

    const header = input.subarray(0, headerEnd).toString("ascii");
    const lengthMatch = /^Content-Length: (\d+)$/im.exec(header);
    if (!lengthMatch) {
      appendLog({
        event: "malformed-header",
        header,
      });
      process.exit(3);
    }

    const contentLength = Number(lengthMatch[1]);
    const messageStart = headerEnd + 4;
    const messageEnd = messageStart + contentLength;
    if (input.byteLength < messageEnd) {
      return;
    }

    const rawMessage = input.subarray(messageStart, messageEnd).toString("utf-8");
    input = input.subarray(messageEnd);
    handleMessage(JSON.parse(rawMessage));
  }
}

function handleMessage(message) {
  appendLog({
    event: "message",
    id: message.id,
    method: message.method,
    params: summarizeParams(message.params),
  });

  if (message.method === "initialize") {
    send({
      id: message.id,
      jsonrpc: "2.0",
      result: {
        capabilities: {
          referencesProvider: true,
          textDocumentSync: 1,
        },
        serverInfo: {
          name: "fake-vize",
          version,
        },
      },
    });
    return;
  }

  if (message.method === "shutdown") {
    send({
      id: message.id,
      jsonrpc: "2.0",
      result: null,
    });
    return;
  }

  if (message.method === "exit") {
    process.exit(0);
  }

  if (message.id !== undefined) {
    send({
      id: message.id,
      jsonrpc: "2.0",
      result: null,
    });
  }
}

function summarizeParams(params) {
  if (!params || typeof params !== "object") {
    return params;
  }

  return {
    initializationOptions: params.initializationOptions,
    processId: params.processId,
    rootUri: params.rootUri,
    trace: params.trace,
    workspaceFolders: params.workspaceFolders,
  };
}

function send(payload) {
  const body = JSON.stringify(payload);
  process.stdout.write(`Content-Length: ${Buffer.byteLength(body, "utf-8")}\r\n\r\n${body}`);
}

function appendLog(entry) {
  if (!logPath) {
    return;
  }

  fs.appendFileSync(
    logPath,
    `${JSON.stringify({
      ...entry,
      time: new Date().toISOString(),
    })}\n`,
  );
}
