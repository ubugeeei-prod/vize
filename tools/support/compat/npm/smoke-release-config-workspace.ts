/** Installed CLI settings checks in the existing pnpm workspace; no second install. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import type { Session } from "./smoke-release-config-lsp.ts";
import { runConfigHarness } from "./smoke-release-config-evidence.ts";
import {
  assertLatestPublication,
  expectedConfigReloadError,
  type Message,
  REFERENCE_DIAGNOSTICS,
  REFERENCE_DIRTY_SOURCE,
  REFERENCE_FORMATTED,
  REFERENCE_POLICY,
  REFERENCE_SOURCE,
  referenceWorkspaceFiles,
  CANONICAL_VITE,
  CONFIG_PACKAGE_NAMES,
  CONFIG_SOURCES,
  DOUBLE_FORMATTED,
  SINGLE_FORMATTED,
  assertBuildFileOutput,
  lintReport,
  tsconfigBase,
  vueHelper,
} from "./smoke-release-config-fixture.ts";
import {
  checkReport,
  projectEnv,
  projectLocalVizeBin,
  readJson,
  reportedDiagnostics,
  snapshotFiles,
  writeFiles,
  // @ts-expect-error -- untyped existing release smoke helpers
} from "./smoke-release-init-project.mjs";
// @ts-expect-error -- untyped existing release smoke helpers
import { renderOutput, runResult } from "./smoke-process.mjs";

const LSP_SESSION = fileURLToPath(new URL("./smoke-release-lsp-session.ts", import.meta.url));

/** Same receipt carrier as the existing LSP smoke: complete records in CI stdout/logs. */
export function configSourceSnapshot(project: string) {
  return snapshotFiles(project, [
    "package.json",
    "vite.config.ts",
    "vue-settings.ts",
    "tsconfig.json",
    "tsconfig.base.json",
    "vue-policy.mjs",
    "tsconfig.legacy.json",
    "tsconfig.modern.json",
    "src/legacy/App.vue",
    "src/modern/App.vue",
    ...Object.keys(CONFIG_SOURCES),
    "src/App.vue",
    "src/main.ts",
    "src/components/HelloWorld.vue",
    "config-dist/BuildPanel.js",
  ]);
}

const INSTALLED_IDENTITY = `
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
const [project, workspace] = process.argv.slice(1);
const root = fs.realpathSync(workspace);
const vizeManifest = fs.realpathSync(path.join(project, "node_modules/vize/package.json"));
const require = createRequire(vizeManifest);
const projectRequire = createRequire(path.join(project, "package.json"));
const describe = (file) => {
  const realPath = fs.realpathSync(file);
  const relative = path.relative(root, realPath);
  assert.ok(!path.isAbsolute(relative) && !relative.startsWith(".."), realPath);
  const bytes = fs.readFileSync(realPath);
  return { path: realPath, sha256: createHash("sha256").update(bytes).digest("hex") };
};
const manifest = (file) => ({ ...describe(file), contents: fs.readFileSync(file, "utf8") });
const nativeManifest = require.resolve("@vizejs/native/package.json");
const nativeEntry = require.resolve("@vizejs/native");
const native = require("@vizejs/native");
assert.equal(typeof native.compileSfc, "function");
const binaries = Object.keys(require.cache).filter((file) => file.endsWith(".node")).map(describe);
assert.ok(binaries.length > 0, "installed native provider did not load a native binary");
assert.equal(JSON.parse(fs.readFileSync(nativeManifest)).version, JSON.parse(fs.readFileSync(vizeManifest)).version);
const corsa = "@typescript/typescript-" + process.platform + "-" + process.arch;
process.stdout.write(JSON.stringify({
  project, workspace: root, platform: process.platform, arch: process.arch, node: process.versions.node,
  vize: { manifest: manifest(vizeManifest), bin: describe(path.join(project, "node_modules/vize/bin/vize")) },
  native: { manifest: manifest(nativeManifest), entry: describe(nativeEntry), binaries },
  vite: { manifest: manifest(projectRequire.resolve("vite/package.json")), entry: describe(projectRequire.resolve("vite")) },
  corsa: { name: corsa, manifest: manifest(require.resolve(corsa + "/package.json")) }
}) + "\\n");
`;

function installedIdentity(project: string, workspace: string): void {
  const args = ["--input-type=module", "-e", INSTALLED_IDENTITY, project, workspace];
  const result = runResult(process.execPath, args, { cwd: project, env: projectEnv() });
  // Retain failed process output too; never reduce an install failure to diagnostics.
  console.log(
    JSON.stringify({
      executable: process.execPath,
      args,
      ...result,
      sources: configSourceSnapshot(project),
    }),
  );
  assert.equal(result.signal, null, renderOutput(result));
  assert.equal(result.status, 0, renderOutput(result));
}

