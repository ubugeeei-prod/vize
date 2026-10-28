import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { collectCleanupFailures, initializeWithCleanupCustody } from "./dual-lsp-report.ts";
import { DualLspWire, type ClientOperation, type WireLaunch } from "./dual-lsp-wire.ts";

export type ProductKind = "vize" | "vue-language-tools";

export type ProductResponse = {
  provider: string;
  result: unknown;
};

export type PrimaryOperationReceipt = {
  queryPackets: number;
  diagnosticBarriersPerRevision: number;
  totalOperations: number;
  methodOrder: string[];
  methodCounts: Record<string, number>;
};

export class BackgroundTaskCustody {
  private readonly pending = new Set<Promise<void>>();
  private readonly failures: Error[] = [];

  track(task: Promise<void>): Promise<void> {
    const guarded = task.catch((error: unknown) => {
      this.failures.push(
        error instanceof Error ? error : new Error("background task rejected with a non-Error"),
      );
      throw error;
    });
    this.pending.add(guarded);
    void guarded.then(
      () => {
        this.pending.delete(guarded);
      },
      () => {
        this.pending.delete(guarded);
      },
    );
    return guarded;
  }

  assertHealthy(): void {
    if (this.failures.length === 0) return;
    throw new AggregateError(this.failures, "background task custody failed");
  }

  async settle(): Promise<void> {
    await new Promise<void>((resolve) => setImmediate(resolve));
    while (this.pending.size > 0) {
      await Promise.allSettled(this.pending);
      await new Promise<void>((resolve) => setImmediate(resolve));
    }
    this.assertHealthy();
  }
}

type ResolvedTools = {
  pluginRoot: string;
  tsdk: string;
  tsLanguageServer: string;
  vueLanguageServer: string;
};

let launchSerial = 0;

export const dualLspProviderRoot = path.resolve(
  process.env.LSP_DUAL_PROVIDER_ROOT ??
    path.join(
      path.dirname(fileURLToPath(import.meta.url)),
      "../../_fixtures/performance/lsp-dual-server/provider",
    ),
);

function resolveTools(): ResolvedTools {
  const require = createRequire(path.join(dualLspProviderRoot, "package.json"));
  const pluginRoot = path.dirname(require.resolve("@vue/typescript-plugin/package.json"));
  const vuePackageRoot = path.dirname(require.resolve("@vue/language-server/package.json"));
  const tsLanguageServerRoot = path.dirname(
    require.resolve("typescript-language-server/package.json"),
  );
  const typescriptRoot = path.dirname(require.resolve("typescript/package.json"));
  return {
    pluginRoot,
    tsdk: path.join(typescriptRoot, "lib"),
    tsLanguageServer: path.join(tsLanguageServerRoot, "lib/cli.mjs"),
    vueLanguageServer: path.join(vuePackageRoot, "bin/vue-language-server.js"),
  };
}

export function commonInitializeParams(
  workspaceDir: string,
  tsdk: string,
): Record<string, unknown> {
  const rootUri = pathToFileURL(workspaceDir).href;
  return {
    processId: process.pid,
    rootUri,
    capabilities: {
      workspace: { configuration: true, workspaceFolders: true },
      textDocument: {
        completion: {
          completionItem: {
            documentationFormat: ["markdown", "plaintext"],
            insertReplaceSupport: true,
            snippetSupport: true,
          },
        },
        definition: { linkSupport: false },
        hover: { contentFormat: ["markdown", "plaintext"] },
        publishDiagnostics: { relatedInformation: true, versionSupport: true },
        rename: { prepareSupport: true },
      },
    },
    initializationOptions: {
      completion: true,
      definition: true,
      editor: true,
      hover: true,
      lint: false,
      references: true,
      rename: true,
      typecheck: true,
      typescript: { tsdk },
      vue: { hybridMode: true },
    },
    workspaceFolders: [{ uri: rootUri, name: path.basename(workspaceDir) }],
  };
}

