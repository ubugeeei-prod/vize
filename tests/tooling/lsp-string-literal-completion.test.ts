import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { offsetToPosition } from "./support/lsp/assertions.ts";
import { createMarker, hash, witness } from "./support/lsp/string-literal-witness.ts";
import { resolveTypecheckRuntime } from "./support/typecheck-dependency.ts";

const corpus = path.join(
  root,
  "tests/_fixtures/differential/lsp/string-literal-completion-original",
);
const read = (name: string): string => fs.readFileSync(path.join(corpus, name), "utf8");
const original = read("Page.vue.txt");
const messages = read("messages.ts");
const plainMessages = read("plain-messages.ts.txt");
const tsconfig = read("tsconfig.json");
const nativeOracle: unknown = JSON.parse(read("native-items.expected.json"));
const sources = [
  { name: "original-LF", source: original, version: 1, revision: 1, quote: '"' },
  {
    name: "dirty-CRLF-Unicode",
    source: read("dirty-crlf-unicode.vue.txt"),
    version: 2,
    revision: 2,
    quote: '"',
  },
  {
    name: "dirty-single-quotes",
    source: read("dirty-single-quotes.vue.txt"),
    version: 3,
    revision: 3,
    quote: "'",
  },
  { name: "restored-original", source: original, version: 4, revision: 4, quote: '"' },
  { name: "reopened-client-version-one", source: original, version: 1, revision: 5, quote: '"' },
];

function substitute(value: unknown, slots: Record<string, unknown>): unknown {
  if (typeof value === "string" && Object.hasOwn(slots, value)) return slots[value];
  if (Array.isArray(value)) return value.map((entry) => substitute(entry, slots));
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, entry]) => [key, substitute(entry, slots)]),
    );
  return value;
}

