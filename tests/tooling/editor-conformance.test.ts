// TS-45 (Davinci P5-12): the judge, the scenario and the CI wiring of the
// multi-client LSP conformance suite in `tests/editor-conformance/`. The
// editors themselves run in `.github/workflows/editor-conformance.yml`; this
// file proves the judge is exact, that every expectation is load-bearing, and
// that the scenario's text expectations follow from its response expectations.
import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

import {
  evaluate,
  type Entry,
  type Scenario,
  WORKSPACE,
} from "../editor-conformance/conformance.ts";
import { corruptExpectation, negativeControl } from "../editor-conformance/negative-control.ts";
import { loadScenario, suiteRoot } from "../editor-conformance/support/context.ts";
import { applyTextEdits, workspaceEditsFor } from "../editor-conformance/support/document.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const root = "/work/space";
const scenario = loadScenario();
const uri = `file://${root}/${scenario.document}`;
const key = `${WORKSPACE}/${scenario.document}`;
const byId = (id: string) => scenario.steps.find((step) => step.id === id)!;

/** The transcript a perfectly conformant client and server would produce. */
function idealTranscript(source: Scenario = scenario): Entry[] {
  const entries: Entry[] = [];
  let id = 0;
  let version = 0;
  let published: unknown[] = [];
  const push = (dir: "c2s" | "s2c", msg: object) =>
    entries.push({
      session: 7,
      t: entries.length,
      dir,
      msg: JSON.parse(JSON.stringify(msg).replaceAll(WORKSPACE, `file://${root}`)),
    });
  for (const step of source.steps) {
    const expect = step.expect as any;
    if (step.kind === "initialize") {
      push("c2s", {
        id: ++id,
        method: "initialize",
        params: { initializationOptions: source.initializationOptions },
      });
      push("s2c", {
        id,
        result: { capabilities: expect.capabilities, serverInfo: { name: expect.serverName } },
      });
    } else if (step.kind === "open") {
      push("c2s", {
        method: "textDocument/didOpen",
        params: { textDocument: { uri, languageId: source.languageId, version, text: expect } },
      });
    } else if (step.kind === "diagnostics") {
      published = expect;
      push("s2c", {
        method: "textDocument/publishDiagnostics",
        params: { uri, version, diagnostics: expect },
      });
    } else if (step.kind === "request") {
      push("c2s", {
        id: ++id,
        method: step.method,
        params: {
          textDocument: { uri },
          ...step.match,
          ...(step.diagnosticsFromRequest ? { context: { diagnostics: published } } : {}),
        },
      });
      push("s2c", { id, result: expect });
    } else if (step.kind === "text") {
      push("c2s", {
        method: "textDocument/didChange",
        params: { textDocument: { uri, version: ++version }, contentChanges: [{ text: expect }] },
      });
    } else {
      push("c2s", { id: ++id, method: "shutdown" });
      push("s2c", { id, result: null });
      push("c2s", { method: "exit" });
      entries.push({ session: 7, t: entries.length, event: "exit", code: expect.exitCode });
    }
  }
  return entries;
}

const failures = (entries: Entry[], source: Scenario = scenario) =>
  evaluate(source, entries, [root])
    .steps.filter((step) => !step.pass)
    .map((step) => step.id);

test("the scenario covers the P5-12 contract and every expectation file exists", () => {
  const raw = JSON.parse(fs.readFileSync(`${suiteRoot}/scenario.json`, "utf8")) as Scenario;
  const methods = raw.steps.flatMap((step) => (step.kind === "request" ? [step.method] : []));
  assert.deepEqual(methods, [
    "textDocument/hover",
    "textDocument/completion",
    "textDocument/definition",
    "textDocument/codeAction",
    "textDocument/formatting",
    "textDocument/rename",
  ]);
  assert.deepEqual(
    [...new Set(raw.steps.map((step) => step.kind))],
    ["initialize", "open", "diagnostics", "request", "text", "shutdown"],
  );
  for (const step of raw.steps as Array<{ expect: any }>) {
    if (step.expect?.$file != null)
      assert.ok(fs.existsSync(`${suiteRoot}/${step.expect.$file}`), step.expect.$file);
  }
  assert.equal(
    new Set(raw.steps.map((step) => step.id)).size,
    raw.steps.length,
    "step ids are unique",
  );
});

