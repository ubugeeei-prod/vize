import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { CONFIG_FREE_PROJECT_SHAPES } from "../../../tools/support/compat/npm/smoke-release-init-config-free.mjs";
import {
  FRESH_INIT_MATRIX,
  PROJECT_SHAPES,
} from "../../../tools/support/compat/npm/smoke-release-init-shapes.mjs";
import { PACKAGE_MANAGERS } from "../../../tools/support/compat/npm/smoke-release-init-managers.mjs";
import { readRepoFile } from "../support/github-workflows.ts";
import { expectedInitOutput } from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";
import {
  CONFIG_SOURCES,
  DEFAULT_BUILD,
  PRESERVED_BUILD,
  lintReport,
  tsconfigBase,
  typeDiagnostics,
  vueHelper,
} from "../../../tools/support/compat/npm/smoke-release-config-fixture.ts";
import { runWorkspacePackageCheck } from "../../../tools/support/compat/npm/smoke-release-init-workspace.ts";
import {
  assertLatestPublication,
  expectedConfigReloadError,
  runConfigLspSession,
  type ConfigLspReceipt,
} from "../../../tools/support/compat/npm/smoke-release-config-lsp.ts";

const CHANGED_FIELDS = new Set([
  "detection",
  "features",
  "reconfiguredDetection",
  "reconfiguredFeatures",
  "createdFiles",
  "expectedFiles",
]);
const PEERS = { typescript: "6.0.3", vite: "8.0.0", "vite-plus": "0.1.0", vue: "3.5.0" };

void test("current initializer changes only the declared generated-plan capability", () => {
  assert.deepEqual(Object.keys(CONFIG_FREE_PROJECT_SHAPES), Object.keys(PROJECT_SHAPES));
  for (const [id, historical] of Object.entries(PROJECT_SHAPES)) {
    const current = CONFIG_FREE_PROJECT_SHAPES[id];
    assert.deepEqual(Object.keys(current), Object.keys(historical), id);
    for (const [key, value] of Object.entries(historical)) {
      if (!CHANGED_FIELDS.has(key)) assert.deepEqual(current[key], value, `${id}.${key}`);
    }
    assert.deepEqual(current.files(PEERS), historical.files(PEERS), `${id}: whole authored inputs`);
    assert.deepEqual(current.check, historical.check, `${id}: whole diagnostic triples`);
    assert.ok(historical.createdFiles.includes("vize.config.ts"), `${id}: archive retained`);
    assert.ok(historical.expectedFiles["vize.config.ts"], `${id}: whole archive retained`);
    const { "vize.config.ts": originalConfig, ...retainedFiles } = historical.expectedFiles;
    assert.equal(typeof originalConfig, "string");
    assert.deepEqual(current.expectedFiles, retainedFiles, `${id}: all other complete files`);
    assert.deepEqual(
      current.createdFiles,
      id === "vite-vue-js-checkjs"
        ? ["tsconfig.json", ".vscode/extensions.json"]
        : [".vscode/extensions.json"],
    );
    assert.deepEqual(current.features, [
      "  lint      skipped    not selected",
      `  bundler   configured adds vize() to vite.config.${id === "vite-vue-js-checkjs" ? "js" : "ts"}`,
      "  fmt       configured uses project settings and formatter defaults",
      id === "vite-vue-js-checkjs"
        ? "  typecheck configured writes tsconfig.json"
        : "  typecheck configured uses tsconfig.json and project settings",
      "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
    ]);
    assert.deepEqual(current.reconfiguredFeatures, [
      "  lint      skipped    not selected",
      `  bundler   unchanged  vite.config.${id === "vite-vue-js-checkjs" ? "js" : "ts"} already uses @vizejs/vite-plugin`,
      "  fmt       unchanged  uses project settings and formatter defaults",
      "  typecheck unchanged  uses tsconfig.json and project settings",
      "  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize",
    ]);
  }
});