function profiledNodeArgs(script: string, args: string[]): string[] {
  const directory = process.env.LSP_DUAL_VUE_CPU_PROFILE_DIR;
  if (!directory) return [script, ...args];
  fs.mkdirSync(directory, { recursive: true });
  return ["--cpu-prof", `--cpu-prof-dir=${directory}`, script, ...args];
}

function vizeLaunch(workspaceDir: string): WireLaunch {
  const binary = process.env.VIZE_LSP_BIN;
  assert.ok(
    binary && path.isAbsolute(binary),
    "VIZE_LSP_BIN must be an absolute source-built binary",
  );
  const args = ["lsp"];
  const profileDir = process.env.LSP_DUAL_VIZE_PROFILE_DIR;
  if (profileDir) {
    fs.mkdirSync(profileDir, { recursive: true });
    args.push("--profile-json", path.join(profileDir, `session-${++launchSerial}.json`));
  }
  return { name: "vize", command: binary, args, cwd: workspaceDir };
}

function vueLaunches(workspaceDir: string, tools: ResolvedTools): [WireLaunch, WireLaunch] {
  const serial = ++launchSerial;
  return [
    {
      name: `vue-language-server-${serial}`,
      command: process.execPath,
      args: profiledNodeArgs(tools.vueLanguageServer, ["--stdio", `--tsdk=${tools.tsdk}`]),
      cwd: workspaceDir,
    },
    {
      name: `typescript-language-server-${serial}`,
      command: process.execPath,
      args: profiledNodeArgs(tools.tsLanguageServer, ["--stdio", "--log-level", "1"]),
      cwd: workspaceDir,
    },
  ];
}

function unwrapTuple(params: unknown): [number, string, unknown] {
  const outer = params as unknown[];
  const tuple =
    Array.isArray(outer) && outer.length === 1 && Array.isArray(outer[0]) ? outer[0] : outer;
  assert.ok(
    Array.isArray(tuple) && tuple.length >= 3,
    `invalid tsserver/request tuple: ${JSON.stringify(params)}`,
  );
  assert.equal(typeof tuple[0], "number");
  assert.equal(typeof tuple[1], "string");
  return [tuple[0] as number, tuple[1] as string, tuple[2]];
}

/** A product boundary: one Vize process, or the complete current Vue+TS stack. */
export class DualLspProduct {
  readonly kind: ProductKind;
  private readonly primary: DualLspWire;
  private readonly semantic?: DualLspWire;
  private readonly bridgeCustody = new BackgroundTaskCustody();

  private constructor(kind: ProductKind, primary: DualLspWire, semantic?: DualLspWire) {
    this.kind = kind;
    this.primary = primary;
    this.semantic = semantic;
    if (semantic) this.installBridge(semantic);
  }

  static async start(kind: ProductKind, workspaceDir: string): Promise<DualLspProduct> {
    const tools = resolveTools();
    if (kind === "vize") {
      const primary = new DualLspWire(vizeLaunch(workspaceDir));
      const product = new DualLspProduct(kind, primary);
      return await initializeWithCleanupCustody(
        async () => {
          await primary.initialize(commonInitializeParams(workspaceDir, tools.tsdk));
          return product;
        },
        () => product.kill(),
        "Vize initialization cleanup failed",
      );
    }

    const [vueLaunch, tsLaunch] = vueLaunches(workspaceDir, tools);
    const primary = new DualLspWire(vueLaunch);
    const semantic = new DualLspWire(tsLaunch);
    const product = new DualLspProduct(kind, primary, semantic);
    const common = commonInitializeParams(workspaceDir, tools.tsdk);
    return await initializeWithCleanupCustody(
      async () => {
        await semantic.initialize(
          {
            ...common,
            initializationOptions: {
              plugins: [
                {
                  name: "@vue/typescript-plugin",
                  location: tools.pluginRoot,
                  languages: ["vue"],
                },
              ],
              tsserver: { path: tools.tsdk },
            },
          },
          false,
        );
        await primary.initialize(common);
        return product;
      },
      () => product.kill(),
      "Vue stack initialization cleanup failed",
    );
  }

