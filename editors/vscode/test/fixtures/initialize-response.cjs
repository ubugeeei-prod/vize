const fs = require("node:fs");

exports.sendInitializeResponse = function sendInitializeResponse(
  id,
  capabilities,
  version,
  send,
  appendLog,
) {
  const response = {
    id,
    jsonrpc: "2.0",
    result: {
      capabilities,
      serverInfo: {
        name: "fake-vize",
        version,
      },
    },
  };
  const gate = process.env.VIZE_TEST_SERVER_INITIALIZE_GATE;
  if (gate && !fs.existsSync(gate)) {
    appendLog({ event: "initialize-gated", gate });
    const timer = setInterval(() => {
      if (fs.existsSync(gate)) {
        clearInterval(timer);
        send(response);
      }
    }, 10);
  } else {
    send(response);
  }
};

exports.createCapabilities = function createCapabilities(activeInitializationOptions) {
  const capabilities = {
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
    documentHighlightProvider: true,
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
    signatureHelpProvider: {
      triggerCharacters: ["(", ","],
    },
    textDocumentSync: 1,
    workspaceSymbolProvider: true,
  };
  if (activeInitializationOptions.autoInsert === true) {
    capabilities.experimental = {
      autoInsertionProvider: {
        triggerCharacters: ["}", "=", ">", "/", "\\w"],
        configurationSections: [["vize.autoInsert.bracketSpacing"]],
      },
    };
  }
  return capabilities;
};

// Same names as WORKSPACE_PROJECT_CONFIG_GLOB; native config watches use this
// registerCapability/RelativePattern/kind7 protocol, independently of typecheck.
exports.createConfigurationWatchRegistration = function createConfigurationWatchRegistration(
  workspaceRootUri,
  requestId,
) {
  return {
    id: requestId,
    jsonrpc: "2.0",
    method: "client/registerCapability",
    params: {
      registrations: [
        {
          id: "vize-config-watch-1",
          method: "workspace/didChangeWatchedFiles",
          registerOptions: {
            watchers: [
              {
                kind: 7,
                globPattern: {
                  baseUri: workspaceRootUri,
                  pattern:
                    "**/{vite.config.ts,vite.config.mts,vite.config.cts,vite.config.js,vite.config.mjs,vite.config.cjs,tsconfig.json,jsconfig.json,tsconfig.*.json}",
                },
              },
            ],
          },
        },
      ],
    },
  };
};
