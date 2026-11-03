/**
 * What VS Code runs for "Vue (Official)": `@vue/language-server` plus one
 * tsserver with `@vue/typescript-plugin`, the language server's
 * `tsserver/request` notifications forwarded to that tsserver. VS Code asks
 * every provider of a feature at once and waits for all of them, so a request
 * costs the slower provider; rename takes the first provider that answers with
 * edits. The setup is the lower bound of VS Code's: no syntax server, no
 * typings installer, no client-side diagnostic debounce. tsserver may grow to
 * 8 GiB (VS Code's default cap is 3 GiB) so a large project cannot fail it.
 */

import { spawn } from "node:child_process";
import { join } from "node:path";

import {
  type Diagnostic,
  type Editor,
  type EditorEnvironment,
  type Location,
  completionLabels,
  exitOf,
  hoverText,
  initializeParams,
  lspLocations,
  spawnPeer,
  stopLsp,
  workspaceEditLocations,
} from "./lsp-vs-volar-editors.ts";
import type { OpenDocument, Position, TextEdit } from "./lsp-vs-volar-text.ts";
import { type LspPeer, TsServerPeer } from "./lsp-vs-volar-wire.ts";

export interface VolarInstall {
  /** Directory whose `node_modules` holds `@vue/language-server`, its plugin and `typescript`. */
  root: string;
}

/** tsserver lines and offsets are 1-based. */
function tsLocation(position: Position) {
  return { line: position.line + 1, offset: position.character + 1 };
}

function fromTs(file: string, start: { line: number; offset: number }): Location {
  return { path: file, line: start.line - 1, character: start.offset - 1 };
}

export class VolarEditor implements Editor {
  readonly kind = "volar" as const;
  private vue!: LspPeer;
  private ts!: TsServerPeer;
  /** Provider failures VS Code would log and skip; the other provider still answers. */
  providerErrors = 0;
  readonly label: string;
  private readonly install: VolarInstall;
  private readonly environment: EditorEnvironment;

  constructor(label: string, install: VolarInstall, environment: EditorEnvironment) {
    this.label = label;
    this.install = install;
    this.environment = environment;
  }

  pids(): number[] {
    return [this.vue?.child.pid, this.ts?.child.pid].filter((pid): pid is number => pid != null);
  }

