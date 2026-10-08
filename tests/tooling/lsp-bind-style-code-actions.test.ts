import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { decodeFrames } from "../differential/lsp-wire.ts";
import { root } from "./support/lsp/paths.ts";
import {
  actions,
  apply,
  diagnostics,
  HTML,
  PROP,
  type Action,
} from "./support/lsp/bind-style/expected.ts";

import { compareCliFixes } from "./support/lsp/bind-style/cli.ts";
import { prepareProviders } from "./support/lsp/bind-style/provider.ts";
import { closePublication } from "./support/lsp/bind-style/close.ts";

const fixture = path.join(root, "tests/_fixtures/differential/lsp/bind-style-code-actions");
const observer = fileURLToPath(new URL("./support/lsp/bind-style/observe.mjs", import.meta.url));
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

await test("whole original bind-style diagnostics have configured fixes and suppression over real stdio", async () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "manifest.json"), "utf8"));
  for (const row of manifest.files) {
    const bytes = fs.readFileSync(path.join(fixture, row.path));
    assert.equal(bytes.length, row.bytes);
    assert.equal(digest(bytes), row.sha256);
  }
  const parent = fs.readFileSync(path.join(fixture, "Parent.vue.txt"), "utf8");
  const child = fs.readFileSync(path.join(fixture, "Child.vue.txt"), "utf8");
  assert.notEqual(process.platform, "win32", "original stdio observer is POSIX scoped");
  const publicAuthority = process.env.VIZE_BIND_STYLE_PUBLIC_AUTHORITY;
  let binary: string;
  let authority: unknown;
  if (publicAuthority) {
    const bytes = fs.readFileSync(publicAuthority);
    assert.equal(digest(bytes), process.env.VIZE_BIND_STYLE_PUBLIC_AUTHORITY_SHA);
    const receipt = JSON.parse(bytes.toString());
    assert.equal(receipt.schema, "vize.root-existing-public-434-for-original7998/v1");
    assert.equal(receipt.version, "0.434.0");
    assert.equal(receipt.actualFixAncestor, true);
    binary = fs.realpathSync(receipt.distribution.binaryPath);
    assert.equal(digest(fs.readFileSync(binary)), receipt.distribution.binarySha256);
    assert.equal(
      digest(fs.readFileSync(receipt.distribution.archivePath)),
      receipt.distribution.archiveSha256,
    );
    authority = receipt;
  } else {
    const build = expectedBuildIdentity(root);
    binary = path.join(root, build.binaryPath);
    authority = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
    validateBuildReceipt(authority, build);
  }
  const output =
    process.env.VIZE_BIND_STYLE_OUTPUT ??
    path.join(root, "target/differential/lsp-bind-style-code-actions");
  fs.mkdirSync(output, { recursive: true });
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(output, "original-project-")));
  for (const name of ["vize.config.json", "tsconfig.json"])
    fs.copyFileSync(path.join(fixture, name), path.join(workspace, name));
  fs.writeFileSync(path.join(workspace, "Parent.vue"), parent);
  fs.writeFileSync(path.join(workspace, "Child.vue"), child);
  fs.writeFileSync(
    path.join(workspace, "lsp-req.mjs"),
    fs.readFileSync(path.join(fixture, "lsp-req.mjs.txt")),
  );
  const providers = prepareProviders(root, workspace, Boolean(publicAuthority));
  fs.mkdirSync(path.join(workspace, "node_modules/.bin"), { recursive: true });
  fs.symlinkSync(binary, path.join(workspace, "node_modules/.bin/vize"));
  const uri = pathToFileURL(path.join(workspace, "Parent.vue")).href;
  const rows: Array<Record<string, unknown>> = [];
  const record = {
    manifest,
    authority,
    providers,
    environment: { ...process.env },
    binary,
    binarySha256: digest(fs.readFileSync(binary)),
    parent,
    child,
    rows,
    status: "PENDING",
    error: null as string | null,
    publicScope:
      "original issue and authored UTF16/CRLF controls; no performance or native-stage credit",
  };
  const save = () =>
    fs.writeFileSync(path.join(output, "result.json"), `${JSON.stringify(record, null, 2)}\n`);
  save();
  const savedEnvironment = { ...process.env };
  let stopObservation = () => {};
  let awaitObservedClosure: (() => Promise<void>) | undefined;
  let session: import("./support/lsp/session.ts").LspSession | undefined;
  try {
    compareCliFixes(binary, fixture, output, rows, save, (project) =>
      prepareProviders(root, project, Boolean(publicAuthority)),
    );
    const capture = path.join(output, "literal-original");
    fs.mkdirSync(capture, { recursive: true });
    const original = spawnSync(
      process.execPath,
      [
        "--import",
        observer,
        path.join(workspace, "lsp-req.mjs"),
        workspace,
        "Parent.vue",
        "textDocument/codeAction",
      ],
      {
        cwd: workspace,
        env: {
          ...process.env,
          VIZE_BIND_STYLE_CAPTURE_ROOT: capture,
          VIZE_BIND_STYLE_BINARY: binary,
        },
        timeout: 35000,
        detached: true,
      },
    );
    if (original.error) {
      try {
        process.kill(-original.pid, "SIGKILL");
      } catch {
        /* The owned process group already exited. */
      }
    }
    fs.writeFileSync(path.join(capture, "stdout.bin"), original.stdout ?? Buffer.alloc(0));
    fs.writeFileSync(path.join(capture, "stderr.bin"), original.stderr ?? Buffer.alloc(0));
    rows.push({
      method: "literal original client",
      status: original.status,
      signal: original.signal,
      error: original.error?.message ?? null,
      stdout: original.stdout?.toString(),
      stderr: original.stderr?.toString(),
      shutdown:
        "original params:null is preserved and unasserted; separate positive session uses omitted params",
    });
    save();
    assert.equal(
      original.stdout.toString(),
      '8:9 vue/prefer-props-shorthand: ["Fix: Use shorthand prop syntax","Suppress with @vize:forget (vue/prefer-props-shorthand)"]\n' +
        '8:9 vue/v-bind-style: ["Fix: Use shorthand syntax","Suppress with @vize:forget (vue/v-bind-style)"]\n' +
        '9:9 vue/prefer-props-shorthand: ["Fix: Use shorthand prop syntax","Suppress with @vize:forget (vue/prefer-props-shorthand)"]\n' +
        '10:7 vue/no-v-html: ["Suppress with @vize:forget (vue/no-v-html)"]\n',
    );
    const literalWire = checkCapture(capture, false);
    const requests = literalWire.client.filter(
      (message) => message.method === "textDocument/codeAction",
    );
    assert.equal(requests.length, 4);
    for (const [index, message] of requests.entries()) {
      const diagnostic = diagnostics(parent)[index];
      assert.deepEqual(message.params, {
        textDocument: { uri },
        range: diagnostic.range,
        context: { diagnostics: [diagnostic] },
      });
      assert.deepEqual(
        literalWire.server.find((response) => response.id === message.id),
        {
          jsonrpc: "2.0",
          id: message.id,
          result: actions(parent, uri, diagnostic),
        },
      );
    }

    process.env.VIZE_LSP_BIN = binary;
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = publicAuthority ? "0" : "1";
    process.env.VIZE_BIND_STYLE_CAPTURE_ROOT = path.join(output, "authored-positive");
    process.env.VIZE_BIND_STYLE_BINARY = binary;
    ({ stopObservation, awaitObservedClosure } =
      await import("./support/lsp/bind-style/observe.mjs"));
    const { LspSession } = await import("./support/lsp/session.ts");
    session = new LspSession();
    rows.push({
      method: "initialize",
      result: await session.request("initialize", {
        processId: process.pid,
        rootUri: pathToFileURL(workspace).href,
        capabilities: {},
      }),
    });
    session.notify("initialized", {});
    for (const [name, text] of [
      ["Child.vue", child],
      ["Parent.vue", parent],
    ])
      session.notify("textDocument/didOpen", {
        textDocument: {
          uri: pathToFileURL(path.join(workspace, name)).href,
          languageId: "vue",
          version: 1,
          text,
        },
      });
    let version = 1;
    await publication(version, parent, diagnostics(parent));
    for (const diagnostic of diagnostics(parent)) await request(parent, diagnostic);
    for (const diagnostic of diagnostics(parent)) {
      const suppressed = apply(parent, actions(parent, uri, diagnostic).at(-1)!, uri);
      // The payload is a reason: primary visitor disables ALL rules on the next element.
      const expected = diagnostics(suppressed).filter(
        (row) => row.range.start.line !== diagnostic.range.start.line + 1,
      );
      await change(++version, suppressed, expected);
      await change(++version, parent, diagnostics(parent));
    }
    // Both one-edit fixes apply independently to the complete original document.
    const expectedOnce = parent.replace('v-bind:title="title"', ':title="title"');
    assert.equal(apply(parent, actions(parent, uri, diagnostics(parent)[1])[0], uri), expectedOnce);
    const final = expectedOnce.replaceAll(':title="title"', ":title");
    let current = expectedOnce;
    await change(++version, current, diagnostics(current));
    for (let index = 0; index < 2; index++) {
      const diagnostic = diagnostics(current).find((d) => d.code === PROP)!;
      const actual = await request(current, diagnostic);
      current = apply(current, actual[0], uri);
      await change(++version, current, diagnostics(current));
    }
    assert.equal(current, final);
    assert.deepEqual(
      diagnostics(current).map((d) => d.code),
      [HTML],
    );
    await request(current, diagnostics(current)[0]);
    // Same rules and whole WorkspaceEdits with a surrogate pair before the carrier.
    for (const newline of ["\n", "\r\n"]) {
      const variant = parent
        .replace("  <Child v-bind", "\t<!-- 😀 --> <Child v-bind")
        .replaceAll("\n", newline);
      await change(++version, variant, diagnostics(variant));
      for (const diagnostic of diagnostics(variant)) await request(variant, diagnostic);
    }
    await change(++version, parent, diagnostics(parent));
    assert.equal(fs.readFileSync(path.join(workspace, "Parent.vue"), "utf8"), parent);
    const close = await closePublication(session, uri);
    const closed = close.actual;
    rows.push({ method: "didClose publication", result: closed, liveObserved: close.observed });
    save();
    assert.deepEqual(closed, { uri, diagnostics: [] });
    await session.shutdown();
    session = undefined;
    await awaitObservedClosure();
    checkCapture(path.join(output, "authored-positive"));
    record.status = "PASS";
  } catch (error) {
    record.status = "FAIL";
    record.error = String(error);
    throw error;
  } finally {
    try {
      await session?.shutdown();
      await awaitObservedClosure?.();
    } catch (error) {
      record.status = "FAIL";
      record.error ??= String(error);
    }
    save();
    stopObservation();
    for (const name of [
      "VIZE_LSP_BIN",
      "VIZE_LSP_REQUIRE_SOURCE_BUILD",
      "VIZE_BIND_STYLE_CAPTURE_ROOT",
      "VIZE_BIND_STYLE_BINARY",
    ]) {
      if (savedEnvironment[name] === undefined) delete process.env[name];
      else process.env[name] = savedEnvironment[name];
    }
  }
  assert.equal(record.status, "PASS", record.error ?? "all whole results must pass");

  async function publication(version: number, source: string, expected: unknown[]) {
    assert.ok(session);
    const actual = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (p) =>
        (p as { uri: string; version: number }).uri === uri &&
        (p as { version: number }).version === version,
    );
    rows.push({ method: "publishDiagnostics", source, actual });
    save();
    assert.deepEqual(actual, { uri, version, diagnostics: expected });
  }
  async function change(version: number, source: string, expected: unknown[]) {
    session!.notify("textDocument/didChange", {
      textDocument: { uri, version },
      contentChanges: [{ text: source }],
    });
    await publication(version, source, expected);
  }
  async function request(source: string, diagnostic: ReturnType<typeof diagnostics>[number]) {
    const params = {
      textDocument: { uri },
      range: diagnostic.range,
      context: { diagnostics: [diagnostic] },
    };
    const row: Record<string, unknown> = { method: "textDocument/codeAction", source, params };
    rows.push(row);
    try {
      row.actual = await session!.request("textDocument/codeAction", params);
      save();
      assert.deepEqual(row.actual, actions(source, uri, diagnostic));
      return row.actual as Action[];
    } catch (error) {
      row.error = String(error);
      save();
      throw error;
    }
  }
  function checkCapture(directory: string, closureRequired = true) {
    const children = fs.readdirSync(directory).filter((name) => name.startsWith("process-"));
    assert.equal(children.length, 1);
    const folder = path.join(directory, children[0]);
    const receipt = JSON.parse(fs.readFileSync(path.join(folder, "process.json"), "utf8"));
    if (closureRequired) {
      assert.equal(receipt.state, "closed");
      assert.equal(receipt.error, null);
      assert.equal(receipt.exitCode, 0);
      assert.equal(receipt.signal, null);
    }
    const decoded: Record<string, ReturnType<typeof decodeFrames>["messages"]> = {};
    for (const stream of ["client", "server", "stderr"]) {
      const bytes = fs.readFileSync(path.join(folder, `${stream}.bin`));
      assert.equal(bytes.length, receipt.streams[stream]);
      if (stream !== "stderr") {
        decoded[stream] = decodeFrames(bytes).messages;
        assert.ok(decoded[stream].length > 0);
      }
    }
    return { receipt, client: decoded.client, server: decoded.server };
  }
});
