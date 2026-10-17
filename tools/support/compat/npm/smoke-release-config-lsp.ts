/** Same installed server, two owners; full packets rather than code-only findings. */
import assert from "node:assert/strict";
import type { ChildProcessWithoutNullStreams } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import {
  assertLatestPublication,
  expectedConfigReloadError,
  type Message,
  CONFIG_PACKAGE_NAMES,
  CONFIG_SOURCES,
  DOUBLE_FORMATTED,
  MALFORMED_CONFIG_DEPENDENCIES,
  SINGLE_FORMATTED,
  tsconfigBase,
  typeDiagnostics,
  vueHelper,
} from "./smoke-release-config-fixture.ts";
import {
  assertConfigCli,
  assertConfigDependencyFailure,
  configSourceSnapshot,
  runConfigReferenceWorkspaceLsp,
} from "./smoke-release-config-workspace.ts";

export {
  assertLatestPublication,
  expectedConfigReloadError,
} from "./smoke-release-config-fixture.ts";

export interface ConfigLspReceipt {
  stdoutBase64: string;
  stderrBase64: string;
  stderr: string;
  closed: boolean;
  status: number | null;
  signal: ChildProcessWithoutNullStreams["signalCode"];
  failures: string[];
  incompleteFrameBase64: string;
}

/** Config-mode custody starts before initialize and ends only after child close. */
export async function runConfigLspSession(
  server: ChildProcessWithoutNullStreams,
  session: {
    run: () => Promise<void>;
    shutdown: () => Promise<unknown>;
    exit: () => void;
    protocolState: () => { remainder: Buffer; error?: unknown };
    save: (receipt: ConfigLspReceipt) => void;
    closeTimeoutMs?: number;
  },
): Promise<void> {
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  const failures: string[] = [];
  let failed = false;
  let firstError: unknown;
  function fail(error: unknown): void {
    failures.push(error instanceof Error ? `${error.name}: ${error.message}` : String(error));
    if (!failed) {
      failed = true;
      firstError = error;
    }
  }
  server.stdout.on("data", (chunk: Buffer) => stdout.push(Buffer.from(chunk)));
  server.stderr.on("data", (chunk: Buffer) => stderr.push(Buffer.from(chunk)));
  server.on("error", fail);
  server.stdin.on("error", fail);
  let closed = false;
  let status: number | null = null;
  let signal: ChildProcessWithoutNullStreams["signalCode"] = null;
  const close = new Promise<void>((resolve) => {
    server.once("close", (code, receivedSignal) => {
      closed = true;
      status = code;
      signal = receivedSignal;
      resolve();
    });
  });
  try {
    await session.run();
  } catch (error) {
    fail(error);
  }
  try {
    await session.shutdown();
  } catch (error) {
    fail(error);
  }
  try {
    session.exit();
  } catch (error) {
    fail(error);
  }
  async function waitForClose(timeout: number): Promise<boolean> {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      return await Promise.race([
        close.then(() => true),
        new Promise<boolean>((resolve) => {
          timer = setTimeout(() => resolve(false), timeout);
        }),
      ]);
    } finally {
      clearTimeout(timer);
    }
  }
  if (!(await waitForClose(session.closeTimeoutMs ?? 60_000))) {
    fail(new Error("config LSP child did not close after shutdown/exit"));
    try {
      server.kill("SIGKILL");
    } catch (error) {
      fail(error);
    }
    if (!(await waitForClose(5_000))) fail(new Error("config LSP child did not close after kill"));
  }
  if (status !== 0 || signal !== null)
    fail(new Error(`config LSP child exited with status ${status}, signal ${signal}`));
  const protocol = session.protocolState();
  if (protocol.error !== undefined) fail(protocol.error);
  if (protocol.remainder.length !== 0)
    fail(
      new Error(`config LSP child left an incomplete frame (${protocol.remainder.length} bytes)`),
    );
  const stderrBytes = Buffer.concat(stderr);
  try {
    session.save({
      stdoutBase64: Buffer.concat(stdout).toString("base64"),
      stderrBase64: stderrBytes.toString("base64"),
      stderr: stderrBytes.toString("utf8"),
      closed,
      status,
      signal,
      failures,
      incompleteFrameBase64: protocol.remainder.toString("base64"),
    });
  } catch (error) {
    fail(error);
  }
  if (failed) throw firstError;
}

export interface Session {
  workspace: string;
  notifications: Message[];
  request: (method: string, params: unknown) => Promise<unknown>;
  send: (message: object) => void;
  record: (name: string, result: unknown) => void;
}