  async start(): Promise<void> {
    const { workspace, env } = this.environment;
    const modules = join(this.install.root, "node_modules");
    const tsdk = join(modules, "typescript/lib");
    const tsChild = spawn(
      process.execPath,
      [
        "--max-old-space-size=8192",
        join(tsdk, "tsserver.js"),
        "--useInferredProjectPerProjectRoot",
        "--disableAutomaticTypingAcquisition",
        "--noGetErrOnBackgroundUpdate",
        "--locale",
        "en",
        "--globalPlugins",
        "@vue/typescript-plugin",
        "--pluginProbeLocations",
        this.install.root,
      ],
      { cwd: workspace, env: { ...process.env, ...env }, stdio: ["pipe", "pipe", "pipe"] },
    );
    this.ts = new TsServerPeer(tsChild);
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
        this.ts.request(command, args).then(
          (body) => this.vue.notify("tsserver/response", [[id, body ?? null]]),
          () => {},
        );
      },
    );
    await Promise.all([
      this.ts.request("configure", {
        hostInfo: "vscode",
        preferences: {
          providePrefixAndSuffixTextForRename: true,
          allowRenameOfImportPath: true,
          includeAutomaticOptionalChainCompletions: true,
          includeCompletionsForModuleExports: true,
          includeCompletionsWithInsertText: true,
          includeCompletionsWithSnippetText: true,
          useLabelDetailsInCompletionEntries: true,
          allowIncompleteCompletions: true,
          displayPartsForJSDoc: true,
        },
      }),
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
    this.fireAndForget("updateOpen", {
      openFiles: [
        { file: doc.path, fileContent: doc.text, projectRootPath: this.environment.workspace },
      ],
    });
  }

  change(doc: OpenDocument, edit: TextEdit): void {
    this.vue.notify("textDocument/didChange", {
      textDocument: { uri: doc.uri, version: doc.version },
      contentChanges: [{ range: edit.range, text: edit.text }],
    });
    this.fireAndForget("updateOpen", {
      changedFiles: [
        {
          fileName: doc.path,
          textChanges: [
            {
              start: tsLocation(edit.range.start),
              end: tsLocation(edit.range.end),
              newText: edit.text,
            },
          ],
        },
      ],
    });
  }

  /** One `geterr` answers for the latest buffer: tsserver applies edits before it. */
  async diagnostics(
    doc: OpenDocument,
    until: (diagnostics: Diagnostic[]) => boolean,
    timeoutMs: number,
  ) {
    const collected = new Map<string, Diagnostic[]>();
    const { seq, response } = this.ts.send("geterr", { files: [doc.path], delay: 0 });
    response.catch(() => {});
    let unsubscribe = () => {};
    const done = new Promise<void>((resolve) => {
      unsubscribe = this.ts.onEvent((event, body) => {
        if ((event === "syntaxDiag" || event === "semanticDiag") && body?.file === doc.path) {
          collected.set(
            event,
            (body.diagnostics ?? []).map((item: any) => ({
              code: String(item.code ?? ""),
              line: item.start.line - 1,
              message: String(item.text ?? ""),
            })),
          );
        }
        if (event === "requestCompleted" && body?.request_seq === seq) resolve();
      });
    });
    let timer: NodeJS.Timeout | undefined;
    const timedOut = await Promise.race([
      done.then(() => false),
      new Promise<boolean>((resolve) => (timer = setTimeout(() => resolve(true), timeoutMs))),
    ]);
    clearTimeout(timer);
    unsubscribe();
    const diagnostics = [...collected.values()].flat();
    return { diagnostics, satisfied: !timedOut && until(diagnostics) };
  }

  /** Like VS Code's `executeWithoutWaitingForResponse`; a dead server surfaces at the next wait. */
  private fireAndForget(command: string, args: unknown): void {
    this.ts.request(command, args).catch(() => {});
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
      this.ts.request("quickinfo", { file: doc.path, ...tsLocation(position) }),
    ]);
    const tsText = ts ? [ts.displayString, ts.documentation].filter(Boolean).join("\n") : "";
    return [hoverText(vue), tsText].filter(Boolean).join("\n");
  }

  async completion(doc: OpenDocument, position: Position) {
    const [vue, ts] = await Promise.all([
      this.vueRequest("textDocument/completion", {
        textDocument: { uri: doc.uri },
        position,
        context: { triggerKind: 1 },
      }),
      this.ts.request("completionInfo", {
        file: doc.path,
        ...tsLocation(position),
        includeExternalModuleExports: true,
        includeInsertTextCompletions: true,
      }),
    ]);
    return [
      ...completionLabels(vue),
      ...(ts?.entries ?? []).map((entry: { name: string }) => entry.name),
    ];
  }

  async definition(doc: OpenDocument, position: Position) {
    const [vue, ts] = await Promise.all([
      this.vueRequest("textDocument/definition", { textDocument: { uri: doc.uri }, position }),
      this.ts.request("definitionAndBoundSpan", { file: doc.path, ...tsLocation(position) }),
    ]);
    const tsLocations = (ts?.definitions ?? []).map((item: any) => fromTs(item.file, item.start));
    return [...lspLocations(vue), ...tsLocations];
  }

  async references(doc: OpenDocument, position: Position) {
    const params = {
      textDocument: { uri: doc.uri },
      position,
      context: { includeDeclaration: true },
    };
    const [vue, ts] = await Promise.all([
      this.vueRequest("textDocument/references", params),
      this.ts.request("references", { file: doc.path, ...tsLocation(position) }),
    ]);
    const tsLocations = (ts?.refs ?? []).map((item: any) => fromTs(item.file, item.start));
    return [...lspLocations(vue), ...tsLocations];
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
    const viaTs = this.ts
      .request("rename", {
        file: doc.path,
        ...tsLocation(position),
        findInStrings: false,
        findInComments: false,
      })
      .then((body) =>
        (body?.locs ?? []).flatMap((group: any) =>
          group.locs.map((item: any) => fromTs(group.file, item.start)),
        ),
      );
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
      this.ts.request("encodedSemanticClassifications-full", {
        file: doc.path,
        start: 0,
        length: doc.text.length,
        format: "2020",
      }),
    ]);
    return (vue?.data?.length ?? 0) / 5 + (ts?.spans?.length ?? 0) / 3;
  }

  async stop() {
    const tsExited = exitOf(this.ts.child);
    if (!this.ts.exit)
      this.ts.child.stdin.write(
        `${JSON.stringify({ seq: 0, type: "request", command: "exit" })}\n`,
      );
    const killer = setTimeout(() => this.ts.child.kill("SIGKILL"), 10_000);
    await Promise.all([stopLsp(this.vue), tsExited]);
    clearTimeout(killer);
  }

  stderr() {
    return `${this.vue?.stderr ?? ""}\n${this.ts?.stderr ?? ""}`;
  }

  exited() {
    return this.vue?.exit ?? this.ts?.exit ?? null;
  }
}