export function prepareConfigWorkspace(workspace: string): void {
  for (const name of CONFIG_PACKAGE_NAMES) {
    const project = path.join(workspace, "packages", name);
    const tsconfig = readJson(path.join(project, "tsconfig.json"));
    tsconfig.extends = "./tsconfig.base.json";
    tsconfig.include.push("TypedPanel.vue");
    writeFiles(project, {
      ...CONFIG_SOURCES,
      "vite.config.ts": CANONICAL_VITE,
      "vue-settings.ts": vueHelper(name === "app"),
      "tsconfig.base.json": tsconfigBase(name === "app"),
      "tsconfig.json": `${JSON.stringify(tsconfig, null, 2)}\n`,
    });
    assert.equal(
      fs.readdirSync(project).some((file) => file.startsWith("vize.config.")),
      false,
    );
  }
}

function cli(project: string, args: string[], expectedStatus: number) {
  const argv = [projectLocalVizeBin(project), ...args];
  const result = runResult(process.execPath, argv, {
    cwd: project,
    env: projectEnv(),
  });
  console.log(
    JSON.stringify({
      project,
      executable: process.execPath,
      args: argv,
      ...result,
      sources: configSourceSnapshot(project),
    }),
  );
  assert.equal(result.signal, null, renderOutput(result));
  assert.equal(result.status, expectedStatus, renderOutput(result));
  return result;
}

export function assertConfigCli(project: string, configured: boolean, strict: boolean): void {
  cli(project, ["build", "BuildPanel.vue", "-o", "config-dist"], 0);
  assertBuildFileOutput(
    fs.readFileSync(path.join(project, "config-dist/BuildPanel.js"), "utf8"),
    configured,
  );
  const lint = cli(project, ["lint", "ImagePanel.vue", "--format", "json"], configured ? 1 : 0);
  assert.deepEqual(
    JSON.parse(lint.stdout),
    lintReport("ImagePanel.vue", configured),
    "complete lint file/message packet",
  );
  fs.writeFileSync(path.join(project, "QuotesPanel.vue"), CONFIG_SOURCES["QuotesPanel.vue"]);
  cli(project, ["fmt", "--check", "QuotesPanel.vue"], 1);
  cli(project, ["fmt", "--write", "QuotesPanel.vue"], 0);
  assert.equal(
    fs.readFileSync(path.join(project, "QuotesPanel.vue"), "utf8"),
    configured ? SINGLE_FORMATTED : DOUBLE_FORMATTED,
    "entire configured formatter output",
  );
  cli(project, ["fmt", "--check", "QuotesPanel.vue"], 0);
  let checked;
  try {
    checked = checkReport(project);
  } catch (error) {
    console.log(
      JSON.stringify({
        project,
        stage: "configured check failure",
        error: String(error),
        outcome: (error as { outcome?: unknown }).outcome,
        sources: configSourceSnapshot(project),
      }),
    );
    throw error;
  }
  console.log(
    JSON.stringify({
      project,
      executable: process.execPath,
      args: [projectLocalVizeBin(project), "check", "--format", "json", "--quiet"],
      stage: "configured check",
      ...checked,
      sources: configSourceSnapshot(project),
    }),
  );
  assert.equal(checked.status, strict ? 1 : 0, checked.rendered);
  assert.equal(checked.report.errorCount, strict ? 1 : 0);
  assert.equal(checked.report.warningCount, 0);
  assert.deepEqual(
    reportedDiagnostics(checked.report, project),
    strict
      ? [
          {
            file: "TypedPanel.vue",
            diagnostics: ["error:2:7 [TS2322] Type 'null' is not assignable to type 'string'."],
          },
        ]
      : [],
  );
  // Restore the original unformatted buffer before native LSP compares full edits.
  fs.writeFileSync(path.join(project, "QuotesPanel.vue"), CONFIG_SOURCES["QuotesPanel.vue"]);
}

function assertControlUnchanged(control: string, before: Record<string, string>): void {
  for (const [file, original] of Object.entries(before)) {
    assert.equal(fs.readFileSync(path.join(control, file), "utf8"), original, file);
  }
  assertConfigCli(control, false, false);
}

export function assertConfigDependencyFailure(project: string, name: string): void {
  const result = cli(
    project,
    [name === "vue-settings.ts" ? "build" : "check", "TypedPanel.vue"],
    1,
  );
  assert.ok(
    result.stderr.includes(name.replace(/\.ts$/u, "")) &&
      /[Ee]rror|[Ff]ailed|[Cc]annot/u.test(result.stderr),
    "missing configuration dependency must be named rather than silently ignored",
  );
}