void test("every original manager cell retains its entire current discovery packet", () => {
  for (const cell of FRESH_INIT_MATRIX) {
    const current = CONFIG_FREE_PROJECT_SHAPES[cell.shape];
    const manager = PACKAGE_MANAGERS[cell.packageManager];
    assert.deepEqual(
      current.detection(manager),
      PROJECT_SHAPES[cell.shape]
        .detection(manager)
        .map((line) =>
          line === "  vize config:     none" ? "  Vue settings:    vue: in vite.config.*" : line,
        ),
    );
    assert.deepEqual(current.reconfiguredDetection(manager), [
      cell.shape === "vite-plus-vue-ts"
        ? "  framework:       Vite+ (vite.config.ts)"
        : `  framework:       Vite (vite.config.${cell.shape === "vite-vue-js-checkjs" ? "js" : "ts"})`,
      `  package manager: ${manager.detectedPackageManager}`,
      "  language:        TypeScript (tsconfig.json)",
      cell.shape === "vite-plus-vue-ts"
        ? "  lint command:    vp lint"
        : "  lint command:    oxlint",
      "  Vue settings:    vue: in vite.config.*",
      "  oxlint config:   none",
    ]);
  }
});

void test("the genuine packed driver selects the named successor and checks both init passes", () => {
  const source = readRepoFile("tools/support/compat/npm/smoke-release-init-fresh.mjs");
  assert.ok(source.includes("CONFIG_FREE_PROJECT_SHAPES as PROJECT_SHAPES"));
  assert.equal(source.match(/assertNoDedicatedConfig\(projectRoot\);/gu)?.length, 2);
  assert.ok(source.includes('name.startsWith("vize.config.")'));
  assert.ok(source.includes('"a second init run changed the project"'));
  assert.ok(source.includes("shape.check.brokenDiagnostics"));
});

void test("the delivered Yarn cell retains its whole archive and declares three current stdout packets", () => {
  const historical = JSON.parse(
    readRepoFile("tests/tooling/fixtures/release-smoke-init-yarn-checkjs.json"),
  );
  const current = JSON.parse(
    readRepoFile("tests/tooling/fixtures/release-smoke-init-yarn-checkjs-config-free.json"),
  );
  const shape = CONFIG_FREE_PROJECT_SHAPES["vite-vue-js-checkjs"];
  const manager = PACKAGE_MANAGERS.yarn;
  for (const [key, value] of Object.entries(historical)) {
    if (key !== "expected") assert.deepEqual(current[key], value, key);
  }
  assert.deepEqual(Object.keys(current), Object.keys(historical));
  assert.deepEqual(Object.keys(current.expected), Object.keys(historical.expected));
  for (const [key, value] of Object.entries(historical.expected)) {
    if (!["files", "dryStdout", "applyStdout", "rerunStdout"].includes(key))
      assert.deepEqual(current.expected[key], value, key);
  }
  assert.deepEqual(FRESH_INIT_MATRIX, [
    ...historical.originalSevenCells,
    historical.proposedEighthCell,
    { packageManager: "bun", shape: "vite-vue-js-checkjs" },
    { packageManager: "vp", shape: "vite-vue-js-checkjs" },
  ]);
  assert.deepEqual(
    { ...shape.files(current.peers), ...manager.projectFiles },
    current.authoredFiles,
  );
  const { "vize.config.ts": archivedConfig, ...retainedFiles } = historical.expected.files;
  assert.equal(typeof archivedConfig, "string");
  assert.deepEqual(current.expected.files, retainedFiles);
  assert.deepEqual(shape.expectedFiles, current.expected.files);
  for (const mode of ["dry", "apply", "rerun"])
    assert.equal(
      expectedInitOutput(shape, current.projectRootToken, manager, mode),
      current.expected[`${mode}Stdout`].replaceAll(
        "  vize config:     none",
        "  Vue settings:    vue: in vite.config.*",
      ),
    );
  assert.deepEqual(shape.check.broken, current.expected.broken.files);
  assert.deepEqual(shape.check.brokenDiagnostics, current.expected.broken.diagnosticFiles);
  assert.deepEqual(current.expected.clean, historical.expected.clean);
  assert.deepEqual(current.expected.broken, historical.expected.broken);
  assert.deepEqual(current.expected.repair, historical.expected.repair);
});