test("text expectations follow from the response expectations (independent oracle)", () => {
  const open = byId("open").expect as string;
  const fix = (byId("code-action").expect as any[]).find(
    (action) => action.title === (byId("apply-quick-fix") as any).action.title,
  );
  const fixed = applyTextEdits(open, workspaceEditsFor(fix.edit, key));
  assert.equal(fixed, byId("apply-quick-fix").expect);
  const formatted = applyTextEdits(fixed, byId("formatting").expect as any);
  assert.equal(formatted, byId("apply-formatting").expect);
  const typed = (byId("edit") as any).action;
  const edited = applyTextEdits(formatted, [{ range: typed.replace, newText: typed.text }]);
  assert.equal(edited, byId("edit").expect);
  assert.equal(
    applyTextEdits(edited, workspaceEditsFor(byId("rename").expect, key)),
    byId("apply-rename").expect,
  );
});

test("the judge accepts the ideal transcript and rejects every corrupted expectation", () => {
  assert.deepEqual(failures(idealTranscript()), []);
  assert.deepEqual(negativeControl(scenario, idealTranscript(), [root]), []);
});

test("initialize requires the declared disabled JSX routing hint without weakening exactness", () => {
  const expected = byId("initialize").expect as any;
  assert.deepEqual(expected.capabilities.experimental, { vize: { jsxTypecheck: false } });
  for (const hint of [
    undefined,
    { vize: { jsxTypecheck: true } },
    { vize: { jsxTypecheck: "false" } },
  ]) {
    const entries = idealTranscript();
    const initialize = entries.find((entry) => entry.msg?.result?.serverInfo != null)!;
    if (hint === undefined) delete initialize.msg!.result.capabilities.experimental;
    else initialize.msg!.result.capabilities.experimental = hint;
    assert.deepEqual(failures(entries), ["initialize"]);
  }
});

test("a deliberately wrong expectation fails exactly its own step", () => {
  const steps = scenario.steps.map((step) =>
    step.id === "hover" ? corruptExpectation(step) : step,
  );
  assert.deepEqual(failures(idealTranscript(), { ...scenario, steps }), ["hover"]);
});

test("the judge fails a wrong response, a server error and a missing request", () => {
  const wrong = idealTranscript().map((entry) =>
    entry.msg?.result?.contents != null
      ? { ...entry, msg: { ...entry.msg, result: { contents: "other" } } }
      : entry,
  );
  assert.deepEqual(failures(wrong), ["hover"]);
  const errored = idealTranscript().map((entry) =>
    entry.msg?.result?.contents != null
      ? { ...entry, msg: { id: entry.msg.id, error: { code: -32603, message: "boom" } } }
      : entry,
  );
  assert.deepEqual(failures(errored), ["hover"]);
  const unsent = idealTranscript().filter(
    (entry) => entry.msg?.method !== "textDocument/definition",
  );
  assert.deepEqual(failures(unsent), ["definition"]);
});

test("stale publishes, extra sessions and a crashed shutdown are failures", () => {
  const stale = idealTranscript().map((entry) =>
    entry.msg?.method === "textDocument/publishDiagnostics" &&
    entry.msg.params.diagnostics.length === 0
      ? { ...entry, msg: { ...entry.msg, params: { ...entry.msg.params, version: 1 } } }
      : entry,
  );
  assert.deepEqual(failures(stale), ["diagnostics-edit"]);
  const twice = [
    ...idealTranscript(),
    ...idealTranscript().map((entry) => ({ ...entry, session: 8 })),
  ];
  assert.equal(failures(twice).length, scenario.steps.length);
  const crashed = idealTranscript().map((entry) =>
    entry.event === "exit" ? { ...entry, code: 101 } : entry,
  );
  assert.deepEqual(failures(crashed), ["shutdown"]);
});

