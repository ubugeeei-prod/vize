/**
 * The editor contract `lsp-vs-volar.ts` drives with one script, the LSP
 * plumbing both setups share, and the `vize` setup: `vize lsp` answers every
 * request, as the Vize VS Code extension uses it. The Vue setup lives in
 * `lsp-vs-volar-volar.ts`.
 */

import { spawn } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

import type { OpenDocument, Position, TextEdit } from "./lsp-vs-volar-text.ts";
import { LspPeer } from "./lsp-vs-volar-wire.ts";

export interface Diagnostic {
  code: string;
  line: number;
  message: string;
}

export interface Location {
  path: string;
  line: number;
  character: number;
}

export interface Editor {
  readonly label: string;
  readonly kind: "vize" | "volar" | "tsgo" | "tsserver";
  pids(): number[];
  start(): Promise<void>;
  open(doc: OpenDocument): void;
  /** `edit` has already been applied to `doc`. */
  change(doc: OpenDocument, edit: TextEdit): void;
  /** Diagnostics of `doc.version` once `until` holds; the last set seen if it never does. */
  diagnostics(
    doc: OpenDocument,
    until: (diagnostics: Diagnostic[]) => boolean,
    timeoutMs: number,
  ): Promise<{ diagnostics: Diagnostic[]; satisfied: boolean }>;
  hover(doc: OpenDocument, position: Position): Promise<string>;
  completion(doc: OpenDocument, position: Position): Promise<string[]>;
  definition(doc: OpenDocument, position: Position): Promise<Location[]>;
  references(doc: OpenDocument, position: Position): Promise<Location[]>;
  rename(doc: OpenDocument, position: Position, newName: string): Promise<Location[]>;
  semanticTokens(doc: OpenDocument): Promise<number>;
  stop(): Promise<void>;
  stderr(): string;
  /** Why a server process has exited, or `null` while all are running. */
  exited(): string | null;
}

export interface EditorEnvironment {
  workspace: string;
  env: Record<string, string>;
}

/** What VS Code advertises for the features measured here; pull diagnostics stay off. */
export function clientCapabilities() {
  return {
    workspace: { workspaceFolders: true, configuration: true, applyEdit: true },
    textDocument: {
      synchronization: { didSave: true, dynamicRegistration: false },
      publishDiagnostics: { relatedInformation: true, versionSupport: true },
      hover: { contentFormat: ["markdown", "plaintext"] },
      completion: {
        completionItem: {
          snippetSupport: true,
          documentationFormat: ["markdown", "plaintext"],
          resolveSupport: { properties: ["documentation", "detail", "additionalTextEdits"] },
          labelDetailsSupport: true,
        },
        contextSupport: true,
      },
      definition: { linkSupport: true },
      references: {},
      rename: { prepareSupport: true },
      semanticTokens: {
        requests: { full: { delta: false }, range: false },
        tokenTypes: [
          "namespace",
          "type",
          "class",
          "enum",
          "interface",
          "struct",
          "typeParameter",
          "parameter",
          "variable",
          "property",
          "enumMember",
          "event",
          "function",
          "method",
          "macro",
          "keyword",
          "modifier",
          "comment",
          "string",
          "number",
          "regexp",
          "operator",
          "decorator",
        ],
        tokenModifiers: [
          "declaration",
          "definition",
          "readonly",
          "static",
          "deprecated",
          "abstract",
          "async",
          "modification",
          "documentation",
          "defaultLibrary",
        ],
        formats: ["relative"],
        multilineTokenSupport: false,
        overlappingTokenSupport: false,
      },
    },
    window: { workDoneProgress: false },
  };
}

export function initializeParams(workspace: string, initializationOptions: unknown) {
  const uri = pathToFileURL(workspace).href;
  return {
    processId: process.pid,
    rootUri: uri,
    capabilities: clientCapabilities(),
    initializationOptions,
    workspaceFolders: [{ uri, name: "workspace" }],
  };
}

function uriPath(uri: string): string {
  return uri.startsWith("file:") ? fileURLToPath(uri) : uri;
}

export function hoverText(hover: any): string {
  const contents = hover?.contents;
  if (contents == null) return "";
  const parts = Array.isArray(contents) ? contents : [contents];
  return parts.map((part) => (typeof part === "string" ? part : (part?.value ?? ""))).join("\n");
}