await test("original literal unions keep complete native candidates, resolution and revision ownership", async () => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "string-literal-completion-"));
  const captureRoot = path.join(
    root,
    "target/differential/string-literal-completions",
    path.basename(workspace),
  );
  fs.mkdirSync(captureRoot, { recursive: true });
  const output = path.join(captureRoot, "whole-observations.json");
  const rows: unknown[] = [];
  const capture = {
    schema: "vize.lsp.string-literal-completion.v1",
    complete: false,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    inputs: { original, messages, plainMessages, tsconfig, sources, nativeOracle },
    qualification:
      "Supplemental source-built full stdio oracle; original 10-second labels-only script is preserved, not replayed.",
    rows,
  };
  const record = (row: unknown): void => {
    rows.push(row);
    fs.writeFileSync(output, `${JSON.stringify(capture, null, 2)}\n`);
  };
  record({
    kind: "start",
    workspace,
    hashes: {
      original: hash(original),
      messages: hash(messages),
      tsconfig: hash(tsconfig),
      nativeOracle: hash(read("native-items.expected.json")),
    },
  });
  let session: LspSession | undefined;
  try {
    const manifest = JSON.parse(read("source.json")) as {
      files: { path: string; sha256: string }[];
      controls: { path: string; sha256: string }[];
    };
    for (const entry of [...manifest.files, ...manifest.controls])
      assert.equal(hash(read(entry.path)), entry.sha256, entry.path);
    const vue = path.dirname(createRequire(import.meta.url).resolve("vue/package.json"));
    fs.mkdirSync(path.join(workspace, "node_modules"));
    const linkType = process.platform === "win32" ? "junction" : "dir";
    fs.symlinkSync(vue, path.join(workspace, "node_modules/vue"), linkType);
    fs.symlinkSync(
      path.join(path.dirname(vue), "@vue"),
      path.join(workspace, "node_modules/@vue"),
      linkType,
    );
    createMarker(workspace);
    fs.writeFileSync(path.join(workspace, "Page.vue"), original);
    fs.writeFileSync(path.join(workspace, "messages.ts"), messages);
    fs.writeFileSync(path.join(workspace, "tsconfig.json"), tsconfig);
    const uri = pathToFileURL(path.join(workspace, "Page.vue")).href;
    const runtime = resolveTypecheckRuntime(root);
    assert.ok(runtime, "the actual native runtime is mandatory without skips");
    const resolvedRuntime = fs.realpathSync(runtime);
    const runtimeHash = hash(fs.readFileSync(resolvedRuntime));
    const runtimeConfig = { typeChecker: { corsaPath: resolvedRuntime } };
    fs.writeFileSync(
      path.join(workspace, "vize.config.json"),
      `${JSON.stringify(runtimeConfig)}\n`,
    );
    record({
      kind: "bootstrap",
      vue: file(vue, "package.json"),
      runtime: { path: resolvedRuntime, sha256: runtimeHash },
      runtimeConfig,
      uri,
    });
    session = new LspSession({ repoRoot: root, binary: path.join(root, "target/ci/vize") });
    session.responseObservers.push((message) => record({ kind: "response", message }));
    session.notificationObservers.push((method, params) =>
      record({ kind: "notification", method, params }),
    );
    let active = session;
    const request = async (
      method: string,
      params: unknown,
      expected: unknown,
    ): Promise<unknown> => {
      record({ kind: "request", method, params, expected });
      const started = process.hrtime.bigint();
      const actual = await active.request(method, params);
      record({
        kind: "whole-result",
        method,
        actual,
        elapsedMs: Number(process.hrtime.bigint() - started) / 1e6,
      });
      assert.deepEqual(actual, expected, output);
      return actual;
    };
    record({
      kind: "initialize",
      workspace,
      options: { editor: true, typecheck: true, lint: false },
    });
    const initialized = await active.initialize(workspace, {
      editor: true,
      typecheck: true,
      lint: false,
    });
    record({ kind: "whole-initialize", initialized });
    assert.equal(
      (initialized as { capabilities: { completionProvider: { resolveProvider: boolean } } })
        .capabilities.completionProvider.resolveProvider,
      true,
    );
    let oldItem: unknown;
    for (const current of sources) {
      if (current.name === "reopened-client-version-one") {
        record({
          kind: "notify",
          method: "textDocument/didClose",
          params: { textDocument: { uri } },
        });
        active.notify("textDocument/didClose", { textDocument: { uri } });
        await request("completionItem/resolve", oldItem, oldItem);
      }
      const method = current.version === 1 ? "textDocument/didOpen" : "textDocument/didChange";
      const params =
        current.version === 1
          ? {
              textDocument: {
                uri,
                languageId: "vue",
                version: current.version,
                text: current.source,
              },
            }
          : {
              textDocument: { uri, version: current.version },
              contentChanges: [{ text: current.source }],
            };
      record({ kind: "notify", method, params });
      active.notify(method, params);
      const publication = await active.waitForNotification(
        "textDocument/publishDiagnostics",
        (value) => {
          const row = value as { uri: string; version: number };
          return row.uri === uri && row.version === current.version;
        },
      );
      record({ kind: "whole-publication", current: current.name, publication });
      assert.deepEqual(publication, { uri, version: current.version, diagnostics: [] });
      if (oldItem) await request("completionItem/resolve", oldItem, oldItem);
      for (const [domain, label, reported] of [
        ["script", "form.name", { line: 3, character: 17 }],
        ["template", "form.help", { line: 8, character: 11 }],
      ] as const) {
        const token = `${current.quote}${label}${current.quote}`;
        const position = offsetToPosition(current.source, current.source.indexOf(token) + 1);
        if (current.name === "original-LF") assert.deepEqual(position, reported);
        const owned = witness(workspace, current.source, messages, tsconfig, token, record);
        const expected = substitute(nativeOracle, {
          $AUTHORED_URI: uri,
          $REVISION: current.revision,
          $NATIVE_URI: owned.requestUri,
          $NATIVE_FILE: owned.fileName,
          $NATIVE_POSITION: owned.position,
          $NATIVE_RANGE: owned.range,
        }) as Record<string, unknown>[];
        record({ kind: "domain", current: current.name, domain, position, owned });
        const result = (await request(
          "textDocument/completion",
          {
            textDocument: { uri },
            position,
            context: { triggerKind: 1 },
          },
          expected,
        )) as Record<string, unknown>[];
        for (const item of expected)
          await request("completionItem/resolve", item, {
            ...item,
            detail: item.label,
            documentation: {
              kind: "markdown",
              value: `\`\`\`typescript\n${String(item.label)}\n\`\`\``,
            },
          });
        if (current.name === "original-LF" && domain === "script") oldItem = result[0];
      }
    }
    await active.shutdown();
    record({ kind: "shutdown", stderr: active.stderrText });
    session = undefined;
    const terminalRuntimeHash = hash(fs.readFileSync(resolvedRuntime));
    record({ kind: "terminal-runtime", path: resolvedRuntime, sha256: terminalRuntimeHash });
    assert.equal(terminalRuntimeHash, runtimeHash);
    // A separate genuine project session proves unconstrained strings remain
    // empty; a missing union is never replaced with Vue APIs or bindings.
    record({ kind: "plain-string-control", source: original, messages: plainMessages });
    fs.writeFileSync(path.join(workspace, "messages.ts"), plainMessages);
    session = new LspSession({ repoRoot: root, binary: path.join(root, "target/ci/vize") });
    active = session;
    active.responseObservers.push((message) => record({ kind: "response", message }));
    active.notificationObservers.push((method, params) =>
      record({ kind: "notification", method, params }),
    );
    record({
      kind: "initialize",
      workspace,
      options: { editor: true, typecheck: true, lint: false },
    });
    record({
      kind: "whole-initialize",
      initialized: await active.initialize(workspace, {
        editor: true,
        typecheck: true,
        lint: false,
      }),
    });
    const opened = { textDocument: { uri, languageId: "vue", version: 1, text: original } };
    record({ kind: "notify", method: "textDocument/didOpen", params: opened });
    active.notify("textDocument/didOpen", opened);
    const plainPublication = await active.waitForNotification(
      "textDocument/publishDiagnostics",
      (value) => {
        const row = value as { uri: string; version: number };
        return row.uri === uri && row.version === 1;
      },
    );
    record({
      kind: "whole-publication",
      current: "plain-string-control",
      publication: plainPublication,
    });
    assert.deepEqual(plainPublication, { uri, version: 1, diagnostics: [] });
    for (const label of ["form.name", "form.help"]) {
      const token = `"${label}"`;
      witness(workspace, original, plainMessages, tsconfig, token, record);
      await request(
        "textDocument/completion",
        {
          textDocument: { uri },
          position: offsetToPosition(original, original.indexOf(token) + 1),
          context: { triggerKind: 1 },
        },
        null,
      );
    }
    await active.shutdown();
    record({ kind: "shutdown", stderr: active.stderrText });
    session = undefined;
    record({
      kind: "final-runtime",
      path: resolvedRuntime,
      sha256: hash(fs.readFileSync(resolvedRuntime)),
    });
    assert.equal(hash(fs.readFileSync(resolvedRuntime)), runtimeHash);
    assert.equal(
      rows.filter((row) => (row as { kind: string }).kind === "whole-result").length,
      37,
    );
    capture.complete = true;
    record({ kind: "complete" });
  } catch (error) {
    record({
      kind: "failure",
      error: error instanceof Error ? error.stack : String(error),
      stderr: session?.stderrText ?? null,
    });
    throw error;
  } finally {
    if (session) {
      await session
        .shutdown()
        .catch((error: unknown) => record({ kind: "cleanup-error", error: String(error) }));
      record({ kind: "cleanup", stderr: session.stderrText });
    }
  }
});

function file(directory: string, name: string): unknown {
  const absolute = path.join(directory, name);
  const source = fs.readFileSync(absolute, "utf8");
  return { path: absolute, source, sha256: hash(source) };
}