void test("the packed settings cell retains whole original sources and compiler/lint oracles", () => {
  const source = (file: string) =>
    readRepoFile(`tests/_fixtures/differential/config/project-settings-8371/${file}.vue.txt`);
  assert.equal(CONFIG_SOURCES["BuildPanel.vue"], source("App"));
  assert.equal(CONFIG_SOURCES["ImagePanel.vue"], source("Image"));
  const snapshot = (name: string) =>
    readRepoFile(
      `crates/vize/tests/snapshots/project_config_cli__project_settings_${name}.snap`,
    ).replace(/^---\n[\s\S]*?\n---\n/u, "");
  assert.equal(PRESERVED_BUILD, snapshot("build_vite"));
  assert.equal(DEFAULT_BUILD, snapshot("build_defaults"));
  const original = JSON.parse(snapshot("lint_vite"));
  assert.deepEqual(lintReport("ImagePanel.vue", true), [
    {
      ...original[0],
      file: "ImagePanel.vue",
      messages: [original[0].messages[1]],
      errorCount: 1,
    },
  ]);
  assert.deepEqual(lintReport("ImagePanel.vue", false), [
    {
      file: "ImagePanel.vue",
      messages: [],
      errorCount: 0,
      warningCount: 0,
    },
  ]);
});

await test("the contrasting helper and inherited TS controls are independent", async () => {
  const load = async (configured: boolean) =>
    (await import(`data:text/javascript,${encodeURIComponent(vueHelper(configured))}`)).default;
  const app = await load(true);
  const control = await load(false);
  assert.equal(app.compiler.whitespace, "preserve");
  assert.equal(control.compiler.whitespace, "condense");
  assert.deepEqual(app.compiler.customElements, ["my-widget"]);
  assert.deepEqual(control.compiler.customElements, []);
  assert.equal(app.formatter.singleQuote, true);
  assert.equal(control.formatter.singleQuote, false);
  assert.equal(app.linter.rules["a11y/alt-text"], "error");
  assert.equal(control.linter.rules["a11y/alt-text"], "off");
  assert.deepEqual(app.languageServer, { typecheck: true, lint: false, formatting: true });
  assert.deepEqual(control.languageServer, app.languageServer);
  assert.deepEqual(JSON.parse(tsconfigBase(true)), { compilerOptions: { strictNullChecks: true } });
  assert.deepEqual(JSON.parse(tsconfigBase(false)), {
    compilerOptions: { strictNullChecks: false },
  });
  const diagnostic = typeDiagnostics(true)[0]!;
  const line = CONFIG_SOURCES["TypedPanel.vue"].split("\n")[diagnostic.range.start.line]!;
  assert.equal(
    line.slice(diagnostic.range.start.character, diagnostic.range.end.character),
    "value",
  );
  assert.deepEqual(typeDiagnostics(false), []);
});

void test("an invoked required packed workspace cannot silently skip missing public artifacts", () => {
  assert.throws(
    () =>
      runWorkspacePackageCheck({
        freshRoot: "/unused",
        packed: new Map(),
        peers: {},
        tempDir: "/unused",
        versions: new Map(),
        vizeBin: "/unused/vize",
      }),
    /requires both packed public packages/u,
  );
});

void test("an older matching native packet cannot hide a later abnormal publication", () => {
  const expected = {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: "file:///workspace/packages/app/TypedPanel.vue",
      version: 1,
      diagnostics: typeDiagnostics(true),
    },
  };
  const otherOwner = {
    ...expected,
    params: {
      uri: "file:///workspace/packages/control/TypedPanel.vue",
      version: 1,
      diagnostics: [],
    },
  };
  assert.doesNotThrow(() => assertLatestPublication([expected, otherOwner], expected));
  for (const abnormal of [
    { ...expected, params: { ...expected.params, diagnostics: [] } },
    { ...expected, params: { ...expected.params, version: 2 } },
    { ...expected, unexpectedField: true },
  ]) {
    assert.throws(
      () => assertLatestPublication([expected, otherOwner, abnormal], expected),
      /latest complete publication/u,
    );
  }
  assert.throws(
    () => assertLatestPublication([otherOwner], expected),
    /latest complete publication/u,
  );
});