/**
 * What each item completes to: `filterText`, else the label. Editors decorate
 * labels (VS Code's TypeScript provider shows an optional member as
 * `channel?` and inserts `channel`), so labels alone do not compare across
 * servers; tsserver entry names already are the completed identifier.
 */
export function completionLabels(result: any): string[] {
  const items = Array.isArray(result) ? result : (result?.items ?? []);
  return items.map(
    (item: { label: string | { label: string }; filterText?: string }) =>
      item.filterText ?? (typeof item.label === "string" ? item.label : item.label.label),
  );
}

export function lspLocations(result: any): Location[] {
  if (result == null) return [];
  const items = Array.isArray(result) ? result : [result];
  return items.map((item) => {
    const { start } = (item.targetSelectionRange ?? item.range) as { start: Position };
    return {
      path: uriPath(item.targetUri ?? item.uri),
      line: start.line,
      character: start.character,
    };
  });
}

export function workspaceEditLocations(edit: any): Location[] {
  const locations: Location[] = [];
  for (const [uri, edits] of Object.entries<any[]>(edit?.changes ?? {})) {
    for (const item of edits) locations.push({ path: uriPath(uri), ...item.range.start });
  }
  for (const change of edit?.documentChanges ?? []) {
    for (const item of change.edits ?? []) {
      locations.push({ path: uriPath(change.textDocument.uri), ...item.range.start });
    }
  }
  return locations;
}

function lspDiagnostics(items: any[]): Diagnostic[] {
  return items.map((item) => ({
    code: String(item.code ?? "").replace(/^TS/, ""),
    line: item.range.start.line,
    message: String(item.message ?? ""),
  }));
}

/** Publishes per URI, so a wait never misses one that arrived before it started. */
class PublishedDiagnostics {
  private readonly latest = new Map<string, { version: number | null; items: Diagnostic[] }>();
  private readonly waiters = new Set<() => void>();

  constructor(peer: LspPeer) {
    peer.onNotification("textDocument/publishDiagnostics", (params) => {
      this.latest.set(params.uri, {
        version: params.version ?? null,
        items: lspDiagnostics(params.diagnostics),
      });
      for (const waiter of this.waiters) waiter();
    });
  }

  wait(doc: OpenDocument, until: (diagnostics: Diagnostic[]) => boolean, timeoutMs: number) {
    return new Promise<{ diagnostics: Diagnostic[]; satisfied: boolean }>((resolve) => {
      let last: Diagnostic[] = [];
      const check = () => {
        const current = this.latest.get(doc.uri);
        // Unversioned publishes describe the open buffer too (e.g. dependency refreshes).
        if (!current || (current.version != null && current.version !== doc.version)) return;
        last = current.items;
        if (until(current.items)) finish(true);
      };
      const timer = setTimeout(() => finish(false), timeoutMs);
      const finish = (satisfied: boolean) => {
        clearTimeout(timer);
        this.waiters.delete(check);
        resolve({ diagnostics: last, satisfied });
      };
      this.waiters.add(check);
      check();
    });
  }
}

export function spawnPeer(
  command: string,
  args: string[],
  name: string,
  env: Record<string, string>,
  cwd: string,
) {
  const child = spawn(command, args, {
    cwd,
    env: { ...process.env, ...env },
    stdio: ["pipe", "pipe", "pipe"],
  });
  return new LspPeer(child, name);
}

export function exitOf(child: {
  exitCode: number | null;
  signalCode: string | null;
  once: Function;
}): Promise<void> {
  if (child.exitCode != null || child.signalCode != null) return Promise.resolve();
  return new Promise<void>((resolve) => child.once("exit", () => resolve()));
}

export async function stopLsp(peer: LspPeer): Promise<void> {
  if (peer.exit) return;
  const exited = exitOf(peer.child);
  try {
    await peer.request("shutdown", null, 10_000);
    peer.notify("exit", null);
  } catch {
    // A server that cannot shut down is killed below; its numbers are already taken.
  }
  const killer = setTimeout(() => peer.child.kill("SIGKILL"), 10_000);
  await exited;
  clearTimeout(killer);
}

export class VizeEditor implements Editor {
  readonly kind: Editor["kind"] = "vize";
  protected peer!: LspPeer;
  protected published!: PublishedDiagnostics;
  readonly label: string;
  protected readonly binary: string;
  protected readonly environment: EditorEnvironment;

  constructor(label: string, binary: string, environment: EditorEnvironment) {
    this.label = label;
    this.binary = binary;
    this.environment = environment;
  }