  get diagnosticWires(): ReadonlyArray<{ provider: string; wire: DualLspWire }> {
    return [
      { provider: this.primary.launch.name, wire: this.primary },
      ...(this.semantic ? [{ provider: this.semantic.launch.name, wire: this.semantic }] : []),
    ];
  }

  get expectedDiagnosticWire(): DualLspWire {
    return this.semantic ?? this.primary;
  }

  get processIds(): number[] {
    return [this.primary.processId, ...(this.semantic ? [this.semantic.processId] : [])];
  }

  primaryOperations(): readonly ClientOperation[] {
    return this.primary.clientOperations();
  }

  open(uri: string, source: string, version: number): void {
    const params = { textDocument: { uri, languageId: "vue", version, text: source } };
    this.primary.notify("textDocument/didOpen", params);
    this.semantic?.notify("textDocument/didOpen", params, false);
  }

  change(uri: string, source: string, version: number): void {
    const params = {
      textDocument: { uri, version },
      contentChanges: [{ text: source }],
    };
    this.primary.notify("textDocument/didChange", params);
    this.semantic?.notify("textDocument/didChange", params, false);
  }

  async requestAll(
    method: string,
    params: unknown,
    timeoutMs = 60_000,
  ): Promise<ProductResponse[]> {
    this.bridgeCustody.assertHealthy();
    assert.ok(this.primary.supports(method), `${this.kind} primary did not advertise ${method}`);
    const requests = [
      this.primary.request(method, params, timeoutMs).then((result) => ({
        provider: this.primary.launch.name,
        result,
      })),
    ];
    if (this.semantic?.supports(method)) {
      requests.push(
        this.semantic.request(method, params, timeoutMs, false).then((result) => ({
          provider: this.semantic!.launch.name,
          result,
        })),
      );
    }
    const result = await Promise.all(requests);
    await this.settleBackgroundWork();
    this.bridgeCustody.assertHealthy();
    return result;
  }

  async settleBackgroundWork(): Promise<void> {
    await this.bridgeCustody.settle();
  }

  async diagnosticBarrier(uri: string): Promise<void> {
    await this.requestAll("textDocument/documentSymbol", { textDocument: { uri } });
  }

  async shutdown(): Promise<void> {
    const failures: unknown[] = [];
    try {
      await this.settleBackgroundWork();
    } catch (error) {
      failures.push(error);
    }
    const wires = [this.primary, this.semantic].filter((item): item is DualLspWire => item != null);
    failures.push(...(await collectCleanupFailures(wires.map((wire) => () => wire.shutdown()))));
    try {
      await this.settleBackgroundWork();
    } catch (error) {
      failures.push(error);
    }
    if (failures.length > 0) throw new AggregateError(failures, `${this.kind} shutdown failed`);
  }

  async kill(): Promise<void> {
    const wires = [this.primary, this.semantic].filter((item): item is DualLspWire => item != null);
    const failures = await collectCleanupFailures(wires.map((wire) => () => wire.kill()));
    if (failures.length > 0) throw new AggregateError(failures, `${this.kind} kill failed`);
  }

  private installBridge(semantic: DualLspWire): void {
    this.primary.addNotificationObserver((notification) => {
      if (notification.method !== "tsserver/request") return;
      const task = this.bridgeCustody.track(
        (async () => {
          const [id, command, payload] = unwrapTuple(notification.params);
          const response = await semantic.request(
            "workspace/executeCommand",
            { command: "typescript.tsserverRequest", arguments: [command, payload] },
            60_000,
            false,
          );
          const body =
            response != null && typeof response === "object" && "body" in response
              ? (response as { body?: unknown }).body
              : response;
          this.primary.notify("tsserver/response", [[id, body]], false);
        })(),
      );
      return task;
    });
  }
}

export function productOrder(round: number): readonly ProductKind[] {
  return round % 2 === 0 ? ["vize", "vue-language-tools"] : ["vue-language-tools", "vize"];
}

export function stableOperationTranscript(operations: readonly ClientOperation[]): string {
  return JSON.stringify(operations);
}

