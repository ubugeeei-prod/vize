export type TestTextDocumentPositionRequest = {
  character: number;
  line: number;
  uri: string;
};

export type TestCompletionRequest = TestTextDocumentPositionRequest;
export type TestReferencesRequest = TestTextDocumentPositionRequest & {
  includeDeclaration?: boolean;
};
export type TestLspRequest = {
  method: string;
  params?: unknown;
};

export type HostTestLanguageClient = {
  sendRequest(method: string, params: unknown): Promise<unknown>;
};

export type HostTestServerInfo = {
  extensionVersion?: string;
  path: string;
  source: string;
  status: string;
  version?: string;
  versionError?: string;
};

export type HostTestCommand = {
  command: string;
  handler: (request?: unknown) => Promise<unknown>;
};

export const HOST_TEST_COMMAND_ENVIRONMENT_FLAG = "VIZE_TEST_ENABLE_HOST_COMMANDS";
export const HOST_TEST_COMPLETION_COMMAND = "vize.test.executeCompletion";
export const HOST_TEST_LSP_REQUEST_COMMAND = "vize.test.executeLspRequest";
export const HOST_TEST_REFERENCES_COMMAND = "vize.test.executeReferences";
export const HOST_TEST_SERVER_INFO_COMMAND = "vize.test.getServerInfo";

/**
 * Hidden host-smoke hooks keep the packaged extension test on the Vize
 * LanguageClient without waiting on unrelated VS Code providers. They only
 * exist when the host smoke sets the environment flag.
 */
export function createHostTestCommands(behavior: {
  environment: Partial<Record<string, string>>;
  getClient: () => HostTestLanguageClient | undefined;
  getServerInfo?: () => HostTestServerInfo | undefined;
}): HostTestCommand[] {
  if (behavior.environment[HOST_TEST_COMMAND_ENVIRONMENT_FLAG] !== "1") {
    return [];
  }

  return [
    {
      command: HOST_TEST_COMPLETION_COMMAND,
      handler: async (request) => {
        const activeClient = requireActiveClient(behavior, "completion");
        assertTextDocumentPositionRequest(request, "completion");
        return activeClient.sendRequest(
          "textDocument/completion",
          textDocumentPositionParams(request),
        );
      },
    },
    {
      command: HOST_TEST_REFERENCES_COMMAND,
      handler: async (request) => {
        const activeClient = requireActiveClient(behavior, "references");
        assertTestReferencesRequest(request);
        return activeClient.sendRequest("textDocument/references", {
          ...textDocumentPositionParams(request),
          context: { includeDeclaration: request.includeDeclaration ?? true },
        });
      },
    },
    {
      command: HOST_TEST_LSP_REQUEST_COMMAND,
      handler: async (request) => {
        const activeClient = requireActiveClient(behavior, "LSP request");
        assertTestLspRequest(request);
        return activeClient.sendRequest(request.method, request.params);
      },
    },
    {
      command: HOST_TEST_SERVER_INFO_COMMAND,
      handler: async () => {
        const serverInfo = behavior.getServerInfo?.();
        if (!serverInfo) {
          throw new Error("Vize test server info command requires selected server evidence.");
        }
        return serverInfo;
      },
    },
  ];
}

/**
 * Registers the gated host commands through the caller's command registry so
 * the wiring stays observable without a VS Code host.
 */
export function bindHostTestCommands<TRegistration>(behavior: {
  environment: Partial<Record<string, string>>;
  getClient: () => HostTestLanguageClient | undefined;
  getServerInfo?: () => HostTestServerInfo | undefined;
  register: (command: string, handler: (request?: unknown) => Promise<unknown>) => TRegistration;
}): TRegistration[] {
  return createHostTestCommands(behavior).map(({ command, handler }) =>
    behavior.register(command, handler),
  );
}

function requireActiveClient(
  behavior: { getClient: () => HostTestLanguageClient | undefined },
  label: string,
): HostTestLanguageClient {
  const activeClient = behavior.getClient();
  if (!activeClient) {
    throw new Error(`Vize test ${label} command requires an active language client.`);
  }
  return activeClient;
}

function assertTestReferencesRequest(request: unknown): asserts request is TestReferencesRequest {
  assertTextDocumentPositionRequest(request, "references");
  if (
    (request as TestReferencesRequest).includeDeclaration !== undefined &&
    typeof (request as TestReferencesRequest).includeDeclaration !== "boolean"
  ) {
    throw new TypeError("Invalid Vize test references request.");
  }
}

function assertTestLspRequest(request: unknown): asserts request is TestLspRequest {
  const candidate = request as Partial<TestLspRequest> | null;
  if (
    candidate == null ||
    typeof candidate.method !== "string" ||
    candidate.method.trim().length === 0
  ) {
    throw new TypeError("Invalid Vize test LSP request.");
  }
}

function assertTextDocumentPositionRequest(
  request: unknown,
  label: string,
): asserts request is TestTextDocumentPositionRequest {
  const candidate = request as Partial<TestTextDocumentPositionRequest> | null;
  const line = candidate?.line;
  const character = candidate?.character;
  if (
    candidate == null ||
    typeof candidate.uri !== "string" ||
    typeof line !== "number" ||
    typeof character !== "number" ||
    !Number.isInteger(line) ||
    !Number.isInteger(character) ||
    line < 0 ||
    character < 0
  ) {
    throw new TypeError(`Invalid Vize test ${label} request.`);
  }
}

function textDocumentPositionParams(request: TestTextDocumentPositionRequest) {
  return {
    textDocument: { uri: request.uri },
    position: { line: request.line, character: request.character },
  };
}
