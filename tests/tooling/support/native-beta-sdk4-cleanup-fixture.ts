import assert from "node:assert/strict";
import fs from "node:fs";
import { logicalFixture } from "./native-beta-logical-fixture.ts";
import { sdk4CleanupPackages } from "./native-beta-dto/after4-cleanup-source.ts";
type Row = Record<string, unknown>;
const object = (value: unknown): Row => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
export const observer =
  "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta.rs";
export const helper = observer.replace(/\.rs$/u, "/storage_authority.rs");
export const input =
  "tests/_fixtures/differential/typechecker/logical-workspace-overlay-revision-11145/input.json";
const version = "2.0.0-beta.4";
export const registry = "registry+https://github.com/rust-lang/crates.io-index";
// Constructed boundaries only, never a native capture or qualification witness.
export function sdk4CleanupFixture() {
  const f = logicalFixture(),
    owners = f.owners.map((owner) => ({
      ...owner,
      client: [...owner.client],
      server: [...owner.server],
      apiClient: [...owner.apiClient],
      apiServer: [...owner.apiServer],
    })),
    owner = owners[0]!,
    main = f.attached.mainUri,
    root = f.root;
  const shutdown = owner.client.pop()!;
  object(shutdown).id = 58;
  object(owner.server.pop()).id = 58;
  const symbols = [
    {
      name: "checked",
      kind: 13,
      location: {
        uri: main,
        range: { start: { line: 2, character: 13 }, end: { line: 2, character: 36 } },
      },
    },
    {
      name: "echo",
      kind: 13,
      location: {
        uri: main,
        range: { start: { line: 3, character: 13 }, end: { line: 3, character: 25 } },
      },
    },
  ];
  for (const [id, name] of [
    [56, f.attached.materializedBatch.virtualUri],
    [57, f.attached.supportUri],
  ] as const) {
    owner.client.push({
      jsonrpc: "2.0",
      method: "textDocument/didClose",
      params: { textDocument: { uri: name } },
    });
    owner.client.push({
      jsonrpc: "2.0",
      id,
      method: "textDocument/documentSymbol",
      params: { textDocument: { uri: main } },
    });
    owner.server.push({ jsonrpc: "2.0", id, result: structuredClone(symbols) });
  }
  owner.client.push(shutdown);
  owner.server.push({ jsonrpc: "2.0", id: 58, result: null });
  owner.apiClient = [];
  owner.apiServer = [];
  const config = root + "/tsconfig.json",
    open = { ensurePrograms: true, openProjects: [config] };
  const parsed = {
    fileNames: [root + "/main.ts", root + "/support.ts"],
    options: {
      module: 99,
      moduleResolution: 100,
      noEmit: true,
      strict: true,
      target: 99,
      types: [],
      configFilePath: config,
    },
    compileOnSave: false,
    raw: JSON.parse(f.attached.config) as unknown,
    errors: [],
  };
  const api = (method: string, params: unknown, result: unknown) => {
    const id = owner.apiClient.length + 1;
    owner.apiClient.push({ jsonrpc: "2.0", id, method, params });
    owner.apiServer.push({ jsonrpc: "2.0", id, result });
  };
  const snapshot = (handle: number, projects: boolean) => ({
    snapshot: handle,
    projects: projects
      ? [
          {
            id: config,
            configFileName: config,
            currentDirectory: root,
            dirty: false,
            parsedCommandLine: structuredClone(parsed),
          },
        ]
      : [],
    operation: {},
  });
  api("initialize", null, { caseSensitivity: 1, currentDirectory: root });
  api("parseConfigFile", { file: config }, parsed);
  api("createSnapshot", open, snapshot(100, true));
  api("updateSnapshot", { changes: open, snapshot: 100 }, snapshot(101, false));
  api("describeCapabilities", null, null);
  owner.apiServer[4] = {
    jsonrpc: "2.0",
    id: 5,
    error: {
      code: -32603,
      message: 'api: invalid request: unknown API method "describeCapabilities"',
    },
  };
  api("release", { handle: "100", snapshot: 100 }, true);
  for (let handle = 102; handle <= 111; handle++) {
    api("createSnapshot", open, snapshot(handle, true));
    api("release", { handle: String(handle - 1), snapshot: handle - 1 }, true);
  }
  const source: Row = {
    source: f.source,
    role: "after",
    sdkVersion: "7.1.0-beta",
    sdkSha256: f.context.sdkSha256,
    observerSha256: "c75ef79c388eb5aee6c93115150196c42aecb75d638ead10dab1a1b94872ca61",
    observerDependencies: [
      { path: helper, sha256: "9bc79324540fd1b37f649f102fa2ed90a09aaf53eca1765f799cb6bc1fe88d40" },
    ],
    testExecutable: "/mock/vize_canon",
    attached: f.attached,
  };
  object(source.attached).observerInputDependencies = [
    {
      path: input,
      bytes: 1108,
      sha256: "4659fc88d76ee98c9f0ddc74f6450ecc80b720be9b3ab4a0335bf8f2a2050c58",
    },
  ];
  const files = new Map<string, Uint8Array>([
    [observer, fs.readFileSync(observer)],
    [helper, fs.readFileSync(helper)],
    [input, fs.readFileSync(input)],
    [
      "Cargo.toml",
      Buffer.from(
        "[workspace.dependencies]\n" +
          ["corsa", "corsa_lsp", "corsa_runtime"]
            .map((name) => `${name} = { package = "${name}", version = "=${version}" }`)
            .join("\n") +
          "\n",
      ),
    ],
    [
      "Cargo.lock",
      Buffer.from(
        "version = 4\n" +
          sdk4CleanupPackages
            .map(
              ([name, checksum, dependencies]) =>
                `\n[[package]]\nname = "${name}"\nversion = "${version}"\nsource = "${registry}"\nchecksum = "${checksum}"\ndependencies = ${JSON.stringify(dependencies)}\n`,
            )
            .join(""),
      ),
    ],
  ]);
  const artifacts: Row[] = sdk4CleanupPackages.map(([name]) => ({
    reason: "compiler-artifact",
    package_id: registry + "#" + name + "@" + version,
    manifest_path:
      "/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/" +
      name +
      "-" +
      version +
      "/Cargo.toml",
    target: { name, kind: ["lib"] },
    profile: { test: false },
    executable: null,
  }));
  artifacts.push(
    {
      reason: "compiler-artifact",
      package_id: "path+file:///mock#vize_canon@0.1.0",
      manifest_path: "/mock/Cargo.toml",
      target: { name: "vize_canon", kind: ["lib"] },
      profile: { test: true },
      executable: "/mock/vize_canon",
    },
    { reason: "build-finished", success: true },
  );
  return { ...f, owners, sourceRecord: source, files, artifacts };
}
