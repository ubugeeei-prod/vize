#!/usr/bin/env node
const fs = require("node:fs");

const args = process.argv.slice(2);
const logPath = process.env.VIZE_TEST_SERVER_LOG;
const version = process.env.VIZE_TEST_SERVER_VERSION ?? "0.0.0";
let activeInitializationOptions = {};
let workspaceRootUri = "file:///fake-vize-workspace";

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
    activeInitializationOptions = message.params?.initializationOptions ?? {};
    workspaceRootUri = message.params?.rootUri ?? workspaceRootUri;
    send({
      id: message.id,
      jsonrpc: "2.0",
      result: {
        capabilities: createCapabilities(),
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

  if (message.method === "textDocument/didOpen") {
    publishDiagnostics(message.params?.textDocument?.uri);
    return;
  }

  if (message.id !== undefined && message.method) {
    send({
      id: message.id,
      jsonrpc: "2.0",
      result: createResponse(message),
    });
    return;
  }
}

function summarizeParams(params) {
  if (!params || typeof params !== "object") {
    return params;
  }

  return {
    changes: summarizeWatchedFileChanges(params.changes),
    context: summarizeContext(params.context),
    initializationOptions: params.initializationOptions,
    newName: params.newName,
    position: params.position,
    processId: params.processId,
    query: params.query,
    range: params.range,
    rootUri: params.rootUri,
    textDocument: summarizeTextDocument(params.textDocument),
    trace: params.trace,
    workspaceFolders: params.workspaceFolders,
  };
}

function createCapabilities() {
  return {
    codeActionProvider: true,
    codeLensProvider: {
      resolveProvider: false,
    },
    completionProvider: {
      resolveProvider: false,
      triggerCharacters: [".", "<", ":"],
    },
    definitionProvider: true,
    documentFormattingProvider: true,
    documentLinkProvider: {
      resolveProvider: false,
    },
    documentSymbolProvider: true,
    foldingRangeProvider: true,
    hoverProvider: true,
    inlayHintProvider: true,
    referencesProvider: true,
    renameProvider: {
      prepareProvider: true,
    },
    semanticTokensProvider: {
      full: true,
      legend: {
        tokenModifiers: ["vue"],
        tokenTypes: ["vueComponent", "vueDirective"],
      },
      range: false,
    },
    textDocumentSync: 1,
    workspaceSymbolProvider: true,
  };
}

function createResponse(message) {
  const uri = message.params?.textDocument?.uri ?? workspaceRootUri;

  switch (message.method) {
    case "textDocument/codeAction":
      return [
        {
          edit: {
            changes: {
              [uri]: [
                {
                  newText: "/* fake quick fix */",
                  range: fixtureRange(),
                },
              ],
            },
          },
          kind: "quickfix",
          title: "Fake Vize Quick Fix",
        },
      ];

    case "textDocument/codeLens":
      return [
        {
          command: {
            command: "vize.showStatus",
            title: "Fake Vize Lens",
          },
          range: fixtureRange(),
        },
      ];

    case "textDocument/completion":
      return {
        isIncomplete: false,
        items: [
          {
            detail: "Fake Vize autocomplete property",
            documentation: {
              kind: "markdown",
              value: "Completion supplied by fake-vize.",
            },
            insertText: "fakeCompletion",
            kind: 10,
            label: "Fake Vize Completion",
            sortText: "0001",
          },
        ],
      };

    case "textDocument/definition":
      return [definitionLocation(uri)];

    case "textDocument/references":
      return [definitionLocation(uri), referenceLocation(uri)];

    case "textDocument/documentLink":
      return [
        {
          range: fixtureRange(),
          target: uri,
        },
      ];

    case "textDocument/documentSymbol":
      return [
        {
          children: [],
          detail: "fake-vize component",
          kind: 5,
          name: "FakeComponent",
          range: fixtureRange(),
          selectionRange: fixtureRange(),
        },
      ];

    case "textDocument/foldingRange":
      return [
        {
          endLine: 3,
          startLine: 0,
        },
      ];

    case "textDocument/formatting":
      return [
        {
          newText: "\n",
          range: {
            end: {
              character: 0,
              line: 0,
            },
            start: {
              character: 0,
              line: 0,
            },
          },
        },
      ];

    case "textDocument/hover":
      return {
        contents: {
          kind: "markdown",
          value: "**Fake Vize Hover**",
        },
        range: fixtureRange(),
      };

    case "textDocument/inlayHint":
      return [
        {
          kind: 1,
          label: ": string",
          position: {
            character: 13,
            line: 1,
          },
        },
      ];

    case "textDocument/prepareRename":
      return {
        placeholder: "message",
        range: fixtureRange(),
      };

    case "textDocument/rename":
      return {
        changes: {
          [uri]: [
            {
              newText: message.params?.newName ?? "renamed",
              range: fixtureRange(),
            },
          ],
        },
      };

    case "textDocument/semanticTokens/full":
      return {
        data: [1, 6, 7, 0, 1, 4, 2, 4, 1, 0],
      };

    case "workspace/symbol":
      return [
        {
          containerName: "fake-vize",
          kind: 13,
          location: {
            range: fixtureRange(),
            uri: `${workspaceRootUri.replace(/\/$/, "")}/src/App.vue`,
          },
          name: "FakeWorkspaceSymbol",
        },
      ];

    default:
      return null;
  }
}

function fixtureRange() {
  return {
    end: {
      character: 13,
      line: 1,
    },
    start: {
      character: 6,
      line: 1,
    },
  };
}

function definitionLocation(uri) {
  return {
    range: fixtureRange(),
    uri,
  };
}

function referenceLocation(uri) {
  return {
    range: {
      end: {
        character: 19,
        line: 5,
      },
      start: {
        character: 12,
        line: 5,
      },
    },
    uri,
  };
}

function publishDiagnostics(uri) {
  if (!uri) {
    return;
  }

  const diagnostics = [];

  if (activeInitializationOptions.typecheck === true) {
    diagnostics.push({
      code: "fake-type-mismatch",
      message: "Fake Vize type error: string is not assignable to number.",
      range: fixtureRange(),
      severity: 1,
      source: "vize:typecheck",
    });
  }

  if (activeInitializationOptions.lint === true) {
    diagnostics.push({
      code: "fake-lint-rule",
      message: "Fake Vize lint error: template expression should be simplified.",
      range: referenceLocation(uri).range,
      severity: 2,
      source: "vize:lint",
    });
  }

  send({
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      diagnostics,
      uri,
    },
  });
}

function summarizeTextDocument(textDocument) {
  if (!textDocument || typeof textDocument !== "object") {
    return textDocument;
  }

  return {
    languageId: textDocument.languageId,
    uri: textDocument.uri,
    version: textDocument.version,
  };
}

function summarizeWatchedFileChanges(changes) {
  if (!Array.isArray(changes)) {
    return changes;
  }

  return changes.map((change) => ({
    type: change.type,
    uri: change.uri,
  }));
}

function summarizeContext(context) {
  if (!context || typeof context !== "object") {
    return context;
  }

  return {
    diagnostics: Array.isArray(context.diagnostics) ? context.diagnostics.length : undefined,
    includeDeclaration: context.includeDeclaration,
    only: context.only,
    triggerKind: context.triggerKind,
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