void test("configuration errors retain the producer's complete stable URI envelope", () => {
  assert.deepEqual(
    expectedConfigReloadError("file:///workspace/app%20one", [
      "file:///workspace/app%20one/z-base.json",
      "file:///workspace/app%20one/./vue-settings.ts",
      "file:///workspace/app%20one/z-base.json",
    ]),
    {
      jsonrpc: "2.0",
      method: "window/showMessage",
      params: {
        type: 1,
        message:
          "Vize could not reload project configuration for file:///workspace/app%20one after changes to file:///workspace/app%20one/./vue-settings.ts, file:///workspace/app%20one/z-base.json. Keeping the last good configuration.",
      },
    },
  );
});

await test("config LSP custody retains late bytes through a successful close", async () => {
  const raw = Buffer.from([0, 255, 13, 10]);
  const child = spawn(process.execPath, [
    "-e",
    `
    process.stdin.resume();
    process.stdin.on('end', () => setTimeout(() => {
      process.stdout.write(Buffer.from([0, 255, 13, 10]));
      process.stderr.write('legitimate tracing at close\\n');
    }, 20));
  `,
  ]);
  let receipt: ConfigLspReceipt | undefined;
  await runConfigLspSession(child, {
    run: async () => {},
    shutdown: async () => {},
    exit: () => child.stdin.end(),
    protocolState: () => ({ remainder: Buffer.alloc(0) }),
    save: (value) => {
      receipt = value;
    },
  });
  assert.ok(receipt);
  assert.equal(receipt.closed, true);
  assert.equal(receipt.status, 0);
  assert.equal(receipt.signal, null);
  assert.deepEqual(Buffer.from(receipt.stdoutBase64, "base64"), raw);
  assert.equal(receipt.stderr, "legitimate tracing at close\n");
  assert.deepEqual(Buffer.from(receipt.stderrBase64, "base64"), Buffer.from(receipt.stderr));
  assert.deepEqual(receipt.failures, []);
});

for (const scenario of [
  "status",
  "signal",
  "truncated frame",
  "protocol error",
  "first error",
  "close timeout",
] as const) {
  await test(`config LSP custody fails and saves the whole receipt: ${scenario}`, async () => {
    const signal = scenario === "signal";
    const timeout = scenario === "close timeout";
    const child = spawn(process.execPath, [
      "-e",
      `
      process.stdin.resume();
      process.stdin.on('end', () => setTimeout(() => {
        process.stdout.write('Content-Length: 9\\r\\n\\r\\n{');
        process.stderr.write('late stderr before termination');
        ${signal ? "process.kill(process.pid, 'SIGTERM');" : `process.exitCode = ${scenario === "status" || scenario === "first error" ? 7 : 0};`}
        ${timeout ? "setInterval(() => {}, 1000);" : ""}
      }, 20));
    `,
    ]);
    const original = new Error("original full-packet assertion");
    const remainder =
      scenario === "truncated frame" ? Buffer.from("Content-Length: 9\r\n\r\n{") : Buffer.alloc(0);
    let receipt: ConfigLspReceipt | undefined;
    await assert.rejects(
      runConfigLspSession(child, {
        run: async () => {
          if (scenario === "first error") throw original;
        },
        shutdown: async () => {
          if (scenario === "first error") throw new Error("shutdown also failed");
        },
        exit: () => child.stdin.end(),
        closeTimeoutMs: timeout ? 2_000 : undefined,
        protocolState: () => ({
          remainder,
          ...(scenario === "protocol error"
            ? { error: new Error("invalid LSP Content-Length") }
            : {}),
        }),
        save: (value) => {
          receipt = value;
        },
      }),
      (error: unknown) => {
        if (scenario === "first error") return error === original;
        assert.match(
          String(error),
          scenario === "truncated frame"
            ? /incomplete frame/u
            : scenario === "protocol error"
              ? /invalid LSP Content-Length/u
              : timeout
                ? /did not close after shutdown\/exit/u
                : /config LSP child exited/u,
        );
        return true;
      },
    );
    assert.ok(receipt);
    assert.equal(receipt.closed, true);
    assert.equal(receipt.stderr, "late stderr before termination");
    assert.deepEqual(
      Buffer.from(receipt.stdoutBase64, "base64"),
      Buffer.from("Content-Length: 9\r\n\r\n{"),
    );
    assert.deepEqual(Buffer.from(receipt.incompleteFrameBase64, "base64"), remainder);
    assert.equal(receipt.signal, signal ? "SIGTERM" : timeout ? "SIGKILL" : null);
    assert.equal(
      receipt.status,
      signal || timeout ? null : scenario === "status" || scenario === "first error" ? 7 : 0,
    );
    if (scenario === "first error") {
      assert.equal(receipt.failures.length, 3);
      assert.match(receipt.failures[0]!, /original full-packet assertion/u);
      assert.match(receipt.failures[1]!, /shutdown also failed/u);
      assert.match(receipt.failures[2]!, /status 7/u);
    }
  });
}

