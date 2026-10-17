// Real adapter serializers, followed by explicitly labeled native protocol replay.
// GUI behavior remains the responsibility of the existing host scenarios.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { sha256, verifyArtifact } from "./server-artifact.ts";
import { type Json, LspClient } from "./support/lsp-client.ts";

const repository = path.resolve(import.meta.dirname, "../..");
const [editor, outputArg] = process.argv.slice(2);
assert.ok(editor === "neovim" || editor === "vim" || editor === "emacs");
assert.ok(outputArg, "Usage: named-profiles.ts neovim|vim|emacs OUTPUT");
const output = path.resolve(outputArg);
assert.ok(!fs.existsSync(output), "Profile evidence must use a fresh output directory");
fs.mkdirSync(output, { recursive: true });
const sourceSha = execFileSync("git", ["rev-parse", "HEAD"], {
  cwd: repository,
  encoding: "utf8",
}).trim();
assert.equal(sourceSha, process.env.GITHUB_SHA, "Use the exact checked-out Actions source");
const server = process.env.VIZE_SERVER_PATH;
assert.ok(server && path.isAbsolute(server), "Use only the verified source-built server");
const artifact = await verifyArtifact(
  server,
  path.join(path.dirname(server), "server.json"),
  sourceSha,
);

const adapter = editor === "neovim" ? "nvim" : editor;
const inputs = [
  "tests/editor-conformance/named-profiles.ts",
  "tests/editor-conformance/server-artifact.ts",
  "tests/editor-conformance/support/lsp-client.ts",
  "tests/editor-conformance/lsp-tap.mjs",
  ".github/workflows/editor-conformance.yml",
  "crates/vize_maestro/src/server/state/features.rs",
  ...(editor === "neovim"
    ? [
        "editors/nvim/lua/vize/config.lua",
        "editors/nvim/lua/vize/init.lua",
        "editors/nvim/ftdetect/vize.lua",
        "editors/nvim/test/vize_spec.lua",
      ]
    : editor === "vim"
      ? [
          "editors/vim/autoload/vize.vim",
          "editors/vim/plugin/vize.vim",
          "editors/vim/ftdetect/vize.vim",
          "editors/vim/test/vize_spec.vim",
        ]
      : ["editors/emacs/vize.el", "editors/emacs/test/vize-test.el"]),
  `editors/${adapter}/README.md`,
];
const hashes = await Promise.all(
  inputs.map(async (file) => {
    const bytes = fs.readFileSync(path.join(repository, file));
    const committed = execFileSync("git", ["show", `${sourceSha}:${file}`], { cwd: repository });
    assert.deepEqual(bytes, committed, `Consumed source must match ${sourceSha}: ${file}`);
    return `${await sha256(path.join(repository, file))}  ${file}\n`;
  }),
);
fs.writeFileSync(path.join(output, "inputs.sha256"), hashes.join(""));

const units = {
  neovim: [
    process.env.VIZE_TEST_NVIM_PATH ?? "nvim",
    "--headless",
    "-u",
    "NONE",
    "--noplugin",
    "-l",
    "editors/nvim/test/vize_spec.lua",
  ],
  vim: [
    process.env.VIZE_TEST_VIM_PATH ?? "vim",
    "-Nu",
    "NONE",
    "-n",
    "-es",
    "-S",
    "editors/vim/test/vize_spec.vim",
  ],
  emacs: [
    process.env.VIZE_TEST_EMACS_PATH ?? "emacs",
    "-Q",
    "--batch",
    "-l",
    "ert",
    "-l",
    "editors/emacs/test/vize-test.el",
    "-f",
    "ert-run-tests-batch-and-exit",
  ],
};
const packet = path.join(output, "profiles.json");
const [command, ...args] = units[editor];
const hostVersion = execFileSync(command, ["--version"], { encoding: "utf8", timeout: 10_000 });
fs.writeFileSync(path.join(output, "host.version.log"), hostVersion);
const host = spawnSync(command, args, {
  cwd: repository,
  encoding: "utf8",
  timeout: 60_000,
  env: { ...process.env, VIZE_PROFILE_PACKET: packet },
});
fs.writeFileSync(path.join(output, "host.stdout.log"), host.stdout ?? "");
fs.writeFileSync(path.join(output, "host.stderr.log"), host.stderr ?? "");
assert.ifError(host.error);
assert.equal(host.status, 0, "The actual editor's complete unit suite must pass");
assert.ok(fs.existsSync(packet), "An editor error must not silently omit its serialized options");
const profiles: Record<string, Json> = JSON.parse(fs.readFileSync(packet, "utf8"));
const names = ["defaults", "partial", "recommended", "lint", "off"];
assert.deepEqual(
  Object.keys(profiles).sort((left, right) => left.localeCompare(right)),
  [...names].sort((left, right) => left.localeCompare(right)),
);