export function assertConfigDeletion(project: string, name: string): void {
  const file = path.join(project, name);
  const original = fs.readFileSync(file, "utf8");
  try {
    fs.unlinkSync(file);
    assertConfigDependencyFailure(project, name);
  } finally {
    fs.writeFileSync(file, original);
  }
}

export function runConfigWorkspaceChecks(workspace: string): void {
  prepareConfigWorkspace(workspace);
  const project = path.join(workspace, "packages/app");
  const control = path.join(workspace, "packages/control");
  installedIdentity(project, workspace);
  installedIdentity(control, workspace);
  const controlFiles = Object.fromEntries(
    [
      ...Object.keys(CONFIG_SOURCES),
      "vite.config.ts",
      "vue-settings.ts",
      "tsconfig.json",
      "tsconfig.base.json",
    ].map((file) => [file, fs.readFileSync(path.join(control, file), "utf8")]),
  );
  assertConfigCli(project, true, true);
  assertControlUnchanged(control, controlFiles);
  fs.writeFileSync(path.join(project, "vue-settings.ts"), vueHelper(false));
  assertConfigCli(project, false, true);
  assertControlUnchanged(control, controlFiles);
  fs.writeFileSync(path.join(project, "vue-settings.ts"), vueHelper(true));
  fs.writeFileSync(path.join(project, "tsconfig.base.json"), tsconfigBase(false));
  assertConfigCli(project, true, false);
  assertControlUnchanged(control, controlFiles);
  fs.writeFileSync(path.join(project, "tsconfig.base.json"), tsconfigBase(true));
  for (const name of ["vue-settings.ts", "tsconfig.base.json"]) {
    assertConfigDeletion(project, name);
    assertConfigCli(project, true, true);
    assertControlUnchanged(control, controlFiles);
  }
  runConfigHarness(process.execPath, [LSP_SESSION, project, "--config-workspace", workspace], {
    cwd: project,
    env: projectEnv(),
    directory: process.env.VIZE_SMOKE_LSP_LOG_DIR ?? path.join(workspace, ".vize-config-harness"),
  });
  // Same-directory reference-owned dialect identity is not covered by these
  // two different package roots, nor by this installed native LSP cell alone.
  console.log("runtime: packed two-package canonical settings and native LSP reload");
}

const uri = (file: string) => pathToFileURL(file).href;