export function primaryOperationReceipt(
  operations: readonly ClientOperation[],
  queryPackets: number,
  diagnosticBarriersPerRevision: number,
): PrimaryOperationReceipt {
  assert.ok(Number.isSafeInteger(queryPackets) && queryPackets >= 0);
  assert.ok(
    Number.isSafeInteger(diagnosticBarriersPerRevision) && diagnosticBarriersPerRevision > 0,
  );
  const expected: Array<Pick<ClientOperation, "kind" | "method">> = [
    { kind: "request", method: "initialize" },
    { kind: "notification", method: "initialized" },
    { kind: "notification", method: "textDocument/didOpen" },
    ...Array.from({ length: diagnosticBarriersPerRevision }, () => ({
      kind: "request" as const,
      method: "textDocument/documentSymbol",
    })),
    { kind: "notification", method: "textDocument/didChange" },
    ...Array.from({ length: diagnosticBarriersPerRevision }, () => ({
      kind: "request" as const,
      method: "textDocument/documentSymbol",
    })),
  ];
  const queryMethods = [
    "textDocument/completion",
    "textDocument/hover",
    "textDocument/definition",
    "textDocument/references",
    "textDocument/prepareRename",
    "textDocument/rename",
  ];
  for (let packet = 0; packet < queryPackets; packet += 1) {
    expected.push(...queryMethods.map((method) => ({ kind: "request" as const, method })));
  }
  const actual = operations.map(({ kind, method }) => ({ kind, method }));
  assert.deepEqual(actual, expected, "primary request/notification method order drifted");
  const methodOrder = actual.map(({ kind, method }) => `${kind} ${method}`);
  const methodCounts: Record<string, number> = {};
  for (const operation of methodOrder) methodCounts[operation] = (methodCounts[operation] ?? 0) + 1;
  return {
    queryPackets,
    diagnosticBarriersPerRevision,
    totalOperations: methodOrder.length,
    methodOrder,
    methodCounts,
  };
}

export function dualLspToolReceipt(): Record<string, unknown> {
  const require = createRequire(path.join(dualLspProviderRoot, "package.json"));
  const tools = resolveTools();
  const fileReceipt = (filePath: string) => ({
    path: filePath,
    sha256: createHash("sha256").update(fs.readFileSync(filePath)).digest("hex"),
  });
  const packageReceipt = (name: string, entrypoint: string) => {
    const manifestPath = require.resolve(`${name}/package.json`);
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as { version: string };
    return {
      version: manifest.version,
      manifest: fileReceipt(manifestPath),
      entrypoint: fileReceipt(entrypoint),
    };
  };
  const vueLanguageTools = packageReceipt("@vue/language-server", tools.vueLanguageServer);
  const typescriptLanguageServer = packageReceipt(
    "typescript-language-server",
    tools.tsLanguageServer,
  );
  const vueTypescriptPlugin = packageReceipt(
    "@vue/typescript-plugin",
    require.resolve("@vue/typescript-plugin"),
  );
  const typescript = packageReceipt("typescript", require.resolve("typescript"));
  return {
    providerRoot: dualLspProviderRoot,
    vize: {
      binary: process.env.VIZE_LSP_BIN,
      args: ["lsp"],
      allocationProfile: process.env.LSP_DUAL_VIZE_PROFILE_DIR ?? null,
    },
    vueLanguageTools: {
      package: "@vue/language-server",
      ...vueLanguageTools,
      command: process.execPath,
      args: [tools.vueLanguageServer, "--stdio", `--tsdk=${tools.tsdk}`],
      cpuProfile: process.env.LSP_DUAL_VUE_CPU_PROFILE_DIR ?? null,
    },
    typescriptLanguageServer: {
      package: "typescript-language-server",
      ...typescriptLanguageServer,
      command: process.execPath,
      args: [tools.tsLanguageServer, "--stdio", "--log-level", "1"],
    },
    vueTypescriptPlugin: {
      package: "@vue/typescript-plugin",
      ...vueTypescriptPlugin,
      location: tools.pluginRoot,
    },
    typescript: { ...typescript, tsdk: tools.tsdk },
    architecture:
      "Vue Language Tools v3 is measured as Vue LS plus TypeScript LS with @vue/typescript-plugin and the tsserver request bridge",
  };
}