void test("the existing general LSP observer runs without config-mode dependencies", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-general-lsp-"));
  try {
    const session = path.join(root, "session.ts");
    const logs = path.join(root, "logs");
    fs.writeFileSync(
      session,
      readRepoFile("tools/support/compat/npm/smoke-release-lsp-session.ts"),
    );
    fs.mkdirSync(path.join(root, "src/components"), { recursive: true });
    fs.mkdirSync(path.join(root, "node_modules/vize/bin"), { recursive: true });
    fs.writeFileSync(
      path.join(root, "src/App.vue"),
      `
<script setup>
import HelloWorld from './components/HelloWorld.vue';
const title = 'ready';
</script>
<template><HelloWorld :msg="title" /></template>
`,
    );
    // This transport fixture answers the four historical feature contracts.
    // There is deliberately no config module, package manifest, or semver.
    fs.writeFileSync(
      path.join(root, "node_modules/vize/bin/vize"),
      `
const {pathToFileURL} = require('node:url');
const path = require('node:path');
let buffer = Buffer.alloc(0);
process.stdin.on('data', (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (;;) {
    const header = buffer.indexOf('\\r\\n\\r\\n');
    if (header < 0) return;
    const length = Number(/Content-Length: (\\d+)/i.exec(buffer.subarray(0,header).toString())[1]);
    if (buffer.length < header + 4 + length) return;
    const message = JSON.parse(buffer.subarray(header+4,header+4+length).toString());
    buffer = buffer.subarray(header+4+length);
    if (message.method === 'exit') process.exit(0);
    if (message.id === undefined) continue;
    const app = pathToFileURL(path.join(process.cwd(),'src/App.vue')).href;
    const component = pathToFileURL(path.join(process.cwd(),'src/components/HelloWorld.vue')).href;
    const result = message.method === 'initialize' ? {capabilities:{}}
      : message.method === 'textDocument/hover' ? {contents:'title: string'}
      : message.method === 'textDocument/definition' ? {uri:component}
      : message.method === 'textDocument/completion' ? [{label:'title'}]
      : message.method === 'textDocument/rename' ? {changes:{[app]:[]}}
      : null;
    const body = JSON.stringify({jsonrpc:'2.0',id:message.id,result});
    process.stdout.write('Content-Length: '+Buffer.byteLength(body)+'\\r\\n\\r\\n'+body);
  }
});
`,
    );
    const result = spawnSync(process.execPath, [session, root], {
      cwd: root,
      env: { ...process.env, VIZE_SMOKE_LSP_LOG_DIR: logs },
      encoding: "utf8",
      timeout: 15_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /hover, definition, completion, rename/u);
    const receiptFile = fs.readdirSync(logs).find((file) => file.endsWith(".results.json"));
    assert.ok(receiptFile);
    const receipt = JSON.parse(fs.readFileSync(path.join(logs, receiptFile), "utf8"));
    assert.equal(receipt.results.length, 4);
    assert.equal(receipt.custody, undefined, "general observer retains its existing lifecycle");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