// Check the actual public native input inventory, not just a hand-picked
// subset: a newly introduced switch must be explicitly excluded by lint/off.
const schema = fs
  .readFileSync(path.join(repository, "crates/vize_maestro/src/server/state/features.rs"), "utf8")
  .split("impl LspConfigSection")[0];
const featureKeys = [...schema.matchAll(/^\s*(?:pub\(super\)\s+)?(\w+): Option<bool>,/gm)]
  .map((match) => match[1].replace(/_([a-z0-9])/g, (_word, letter: string) => letter.toUpperCase()))
  .sort((left, right) => left.localeCompare(right));
assert.ok(featureKeys.length >= 29, "The complete current native feature schema must be present");
for (const name of ["off", "lint"]) {
  const options = profiles[name];
  assert.ok(options && typeof options === "object" && !Array.isArray(options));
  assert.deepEqual(
    Object.keys(options).sort((left, right) => left.localeCompare(right)),
    featureKeys,
  );
  for (const key of featureKeys)
    assert.equal(
      options[key],
      name === "lint" && (key === "enabled" || key === "lint"),
      `${name}: ${key}`,
    );
}
assert.deepEqual(profiles.partial, { hover: false });
assert.ok(
  profiles.defaults === null ||
    (typeof profiles.defaults === "object" && Object.keys(profiles.defaults).length === 0),
);