/** A dependency cell always restores, without replacing its first scenario failure. */
export async function runConfigDependencyCell(cell: {
  dependency: string;
  failure: "missing" | "malformed";
  run: () => Promise<void>;
  restore: () => Promise<void>;
  record: Session["record"];
}): Promise<void> {
  let failed = false;
  let firstError: unknown;
  try {
    await cell.run();
  } catch (error) {
    failed = true;
    firstError = error;
  }
  try {
    await cell.restore();
  } catch (error) {
    if (!failed) {
      failed = true;
      firstError = error;
    }
    cell.record("configuration dependency restoration failure", {
      dependency: cell.dependency,
      failure: cell.failure,
      error:
        error instanceof Error
          ? { name: error.name, message: error.message, stack: error.stack }
          : error,
    });
  }
  if (failed) throw firstError;
}

const uri = (file: string) => pathToFileURL(file).href;

export async function runConfigWorkspaceLsp(session: Session): Promise<void> {
  const { workspace, notifications, request, send, record } = session;
  const project = (name: string) => path.join(workspace, "packages", name);
  const typed = (name: string) => uri(path.join(project(name), "TypedPanel.vue"));
  const expectedPublication = (name: string, broken: boolean) => ({
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: typed(name), version: 1, diagnostics: typeDiagnostics(broken) },
  });
  let currentAppBroken = true;
  async function publication(name: string, broken: boolean, since: number): Promise<void> {
    const expected = expectedPublication(name, broken);
    const deadline = Date.now() + 60_000;
    for (;;) {
      const found = notifications
        .slice(since)
        .find((message) => isDeepStrictEqual(message, expected));
      if (found !== undefined) {
        assert.deepEqual(found, expected);
        if (name === "app") currentAppBroken = broken;
        record(`full native publication: ${name}`, found);
        return;
      }
      assert.ok(Date.now() < deadline, `no complete publication ${JSON.stringify(expected)}`);
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
  }
  async function formatting(name: string, configured: boolean, broken: boolean): Promise<void> {
    const result = await request("textDocument/formatting", {
      textDocument: { uri: uri(path.join(project(name), "QuotesPanel.vue")) },
      options: { tabSize: 2, insertSpaces: true },
    });
    assert.deepEqual(
      result,
      [
        {
          range: { start: { line: 0, character: 0 }, end: { line: 2, character: 0 } },
          newText: configured ? SINGLE_FORMATTED : DOUBLE_FORMATTED,
        },
      ],
      `complete format edit vector for ${name}`,
    );
    record(`full native formatting: ${name}`, result);
    assertLatestPublication(notifications, expectedPublication(name, broken));
  }
  async function unchangedControl(since: number): Promise<void> {
    assertConfigCli(project("control"), false, false);
    await formatting("control", false, false);
    assertLatestPublication(notifications, expectedPublication("app", currentAppBroken));
    for (const message of notifications.slice(since)) {
      const params = message.params as { uri?: string } | undefined;
      if (
        message.method === "textDocument/publishDiagnostics" &&
        params?.uri === typed("control")
      ) {
        assert.deepEqual(message, expectedPublication("control", false));
      }
    }
  }
  async function configurationError(file: string, since: number): Promise<void> {
    const expected = expectedConfigReloadError(uri(project("app")), [uri(file)]);
    const deadline = Date.now() + 60_000;
    for (;;) {
      const notice = notifications
        .slice(since)
        .find((message) => message.method === "window/showMessage");
      if (notice !== undefined) {
        assert.deepEqual(notice, expected, "complete explicit configuration-error envelope");
        record("full native configuration error", notice);
        return;
      }
      assert.ok(
        Date.now() < deadline,
        `no explicit configuration error ${JSON.stringify(expected)}`,
      );
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
  }
  async function retainedTypecheck(): Promise<void> {
    const result = await request("textDocument/hover", {
      textDocument: { uri: typed("app") },
      position: { line: 1, character: 8 },
    });
    assert.deepEqual(
      result,
      {
        contents: { kind: "markdown", value: "```typescript\nconst value: string\n```" },
        range: { start: { line: 1, character: 6 }, end: { line: 1, character: 11 } },
      },
      "retained native checker still serves the same complete typed hover",
    );
    record("retained native checker hover", result);
    assertLatestPublication(notifications, expectedPublication("app", true));
  }
  function watch(file: string, type: 1 | 2 | 3): number {
    const since = notifications.length;
    record("configuration watch input", {
      file,
      type,
      app: configSourceSnapshot(project("app")),
      control: configSourceSnapshot(project("control")),
    });
    send({
      method: "workspace/didChangeWatchedFiles",
      params: { changes: [{ uri: uri(file), type }] },
    });
    return since;
  }
  for (const name of CONFIG_PACKAGE_NAMES) {
    for (const file of ["TypedPanel.vue", "QuotesPanel.vue"] as const) {
      send({
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: uri(path.join(project(name), file)),
            languageId: "vue",
            version: 1,
            text: CONFIG_SOURCES[file],
          },
        },
      });
    }
    await publication(name, name === "app", 0);
    await formatting(name, name === "app", name === "app");
  }
  const helper = path.join(project("app"), "vue-settings.ts");
  const base = path.join(project("app"), "tsconfig.base.json");
  fs.writeFileSync(helper, vueHelper(false));
  let since = watch(helper, 2);
  // No document request/edit/restart between the event and the new publication.
  await publication("app", true, since);
  await formatting("app", false, true);
  await unchangedControl(since);
  assertConfigCli(project("app"), false, true);
  fs.writeFileSync(helper, vueHelper(true));
  since = watch(helper, 2);
  await publication("app", true, since);
  await formatting("app", true, true);
  await unchangedControl(since);
  fs.writeFileSync(base, tsconfigBase(false));
  since = watch(base, 2);
  await publication("app", false, since);
  await formatting("app", true, false);
  await unchangedControl(since);
  assertConfigCli(project("app"), true, false);
  fs.writeFileSync(base, tsconfigBase(true));
  since = watch(base, 2);
  await publication("app", true, since);
  await formatting("app", true, true);
  await unchangedControl(since);

  // Missing and syntax-invalid dependencies retain the last good owner with
  // the same explicit error. Repairs reload without source edits or a restart.
  // The selected root Vite file is not deleted in these dependency controls.
  for (const file of [helper, base]) {
    const dependency = file === helper ? "vue-settings.ts" : "tsconfig.base.json";
    for (const failure of ["missing", "malformed"] as const) {
      const original = fs.readFileSync(file, "utf8");
      await runConfigDependencyCell({
        dependency,
        failure,
        record,
        run: async () => {
          if (failure === "missing") fs.unlinkSync(file);
          else fs.writeFileSync(file, MALFORMED_CONFIG_DEPENDENCIES[dependency]);
          since = watch(file, failure === "missing" ? 3 : 2);
          await configurationError(file, since);
          await formatting("app", true, true);
          await retainedTypecheck();
          await unchangedControl(since);
          assertConfigDependencyFailure(project("app"), dependency);
          const expected = expectedConfigReloadError(uri(project("app")), [uri(file)]);
          for (const message of notifications.slice(since)) {
            if (message.method === "window/showMessage") assert.deepEqual(message, expected);
          }
          // CREATE repairs missing files; CHANGED repairs existing malformed files.
          // Different settings distinguish a reload from retained-state publication.
          fs.writeFileSync(file, file === helper ? vueHelper(false) : tsconfigBase(false));
          since = watch(file, failure === "missing" ? 1 : 2);
          await publication("app", file === helper, since);
          await formatting("app", file !== helper, file === helper);
          await unchangedControl(since);
          assert.deepEqual(
            notifications.slice(since).filter((message) => message.method === "window/showMessage"),
            [],
            "valid repaired dependency must not produce another configuration error",
          );
          assertConfigCli(project("app"), file !== helper, file === helper);
        },
        restore: async () => {
          const event = fs.existsSync(file) ? 2 : 1;
          fs.writeFileSync(file, original);
          since = watch(file, event);
          await publication("app", true, since);
          await formatting("app", true, true);
          await unchangedControl(since);
          assert.deepEqual(
            notifications.slice(since).filter((message) => message.method === "window/showMessage"),
            [],
            "restoring the original valid dependency must not produce another configuration error",
          );
          assertConfigCli(project("app"), true, true);
        },
      });
    }
  }
  record("final workspace request barrier", await request("workspace/symbol", { query: "" }));
  assertLatestPublication(notifications, expectedPublication("app", true));
  assertLatestPublication(notifications, expectedPublication("control", false));
  record("acceptance scope", {
    covered: "two distinct package roots using the installed native LSP",
    unproven: "actual packaged editor hosts; same-directory phase follows in this same session",
  });
  await runConfigReferenceWorkspaceLsp(session);
}