  pids(): number[] {
    return this.peer?.child.pid ? [this.peer.child.pid] : [];
  }

  async start(): Promise<void> {
    const { workspace, env } = this.environment;
    this.peer = spawnPeer(this.binary, ["lsp"], this.label, env, workspace);
    this.published = new PublishedDiagnostics(this.peer);
    // The extension's default profile when Vize is enabled without explicit switches.
    const options = { lint: true, typecheck: true, editor: true, ecosystem: true };
    await this.peer.request("initialize", initializeParams(workspace, options), 300_000);
    this.peer.notify("initialized", {});
  }

  open(doc: OpenDocument): void {
    this.peer.notify("textDocument/didOpen", {
      textDocument: { uri: doc.uri, languageId: "vue", version: doc.version, text: doc.text },
    });
  }

  change(doc: OpenDocument, edit: TextEdit): void {
    this.peer.notify("textDocument/didChange", {
      textDocument: { uri: doc.uri, version: doc.version },
      contentChanges: [{ range: edit.range, text: edit.text }],
    });
  }

  diagnostics(doc: OpenDocument, until: (diagnostics: Diagnostic[]) => boolean, timeoutMs: number) {
    return this.published.wait(doc, until, timeoutMs);
  }

  async hover(doc: OpenDocument, position: Position) {
    return hoverText(
      await this.peer.request("textDocument/hover", { textDocument: { uri: doc.uri }, position }),
    );
  }

  async completion(doc: OpenDocument, position: Position) {
    const context = { triggerKind: 1 };
    return completionLabels(
      await this.peer.request("textDocument/completion", {
        textDocument: { uri: doc.uri },
        position,
        context,
      }),
    );
  }

  async definition(doc: OpenDocument, position: Position) {
    return lspLocations(
      await this.peer.request("textDocument/definition", {
        textDocument: { uri: doc.uri },
        position,
      }),
    );
  }

  async references(doc: OpenDocument, position: Position) {
    const params = {
      textDocument: { uri: doc.uri },
      position,
      context: { includeDeclaration: true },
    };
    return lspLocations(await this.peer.request("textDocument/references", params));
  }

  async rename(doc: OpenDocument, position: Position, newName: string) {
    const target = { textDocument: { uri: doc.uri }, position };
    const prepared = await this.peer.request("textDocument/prepareRename", target);
    if (prepared == null) return [];
    return workspaceEditLocations(
      await this.peer.request("textDocument/rename", { ...target, newName }),
    );
  }

  async semanticTokens(doc: OpenDocument) {
    const result = await this.peer.request<{ data?: number[] } | null>(
      "textDocument/semanticTokens/full",
      {
        textDocument: { uri: doc.uri },
      },
    );
    return (result?.data?.length ?? 0) / 5;
  }

  stop() {
    return stopLsp(this.peer);
  }

  stderr() {
    return this.peer?.stderr ?? "";
  }

  exited() {
    return this.peer?.exit ?? null;
  }
}

/**
 * TypeScript 7.1's own `tsc --lsp` checking `.vue` through Vize's content
 * mapper (the same mapper as `tsc --runExternalCode` batch checks): no Vize
 * server, mirror or second program. Diagnostics are pulled, as tsgo serves them.
 */
export class TsgoMapperEditor extends VizeEditor {
  override readonly kind: Editor["kind"] = "tsgo";

  override async start(): Promise<void> {
    const { workspace, env } = this.environment;
    this.peer = spawnPeer(this.binary, ["--lsp", "--stdio"], this.label, env, workspace);
    this.published = new PublishedDiagnostics(this.peer);
    const params = initializeParams(workspace, { runExternalCode: true });
    (params.capabilities.textDocument as Record<string, unknown>).diagnostic = {
      dynamicRegistration: false,
    };
    await this.peer.request("initialize", params, 300_000);
    this.peer.notify("initialized", {});
  }

  /** One pull answers for the latest buffer: tsgo applies edits before it. */
  override async diagnostics(
    doc: OpenDocument,
    until: (diagnostics: Diagnostic[]) => boolean,
    timeoutMs: number,
  ) {
    const report = await this.peer.request<{ items?: unknown[] } | null>(
      "textDocument/diagnostic",
      { textDocument: { uri: doc.uri } },
      timeoutMs,
    );
    const diagnostics = lspDiagnostics((report?.items ?? []) as any[]);
    return { diagnostics, satisfied: until(diagnostics) };
  }
}
