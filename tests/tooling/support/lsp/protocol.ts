export type JsonRpcId = number;

export type JsonRpcMessage = {
  jsonrpc: "2.0";
  id?: JsonRpcId;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code: number; message: string };
};

export type LspInitializationOptions = {
  editor?: boolean;
  lint?: boolean;
  typecheck?: boolean;
};

export type LspDiagnostic = {
  code?: unknown;
  source?: string;
  severity?: number;
  message?: string;
  range?: {
    start?: { line?: number; character?: number };
    end?: { line?: number; character?: number };
  };
};

export type PublishDiagnosticsParams = {
  uri: string;
  diagnostics: LspDiagnostic[];
};