// Every stable project switch is enabled. The adapter must override these
// rather than accidentally inheriting them, especially formatting/crossFile.
const projectFeatures = Object.fromEntries(
  [
    "enabled",
    "diagnostics",
    "lint",
    "typecheck",
    "editor",
    "ecosystem",
    "completion",
    "signatureHelp",
    "hover",
    "definition",
    "references",
    "documentSymbols",
    "workspaceSymbols",
    "codeActions",
    "rename",
    "formatting",
    "codeLens",
    "semanticTokens",
    "documentLinks",
    "foldingRanges",
    "inlayHints",
    "fileRename",
    "corsa",
    "tsgo",
    "crossFile",
  ].map((key) => [key, true]),
);
const workspace = path.join(output, "workspace");
fs.mkdirSync(workspace);
fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true}\n');
fs.writeFileSync(path.join(workspace, "tsconfig.json"), '{"vueCompilerOptions":{"target":3}}\n');
fs.writeFileSync(
  path.join(workspace, "vite.config.mjs"),
  `export default ${JSON.stringify({ vue: { languageServer: projectFeatures } })};\n`,
);
const rootUri = pathToFileURL(workspace).href;
const disabledCapabilities = [
  "hoverProvider",
  "completionProvider",
  "definitionProvider",
  "referencesProvider",
  "documentHighlightProvider",
  "documentSymbolProvider",
  "workspaceSymbolProvider",
  "codeActionProvider",
  "renameProvider",
  "linkedEditingRangeProvider",
  "documentFormattingProvider",
  "documentRangeFormattingProvider",
  "documentOnTypeFormattingProvider",
  "signatureHelpProvider",
  "codeLensProvider",
  "semanticTokensProvider",
  "documentLinkProvider",
  "foldingRangeProvider",
  "selectionRangeProvider",
  "inlayHintProvider",
  "colorProvider",
  "typeDefinitionProvider",
  "implementationProvider",
  "declarationProvider",
  "callHierarchyProvider",
  "monikerProvider",
  "experimental",
];
const positives = [
  "completionProvider",
  "definitionProvider",
  "referencesProvider",
  "documentSymbolProvider",
  "workspaceSymbolProvider",
  "codeActionProvider",
  "renameProvider",
  "documentFormattingProvider",
  "codeLensProvider",
  "semanticTokensProvider",
  "documentLinkProvider",
  "foldingRangeProvider",
  "inlayHintProvider",
];
const results: Record<string, Json> = {};
for (const name of names) {
  const transcript = path.join(output, `${name}.jsonl`);
  // GNU timeout also bounds the native process and its tap when the transport
  // refuses initialization; no unresolved request can strand this Actions job.
  const client = new LspClient(
    "timeout",
    [
      "--signal=TERM",
      "--kill-after=5s",
      "30s",
      process.execPath,
      path.join(repository, "tests/editor-conformance/lsp-tap.mjs"),
      server,
      "lsp",
    ],
    { cwd: workspace, env: { ...process.env, VIZE_CONFORMANCE_TRANSCRIPT: transcript } },
    () => null,
  );
  const request = async (method: string, params: Json | undefined) => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      return await Promise.race([
        client.request(method, params),
        new Promise<never>((_resolve, reject) => {
          timer = setTimeout(() => reject(new Error(`${name}: ${method} timed out`)), 32_000);
        }),
      ]);
    } finally {
      clearTimeout(timer);
    }
  };
  try {
    const result = await request("initialize", {
      processId: process.pid,
      rootUri,
      capabilities: {},
      workspaceFolders: [{ uri: rootUri, name: "all-on-project" }],
      ...(profiles[name] === null ? {} : { initializationOptions: profiles[name] }),
      clientInfo: { name: `${editor} serialized profile protocol replay` },
    });
    assert.ok(result && typeof result === "object" && !Array.isArray(result));
    const capabilities = result.capabilities;
    assert.ok(capabilities && typeof capabilities === "object" && !Array.isArray(capabilities));
    if (name === "off" || name === "lint") {
      for (const key of disabledCapabilities)
        assert.ok(capabilities[key] == null || capabilities[key] === false, `${name}: ${key}`);
      const workspaceCapability = capabilities.workspace;
      assert.ok(
        workspaceCapability &&
          typeof workspaceCapability === "object" &&
          !Array.isArray(workspaceCapability),
      );
      assert.ok(workspaceCapability.fileOperations == null);
    } else {
      for (const key of positives) assert.ok(capabilities[key], `${name} inherits project ${key}`);
      assert.equal(capabilities.hoverProvider, name === "partial" ? undefined : true);
    }
    results[name] = result;
    await request("shutdown", undefined);
    client.notify("exit", undefined);
  } finally {
    assert.equal(await client.exit(), 0, `${name}: the native server must terminate successfully`);
  }
  const records: Array<{
    event?: string;
    code?: number;
    dir?: string;
    msg?: { method?: string; params?: { initializationOptions?: Json } };
  }> = fs
    .readFileSync(transcript, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const initialize = records.filter(
    (record) => record.dir === "c2s" && record.msg?.method === "initialize",
  );
  assert.equal(initialize.length, 1);
  assert.deepEqual(
    initialize[0].msg?.params?.initializationOptions,
    profiles[name] === null ? undefined : profiles[name],
  );
  assert.equal(records.filter((record) => record.event === "exit" && record.code === 0).length, 1);
}

fs.writeFileSync(
  path.join(output, "receipt.json"),
  JSON.stringify(
    {
      passed: true,
      sourceSha,
      mode: "actual editor serializers + native protocol replay",
      editor,
      binary: { path: server, sha256: artifact.sha256 },
      serializer: { command, version: hostVersion, exitCode: host.status },
      cases: names,
      inputs: { path: "inputs.sha256", sha256: await sha256(path.join(output, "inputs.sha256")) },
      packet: { path: "profiles.json", sha256: await sha256(packet) },
      workspaceInputs: await Promise.all(
        ["package.json", "tsconfig.json", "vite.config.mjs"].map(async (file) => ({
          path: file,
          sha256: await sha256(path.join(workspace, file)),
        })),
      ),
      transcripts: await Promise.all(
        names.map(async (name) => ({
          path: `${name}.jsonl`,
          sha256: await sha256(path.join(output, `${name}.jsonl`)),
        })),
      ),
      results,
    },
    null,
    2,
  ) + "\n",
);
console.log(`${editor}: all five serialized profiles qualified against source ${sourceSha}`);
