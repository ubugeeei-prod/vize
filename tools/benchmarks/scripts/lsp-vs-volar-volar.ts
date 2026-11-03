/**
 * What VS Code runs for "Vue (Official)": `@vue/language-server` plus one
 * tsserver with `@vue/typescript-plugin`, the language server's
 * `tsserver/request` notifications forwarded to that tsserver. VS Code asks
 * every provider of a feature at once and waits for all of them, so a request
 * costs the slower provider; rename takes the first provider that answers with
 * edits. The setup is the lower bound of VS Code's: no syntax server, no
 * typings installer, no client-side diagnostic debounce.
 */

import { join } from "node:path";

import {
  type Diagnostic,
  type Editor,
  type EditorEnvironment,
  type Location,
  completionLabels,
  hoverText,
  initializeParams,
  lspLocations,
  spawnPeer,
  stopLsp,
  workspaceEditLocations,
} from "./lsp-vs-volar-editors.ts";
import type { OpenDocument, Position, TextEdit } from "./lsp-vs-volar-text.ts";
import { TsServerEditor } from "./lsp-vs-volar-tsserver.ts";
import type { LspPeer } from "./lsp-vs-volar-wire.ts";

export interface VolarInstall {
  /** Directory whose `node_modules` holds `@vue/language-server`, its plugin and `typescript`. */
  root: string;
}

export class VolarEditor implements Editor {
  readonly kind = "volar" as const;
  private vue!: LspPeer;
  private readonly ts: TsServerEditor;
  /** Provider failures VS Code would log and skip; the other provider still answers. */
  providerErrors = 0;
  readonly label: string;
  private readonly install: VolarInstall;
  private readonly environment: EditorEnvironment;

  constructor(label: string, install: VolarInstall, environment: EditorEnvironment) {
    this.label = label;
    this.install = install;
    this.environment = environment;
    this.ts = new TsServerEditor(
      label,
      { root: install.root, plugin: "@vue/typescript-plugin" },
      environment,
    );
  }

  pids(): number[] {
    return [this.vue?.child.pid, ...this.ts.pids()].filter((pid): pid is number => pid != null);
  }

  async start(): Promise<void> {
    const { workspace, env } = this.environment;
    const modules = join(this.install.root, "node_modules");
    const tsdk = this.ts.tsdk;
    const tsStarted = this.ts.start();
    this.vue = spawnPeer(
      process.execPath,
      [
        join(modules, "@vue/language-server/bin/vue-language-server.js"),
        "--stdio",
        `--tsdk=${tsdk}`,
      ],
      this.label,
      env,
      workspace,
    );
    // The Vue extension's bridge: `typescript.tsserverRequest`, then the body back.
    // vscode-jsonrpc sends a lone array argument wrapped: `params: [[id, command, args]]`.
    this.vue.onNotification(
      "tsserver/request",
      ([[id, command, args]]: [[number, string, unknown]]) => {
        // `configure` was written first, so tsserver applies it before this.
        this.ts.ts.request(command, args).then(
          (body) => this.vue.notify("tsserver/response", [[id, body ?? null]]),
          () => {},
        );
      },
    );
    await Promise.all([
      tsStarted,
      this.vue.request(
        "initialize",
        initializeParams(workspace, { typescript: { tsdk } }),
        300_000,
      ),
    ]);
    this.vue.notify("initialized", {});
  }

  open(doc: OpenDocument): void {
    this.vue.notify("textDocument/didOpen", {
      textDocument: { uri: doc.uri, languageId: "vue", version: doc.version, text: doc.text },
    });
    this.ts.open(doc);
  }

  change(doc: OpenDocument, edit: TextEdit): void {
    this.vue.notify("textDocument/didChange", {
      textDocument: { uri: doc.uri, version: doc.version },
      contentChanges: [{ range: edit.range, text: edit.text }],
    });
    this.ts.change(doc, edit);
  }

  diagnostics(doc: OpenDocument, until: (diagnostics: Diagnostic[]) => boolean, timeoutMs: number) {
    return this.ts.diagnostics(doc, until, timeoutMs);
  }

  private vueRequest<T = any>(method: string, params: unknown): Promise<T | null> {
    return this.vue.request<T>(method, params).catch(() => {
      this.providerErrors += 1;
      return null;
    });
  }

  async hover(doc: OpenDocument, position: Position) {
    const [vue, ts] = await Promise.all([
      this.vueRequest("textDocument/hover", { textDocument: { uri: doc.uri }, position }),
      this.ts.hover(doc, position),
    ]);
    return [hoverText(vue), ts].filter(Boolean).join("\n");
  }

  async completion(doc: OpenDocument, position: Position) {
    const [vue, ts] = await Promise.all([
      this.vueRequest("textDocument/completion", {
        textDocument: { uri: doc.uri },
        position,
        context: { triggerKind: 1 },
      }),
      this.ts.completion(doc, position),
    ]);
    return [...completionLabels(vue), ...ts];
  }

  async definition(doc: OpenDocument, position: Position) {
    const [vue, ts] = await Promise.all([
      this.vueRequest("textDocument/definition", { textDocument: { uri: doc.uri }, position }),
      this.ts.definition(doc, position),
    ]);
    return [...lspLocations(vue), ...ts];
  }

  async references(doc: OpenDocument, position: Position) {
    const params = {
      textDocument: { uri: doc.uri },
      position,
      context: { includeDeclaration: true },
    };
    const [vue, ts] = await Promise.all([
      this.vueRequest("textDocument/references", params),
      this.ts.references(doc, position),
    ]);
    return [...lspLocations(vue), ...ts];
  }

  /** VS Code renames with the first provider that offers edits. */
  async rename(doc: OpenDocument, position: Position, newName: string) {
    const target = { textDocument: { uri: doc.uri }, position };
    const viaVue = (async () => {
      const prepared = await this.vueRequest("textDocument/prepareRename", target);
      if (prepared == null) return [];
      return workspaceEditLocations(
        await this.vueRequest("textDocument/rename", { ...target, newName }),
      );
    })();
    const viaTs = this.ts.rename(doc, position, newName);
    return new Promise<Location[]>((resolve, reject) => {
      let pending = 2;
      const settle = (locations: Location[]) => {
        pending -= 1;
        if (locations.length || pending === 0) resolve(locations);
      };
      viaVue.then(settle, reject);
      viaTs.then(settle, reject);
    });
  }

  async semanticTokens(doc: OpenDocument) {
    const [vue, ts] = await Promise.all([
      this.vueRequest<{ data?: number[] }>("textDocument/semanticTokens/full", {
        textDocument: { uri: doc.uri },
      }),
      this.ts.semanticTokens(doc),
    ]);
    return (vue?.data?.length ?? 0) / 5 + ts;
  }

  async stop() {
    await Promise.all([stopLsp(this.vue), this.ts.stop()]);
  }

  stderr() {
    return `${this.vue?.stderr ?? ""}\n${this.ts.stderr()}`;
  }

  exited() {
    return this.vue?.exit ?? this.ts.exited();
  }
}