test("batched and incremental edits are judged by the states the server applied", () => {
  const entries = idealTranscript();
  const formatting = entries.findIndex(
    (entry) => entry.msg?.params?.contentChanges?.[0]?.text === byId("apply-formatting").expect,
  );
  const typed = entries[formatting + 1];
  assert.equal(typed.msg!.params.contentChanges[0].text, byId("edit").expect);
  // One notification carrying the formatting edit and then the keystroke as
  // an incremental change, the way Neovim debounces them.
  const contentChanges = [
    entries[formatting].msg!.params.contentChanges[0],
    { range: (byId("edit") as any).action.replace, text: (byId("edit") as any).action.text },
  ];
  const batched: Entry = {
    ...typed,
    msg: {
      method: "textDocument/didChange",
      params: { textDocument: { uri, version: 3 }, contentChanges },
    },
  };
  const merged = [...entries.slice(0, formatting), batched, ...entries.slice(formatting + 2)];
  assert.deepEqual(failures(merged), []);
  // Without the keystroke the typed-edit state is never reached.
  const dropped = {
    ...batched,
    msg: {
      ...batched.msg!,
      params: { ...batched.msg!.params, contentChanges: contentChanges.slice(0, 1) },
    },
  };
  // …and the rename that was requested on the wrong document fails with it.
  assert.deepEqual(
    failures([...entries.slice(0, formatting), dropped, ...entries.slice(formatting + 2)]),
    ["edit", "rename"],
  );
});