/** Same installed packages/process, now distinct referenced owners beside one Vite file. */
export async function runConfigReferenceWorkspaceLsp(session: Session): Promise<void> {
  const { workspace, notifications, request, send, record } = session;
  const root = path.join(workspace, "packages/app");
  const control = path.join(workspace, "packages/control");
  const controlSources = configSourceSnapshot(control);
  const file = (name: string) => path.join(root, name);
  const document = (name: "legacy" | "modern") => uri(file(`src/${name}/App.vue`));
  const packet = (uri: string, version: number, diagnostics: unknown[]) => ({
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri, version, diagnostics },
  });
  const oldAppPackets = ["src/App.vue", "TypedPanel.vue", "QuotesPanel.vue"].map((name) =>
    packet(uri(file(name)), 1, []),
  );
  const controlPacket = packet(uri(path.join(control, "TypedPanel.vue")), 1, []);
  let modernVersion = 1;
  const legacyPacket = packet(document("legacy"), 1, []);
  const modernPacket = () => packet(document("modern"), modernVersion, REFERENCE_DIAGNOSTICS);
  async function publication(expected: Message, since: number): Promise<void> {
    const deadline = Date.now() + 60_000;
    while (!notifications.slice(since).some((message) => isDeepStrictEqual(message, expected))) {
      assert.ok(Date.now() < deadline, `no full reference publication ${JSON.stringify(expected)}`);
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    record("complete reference publication", expected);
  }
  async function formatting(name: "legacy" | "modern", expected: unknown): Promise<void> {
    const result = await request("textDocument/formatting", {
      textDocument: { uri: document(name) },
      options: { tabSize: 2, insertSpaces: true },
    });
    assert.deepEqual(result, expected, "whole authored Vim reference formatting result");
    record(`reference formatting: ${name}`, result);
  }
  function latest(): void {
    for (const expected of [legacyPacket, modernPacket(), controlPacket, ...oldAppPackets])
      assertLatestPublication(notifications, expected);
  }
  async function barrier(modernFormatting: unknown): Promise<void> {
    await formatting("modern", modernFormatting);
    const controlFormatting = await request("textDocument/formatting", {
      textDocument: { uri: uri(path.join(control, "QuotesPanel.vue")) },
      options: { tabSize: 2, insertSpaces: true },
    });
    assert.deepEqual(controlFormatting, [
      {
        range: { start: { line: 0, character: 0 }, end: { line: 2, character: 0 } },
        newText: DOUBLE_FORMATTED,
      },
    ]);
    record("unchanged sibling complete formatting", controlFormatting);
    record("reference request barrier", await request("workspace/symbol", { query: "" }));
    latest();
    assert.equal(fs.readFileSync(file("src/modern/App.vue"), "utf8"), REFERENCE_SOURCE);
    assert.equal(fs.readFileSync(file("src/legacy/App.vue"), "utf8"), REFERENCE_SOURCE);
    assert.deepEqual(
      configSourceSnapshot(control),
      controlSources,
      "whole sibling source/config snapshot unchanged",
    );
  }
  function watch(changes: { uri: string; type: number }[]): number {
    const since = notifications.length;
    record("reference watch input and whole source snapshots", {
      changes,
      app: configSourceSnapshot(root),
      control: configSourceSnapshot(control),
    });
    send({ method: "workspace/didChangeWatchedFiles", params: { changes } });
    return since;
  }
  const fixtures = referenceWorkspaceFiles();
  for (const [name, source] of Object.entries(fixtures)) {
    fs.mkdirSync(path.dirname(file(name)), { recursive: true });
    fs.writeFileSync(file(name), source);
  }
  record("expected reference owner inputs (not observed private owner keys)", {
    legacy: {
      root,
      tsconfig: file("tsconfig.legacy.json"),
      documentUri: document("legacy"),
      target: 2.7,
    },
    modern: {
      root,
      tsconfig: file("tsconfig.modern.json"),
      documentUri: document("modern"),
      target: 3.5,
    },
  });
  let since = watch([
    { uri: uri(file("vite.config.ts")), type: 2 },
    { uri: uri(file("tsconfig.json")), type: 2 },
    ...["vue-policy.mjs", "tsconfig.legacy.json", "tsconfig.modern.json"].map((name) => ({
      uri: uri(file(name)),
      type: 1,
    })),
  ]);
  // Previously open app documents remain open: disabling typecheck must clear
  // their complete packets instead of silently retaining the earlier TS2322.
  for (const expected of oldAppPackets) await publication(expected, since);
  for (const name of ["legacy", "modern"] as const) {
    send({
      method: "textDocument/didOpen",
      params: {
        textDocument: {
          uri: document(name),
          languageId: "vue",
          version: 1,
          text: REFERENCE_SOURCE,
        },
      },
    });
    await publication(name === "legacy" ? legacyPacket : modernPacket(), since);
    await formatting(name, null);
    const hover = await request("textDocument/hover", {
      textDocument: { uri: document(name) },
      position: { line: 1, character: 8 },
    });
    assert.deepEqual(hover, null, "authored reference policy disables hover");
    record(`reference disabled hover: ${name}`, hover);
  }
  modernVersion = 2;
  since = notifications.length;
  send({
    method: "textDocument/didChange",
    params: {
      textDocument: { uri: document("modern"), version: modernVersion },
      contentChanges: [{ text: REFERENCE_DIRTY_SOURCE }],
    },
  });
  record("unsaved modern buffer input", {
    uri: document("modern"),
    version: modernVersion,
    text: REFERENCE_DIRTY_SOURCE,
  });
  await publication(modernPacket(), since);
  await barrier(null);

  const helper = file("vue-policy.mjs");
  fs.writeFileSync(helper, "export default {\n");
  since = watch([{ uri: uri(helper), type: 2 }]);
  const expectedError = expectedConfigReloadError(uri(root), [uri(helper)]);
  const deadline = Date.now() + 60_000;
  while (!notifications.slice(since).some((message) => isDeepStrictEqual(message, expectedError))) {
    assert.ok(Date.now() < deadline, "same-directory invalid helper did not notify the editor");
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  record("complete reference configuration error", expectedError);
  await publication(legacyPacket, since);
  await publication(modernPacket(), since);
  await barrier(null);
  for (const message of notifications.slice(since))
    if (message.method === "window/showMessage") assert.deepEqual(message, expectedError);

  fs.writeFileSync(helper, `${REFERENCE_POLICY.replace("formatting:false", "formatting:true")}\n`);
  since = watch([{ uri: uri(helper), type: 2 }]);
  await publication(legacyPacket, since);
  await publication(modernPacket(), since);
  await barrier(REFERENCE_FORMATTED);
  assert.deepEqual(
    notifications.slice(since).filter((message) => message.method === "window/showMessage"),
    [],
    "valid reference helper recovery must not report another config error",
  );
  record("reference acceptance scope", {
    covered:
      "same-directory2.7/3.5 full authored packets and retained dirty version2 in the same installed session",
    unproven: "observed private owner keys and actual packaged editor hosts",
  });
}
