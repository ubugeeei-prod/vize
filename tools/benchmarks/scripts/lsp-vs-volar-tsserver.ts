/**
 * One tsserver with one TypeScript plugin, driven over its own protocol the way
 * VS Code's TypeScript extension drives it. Volar pairs it with
 * `@vue/typescript-plugin`. On its own with `@vizejs/typescript-vue-plugin` it
 * is Vize's backend for the project's TypeScript 5.7–6.x. tsserver may grow to
 * 8 GiB (VS Code's default cap is 3 GiB) so a large project cannot fail it.
 */

import { spawn } from "node:child_process";
import { join } from "node:path";

import {
  type Diagnostic,
  type Editor,
  type EditorEnvironment,
  type Location,
  exitOf,
} from "./lsp-vs-volar-editors.ts";
import type { OpenDocument, Position, TextEdit } from "./lsp-vs-volar-text.ts";
import { TsServerPeer } from "./lsp-vs-volar-wire.ts";

export interface TsServerInstall {
  /** Directory whose `node_modules` holds `typescript` and `plugin`. */
  root: string;
  plugin: string;
}

/** tsserver lines and offsets are 1-based. */
function tsLocation(position: Position) {
  return { line: position.line + 1, offset: position.character + 1 };
}

function fromTs(file: string, start: { line: number; offset: number }): Location {
  return { path: file, line: start.line - 1, character: start.offset - 1 };
}

export class TsServerEditor implements Editor {
  readonly kind: Editor["kind"] = "tsserver";
  ts!: TsServerPeer;
  readonly label: string;
  private readonly install: TsServerInstall;
  private readonly environment: EditorEnvironment;

  constructor(label: string, install: TsServerInstall, environment: EditorEnvironment) {
    this.label = label;
    this.install = install;
    this.environment = environment;
  }

  get tsdk(): string {
    return join(this.install.root, "node_modules/typescript/lib");
  }

  pids(): number[] {
    return this.ts?.child.pid ? [this.ts.child.pid] : [];
  }

  async start(): Promise<void> {
    const { workspace, env } = this.environment;
    const diagnosticProfile = env.XDG_CACHE_HOME
      ? join(env.XDG_CACHE_HOME, "vize/bench/tsserver-diagnostic-phases.json")
      : undefined;
    const child = spawn(
      process.execPath,
      [
        "--max-old-space-size=8192",
        join(this.tsdk, "tsserver.js"),
        "--useInferredProjectPerProjectRoot",
        "--disableAutomaticTypingAcquisition",
        "--noGetErrOnBackgroundUpdate",
        "--locale",
        "en",
        "--globalPlugins",
        this.install.plugin,
        "--pluginProbeLocations",
        this.install.root,
      ],
      {
        cwd: workspace,
        env: {
          ...process.env,
          ...env,
          ...(diagnosticProfile
            ? { VIZE_TYPESCRIPT_PLUGIN_DIAGNOSTIC_PROFILE: diagnosticProfile }
            : {}),
        },
        stdio: ["pipe", "pipe", "pipe"],
      },
    );
    this.ts = new TsServerPeer(child);
    await this.ts.request("configure", {
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
    });
  }

  open(doc: OpenDocument): void {
    this.fireAndForget("updateOpen", {
      openFiles: [
        { file: doc.path, fileContent: doc.text, projectRootPath: this.environment.workspace },
      ],
    });
  }

  change(doc: OpenDocument, edit: TextEdit): void {
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

  async hover(doc: OpenDocument, position: Position) {
    const body = await this.ts.request("quickinfo", { file: doc.path, ...tsLocation(position) });
    if (!body) return "";
    // `displayPartsForJSDoc` makes documentation display parts.
    const documentation = Array.isArray(body.documentation)
      ? body.documentation.map((part: { text: string }) => part.text).join("")
      : body.documentation;
    return [body.displayString, documentation].filter(Boolean).join("\n");
  }

  async completion(doc: OpenDocument, position: Position) {
    const body = await this.ts.request("completionInfo", {
      file: doc.path,
      ...tsLocation(position),
      includeExternalModuleExports: true,
      includeInsertTextCompletions: true,
    });
    return (body?.entries ?? []).map((entry: { name: string }) => entry.name);
  }

  async definition(doc: OpenDocument, position: Position) {
    const body = await this.ts.request("definitionAndBoundSpan", {
      file: doc.path,
      ...tsLocation(position),
    });
    return (body?.definitions ?? []).map((item: any) => fromTs(item.file, item.start));
  }

  async references(doc: OpenDocument, position: Position) {
    const body = await this.ts.request("references", { file: doc.path, ...tsLocation(position) });
    return (body?.refs ?? []).map((item: any) => fromTs(item.file, item.start));
  }

  async rename(doc: OpenDocument, position: Position, _newName: string) {
    const body = await this.ts.request("rename", {
      file: doc.path,
      ...tsLocation(position),
      findInStrings: false,
      findInComments: false,
    });
    return (body?.locs ?? []).flatMap((group: any) =>
      group.locs.map((item: any) => fromTs(group.file, item.start)),
    );
  }

  async semanticTokens(doc: OpenDocument) {
    const body = await this.ts.request("encodedSemanticClassifications-full", {
      file: doc.path,
      start: 0,
      length: doc.text.length,
      format: "2020",
    });
    return (body?.spans?.length ?? 0) / 3;
  }

  /** Resolves once tsserver has exited; `kill` follows if it lingers. */
  async stop() {
    const exited = exitOf(this.ts.child);
    if (!this.ts.exit)
      this.ts.child.stdin.write(
        `${JSON.stringify({ seq: 0, type: "request", command: "exit" })}\n`,
      );
    const killer = setTimeout(() => this.ts.child.kill("SIGKILL"), 10_000);
    await exited;
    clearTimeout(killer);
  }

  stderr() {
    return this.ts?.stderr ?? "";
  }

  exited() {
    return this.ts?.exit ?? null;
  }
}
