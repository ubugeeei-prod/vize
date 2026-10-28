import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { DualLspWire, type ClientOperation, type WireLaunch } from "./dual-lsp-wire.ts";

export type ProductKind = "vize" | "vue-language-tools";

export type ProductResponse = {
  provider: string;
  result: unknown;
};

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
  const tsLanguageServerRoot = path.dirname(require.resolve("typescript-language-server/package.json"));
  const typescriptRoot = path.dirname(require.resolve("typescript/package.json"));
  return {
    pluginRoot,
    tsdk: path.join(typescriptRoot, "lib"),
    tsLanguageServer: path.join(tsLanguageServerRoot, "lib/cli.mjs"),
    vueLanguageServer: path.join(vuePackageRoot, "bin/vue-language-server.js"),
  };
}

export function commonInitializeParams(workspaceDir: string, tsdk: string): Record<string, unknown> {
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
  assert.ok(binary && path.isAbsolute(binary), "VIZE_LSP_BIN must be an absolute source-built binary");
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
  const tuple = Array.isArray(outer) && outer.length === 1 && Array.isArray(outer[0]) ? outer[0] : outer;
  assert.ok(Array.isArray(tuple) && tuple.length >= 3, `invalid tsserver/request tuple: ${JSON.stringify(params)}`);
  assert.equal(typeof tuple[0], "number");
  assert.equal(typeof tuple[1], "string");
  return [tuple[0] as number, tuple[1] as string, tuple[2]];
}

/** A product boundary: one Vize process, or the complete current Vue+TS stack. */
export class DualLspProduct {
  private readonly semantic?: DualLspWire;
  private readonly bridgeFailures: Error[] = [];

  private constructor(
    readonly kind: ProductKind,
    private readonly primary: DualLspWire,
    semantic?: DualLspWire,
  ) {
    this.semantic = semantic;
    if (semantic) this.installBridge(semantic);
  }

  static async start(kind: ProductKind, workspaceDir: string): Promise<DualLspProduct> {
    const tools = resolveTools();
    if (kind === "vize") {
      const primary = new DualLspWire(vizeLaunch(workspaceDir));
      const product = new DualLspProduct(kind, primary);
      await primary.initialize(commonInitializeParams(workspaceDir, tools.tsdk));
      return product;
    }

    const [vueLaunch, tsLaunch] = vueLaunches(workspaceDir, tools);
    const primary = new DualLspWire(vueLaunch);
    const semantic = new DualLspWire(tsLaunch);
    const product = new DualLspProduct(kind, primary, semantic);
    const common = commonInitializeParams(workspaceDir, tools.tsdk);
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
  }

  get diagnosticWire(): DualLspWire {
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

  async requestAll(method: string, params: unknown, timeoutMs = 60_000): Promise<ProductResponse[]> {
    this.assertBridgeHealthy();
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
    this.assertBridgeHealthy();
    return result;
  }

  async diagnosticBarrier(uri: string): Promise<void> {
    await this.requestAll("textDocument/documentSymbol", { textDocument: { uri } });
  }

  async shutdown(): Promise<void> {
    let failure: unknown;
    try {
      await this.primary.shutdown();
    } catch (error) {
      failure = error;
      await this.primary.kill();
    }
    if (this.semantic) {
      try {
        await this.semantic.shutdown();
      } catch (error) {
        failure ??= error;
        await this.semantic.kill();
      }
    }
    if (failure != null) throw failure;
  }

  async kill(): Promise<void> {
    await this.primary.kill();
    if (this.semantic) await this.semantic.kill();
  }

  private installBridge(semantic: DualLspWire): void {
    this.primary.addNotificationObserver(async (notification) => {
      if (notification.method !== "tsserver/request") return;
      try {
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
      } catch (error) {
        this.bridgeFailures.push(error as Error);
        throw error;
      }
    });
  }

  private assertBridgeHealthy(): void {
    if (this.bridgeFailures.length === 0) return;
    throw new AggregateError(this.bridgeFailures, `${this.kind} tsserver bridge failed`);
  }
}

export function productOrder(round: number): readonly ProductKind[] {
  return round % 2 === 0 ? ["vize", "vue-language-tools"] : ["vue-language-tools", "vize"];
}

export function stableOperationTranscript(operations: readonly ClientOperation[]): string {
  return JSON.stringify(operations);
}

export function dualLspToolReceipt(): Record<string, unknown> {
  const require = createRequire(path.join(dualLspProviderRoot, "package.json"));
  const tools = resolveTools();
  const packageVersion = (name: string) => {
    const manifest = JSON.parse(fs.readFileSync(require.resolve(`${name}/package.json`), "utf8")) as {
      version: string;
    };
    return manifest.version;
  };
  return {
    providerRoot: dualLspProviderRoot,
    vize: {
      binary: process.env.VIZE_LSP_BIN,
      args: ["lsp"],
      allocationProfile: process.env.LSP_DUAL_VIZE_PROFILE_DIR ?? null,
    },
    vueLanguageTools: {
      package: "@vue/language-server",
      version: packageVersion("@vue/language-server"),
      command: process.execPath,
      args: [tools.vueLanguageServer, "--stdio", `--tsdk=${tools.tsdk}`],
      cpuProfile: process.env.LSP_DUAL_VUE_CPU_PROFILE_DIR ?? null,
    },
    typescriptLanguageServer: {
      package: "typescript-language-server",
      version: packageVersion("typescript-language-server"),
      command: process.execPath,
      args: [tools.tsLanguageServer, "--stdio", "--log-level", "1"],
    },
    vueTypescriptPlugin: {
      package: "@vue/typescript-plugin",
      version: packageVersion("@vue/typescript-plugin"),
      location: tools.pluginRoot,
    },
    typescript: { version: packageVersion("typescript"), tsdk: tools.tsdk },
    architecture:
      "Vue Language Tools v3 is measured as Vue LS plus TypeScript LS with @vue/typescript-plugin and the tsserver request bridge",
  };
}