test("CI runs every client at the editor-host-smoke pins and requires all four results", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "editor-conformance.yml")) as any;
  const smoke = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  assert.deepEqual(workflow.jobs.conformance.strategy.matrix.client, [
    "neovim",
    "helix",
    "zed",
    "vscode",
  ]);
  const steps = workflow.jobs.conformance.steps as Array<{
    name?: string;
    env?: Record<string, string>;
    uses?: string;
    if?: string;
    run?: string;
    with?: Record<string, unknown>;
  }>;
  const pins = Object.assign({}, ...steps.map((step) => step.env ?? {}));
  for (const name of [
    "NVIM_VERSION",
    "NVIM_SHA256",
    "HELIX_VERSION",
    "HELIX_SHA256",
    "VIZE_TEST_VSCODE_VERSION",
  ]) {
    assert.ok(pins[name], `${name} is pinned`);
    assert.match(
      smoke,
      new RegExp(`${name}: "?${pins[name]}"?`),
      `${name} matches the editor-host-smoke pin`,
    );
  }
  const report = workflow.jobs.report.steps.at(-1).run as string;
  assert.match(report, /report\.ts "\$\{RUNNER_TEMP\}\/ts45" --require helix,neovim,vscode,zed$/u);
  assert.equal(workflow.jobs.report.if, "always()");
  const upload = steps.find((step) => step.with?.name === "ts45-${{ matrix.client }}");
  assert.ok(upload?.uses?.startsWith("actions/upload-artifact@"));
  const lifecycle = steps.findIndex(
    (step) => step.name === "Exercise existing VS Code configuration and lifecycle contracts",
  );
  assert.ok(lifecycle > steps.findIndex((step) => step.name === "Prepare pinned VS Code"));
  assert.ok(lifecycle < steps.findIndex((step) => step.name === "Play the TS-45 scenario"));
  const packageExtension = steps.findIndex(
    (step) => step.name === "Exercise original real VS Code authoring contracts",
  );
  assert.ok(packageExtension > lifecycle);
  assert.ok(packageExtension < steps.findIndex((step) => step.name === "Play the TS-45 scenario"));
  assert.equal(steps[packageExtension].if, "matrix.client == 'vscode'");
  assert.match(
    steps[packageExtension].run.trim(),
    /xvfb-run -a vp run --workspace-root test:vscode-extension:host-real$/u,
  );
  assert.doesNotMatch(steps[packageExtension].run, /lsp-tap|rich-host-tap/);
  assert.match(steps[packageExtension].run, /consumed-server\.sha256/);
  const producerSteps = workflow.jobs.server.steps;
  assert.equal(
    producerSteps.find((step: { name?: string }) => step.name === "Build vize CLI").run,
    "cargo build --profile ci -p vize",
  );
  const receipt = producerSteps.find(
    (step: { name?: string }) =>
      step.name === "Record the real server source and executable identity",
  );
  assert.match(receipt.run, /test "\$\(git rev-parse HEAD\)" = "\$\{GITHUB_SHA\}"/);
  assert.match(
    steps.find((step) => step.name === "Make the server executable").run,
    /sha256sum --check vize\.sha256/,
  );
  assert.equal(steps[lifecycle].if, "matrix.client == 'vscode'");
  assert.equal(
    steps[lifecycle].run,
    "xvfb-run -a vp run --workspace-root test:vscode-extension:host",
  );
  const lifecycleEvidence = steps.find(
    (step) => step.with?.name === "vscode-configuration-lifecycle",
  );
  assert.ok(lifecycleEvidence?.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(lifecycleEvidence?.if, "${{ always() && matrix.client == 'vscode' }}");
  assert.equal(lifecycleEvidence?.with?.["include-hidden-files"], true);
  assert.equal(lifecycleEvidence?.with?.["if-no-files-found"], "error");
  assert.deepEqual(String(lifecycleEvidence?.with?.path).trim().split("\n"), [
    "${{ runner.temp }}/rich-host-evidence/",
    "editors/vscode/.vscode-test/host-smoke-real/failure/",
    "editors/vscode/.vscode-test/host-smoke/failure/",
    "editors/vscode/.vscode-test/host-smoke/output-channel/",
    "editors/vscode/.vscode-test/host-smoke/fake-server/events.jsonl",
    "editors/vscode/.vscode-test/host-smoke/fake-server/existing-config-events/",
  ]);
  // A failed-only rerun must replace its earlier result without requiring
  // fresh artifacts from the three clients GitHub does not rerun.
  assert.equal(upload?.with?.name, "ts45-${{ matrix.client }}");
  assert.equal(upload?.with?.overwrite, true);
  const download = workflow.jobs.report.steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("actions/download-artifact@"),
  );
  assert.equal(download?.with?.pattern, "ts45-*");
});

test("every client has a driver and only Zed is a protocol replay", async () => {
  const modes: Record<string, string> = {};
  for (const client of ["helix", "neovim", "vscode", "zed"]) {
    const { driver } = await import(`../editor-conformance/drivers/${client}.ts`);
    modes[client] = driver.mode;
  }
  assert.deepEqual(modes, { helix: "editor", neovim: "editor", vscode: "editor", zed: "replay" });
  const zed = JSON.parse(fs.readFileSync(`${suiteRoot}/clients/zed-1.16.1.json`, "utf8"));
  assert.match(zed.source, /^zed-industries\/zed v1\.16\.1 crates\/lsp\/src\/lsp\.rs /u);
  assert.equal(zed.clientInfo.version, "1.16.1");
});

function codeActionExchange(entries: Entry[]) {
  const request = entries.find((entry) => entry.msg?.method === "textDocument/codeAction")!.msg!;
  const response = entries.find(
    (entry) => entry.dir === "s2c" && entry.msg?.id === request.id,
  )!.msg!;
  const diagnostic = request.params.context.diagnostics.find(
    (value: any) => value.code === "vue/no-multi-spaces",
  );
  return { request, response, diagnostic };
}

function echoActionDiagnostic(response: any, diagnostic: unknown) {
  for (const action of response.result) action.diagnostics = [structuredClone(diagnostic)];
}

test("code actions compare full client diagnostic echoes while published diagnostics stay exact", () => {
  const entries = idealTranscript();
  const { response, diagnostic } = codeActionExchange(entries);
  delete diagnostic.codeDescription;
  echoActionDiagnostic(response, diagnostic);
  assert.deepEqual(failures(entries), []);
  response.result[0].diagnostics[0].codeDescription = { href: "https://unexpected.test/" };
  assert.deepEqual(failures(entries), ["code-action"]);
  delete response.result[0].diagnostics[0].codeDescription;
  response.result[0].diagnostics[0].message = "changed by server";
  assert.deepEqual(failures(entries), ["code-action"]);
});

test("code action echo preserves supplied metadata and rejects changed rule identity", () => {
  const entries = idealTranscript();
  const { response, diagnostic } = codeActionExchange(entries);
  diagnostic.codeDescription = { href: "https://client.test/rule" };
  diagnostic.data = { client: "retained", revision: 7 };
  diagnostic.message = "Client explanation retained";
  echoActionDiagnostic(response, diagnostic);
  assert.deepEqual(failures(entries), []);
  delete response.result[0].diagnostics[0].data;
  assert.deepEqual(failures(entries), ["code-action"]);
  for (const change of [
    { code: "unknown/rule" },
    { source: "another/linter" },
    { range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } } },
  ]) {
    const invalid = idealTranscript();
    const exchange = codeActionExchange(invalid);
    Object.assign(exchange.diagnostic, change);
    echoActionDiagnostic(exchange.response, exchange.diagnostic);
    assert.deepEqual(failures(invalid), ["code-action"]);
  }
});

test("corrupted canonical action documentation fails even when the client omits documentation", () => {
  const entries = idealTranscript();
  const { response, diagnostic } = codeActionExchange(entries);
  delete diagnostic.codeDescription;
  echoActionDiagnostic(response, diagnostic);
  const corrupted = structuredClone(scenario);
  const action = corrupted.steps.find((value) => value.id === "code-action")!;
  (action.expect as any[])[0].diagnostics[0].codeDescription.href = "https://corrupted.test/";
  assert.deepEqual(failures(entries, corrupted), ["code-action"]);
});

test("current completion retains authored candidates and the complete independent Vue globals vector", () => {
  const authored = JSON.parse(fs.readFileSync(`${suiteRoot}/expected/completion.json`, "utf8"));
  const globals = JSON.parse(
    readRepoFile(
      "tests",
      "_fixtures",
      "differential",
      "lsp",
      "template-expression-globals-8015",
      "globals.expected.json",
    ),
  );
  assert.deepEqual(byId("completion").expect, [...authored, ...globals]);
});

test("current initialization derives only source-backed capabilities from the complete prior vector", () => {
  const original = JSON.parse(fs.readFileSync(`${suiteRoot}/expected/initialize.json`, "utf8"));
  original.capabilities.codeActionProvider.codeActionKinds.push(
    "source.organizeImports",
    "source.sortImports",
    "source.removeUnusedImports",
  );
  original.capabilities.monikerProvider = true;
  assert.deepEqual(byId("initialize").expect, original);
  const capabilities = readRepoFile("crates", "vize_maestro", "src", "server", "capabilities.rs");
  assert.match(
    capabilities,
    /moniker_provider: features\.definition\.then_some\(OneOf::Left\(true\)\)/,
  );
  const kinds = readRepoFile(
    "crates",
    "vize_maestro",
    "src",
    "server",
    "capabilities",
    "code_action_tests.rs",
  );
  for (const kind of original.capabilities.codeActionProvider.codeActionKinds.slice(1)) {
    assert.ok(kinds.includes(`CodeActionKind::from("${kind}".to_owned())`));
  }
});
