import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { offsetToPosition } from "./support/lsp/assertions.ts";
import { resolveTypecheckRuntime } from "./support/typecheck-dependency.ts";

type Item = { label: string; insertText?: string; textEdit?: unknown; [key: string]: unknown };
const corpus = path.join(root, "tests/_fixtures/differential/lsp/component-global-attributes-8015");
const read = (name: string): string => fs.readFileSync(path.join(corpus, name), "utf8");
const hash = (value: string | Buffer): string => createHash("sha256").update(value).digest("hex");
const original = read("Comp.vue.txt");
const common = JSON.parse(read("common.expected.json")) as Item[];
const manifest = JSON.parse(read("source.json")) as {
  files: { path: string; bytes: number; sha256: string }[];
  queries: unknown[];
  scope: string;
};
const prefixes: Record<string, string> = {
  id: "id",
  class: "cl",
  style: "st",
  title: "ti",
  role: "ro",
  tabindex: "tab",
  "aria-label": "aria-",
  ref: "re",
  key: "ke",
};

await test("component tags keep complete common attributes through real stdio and native options", async () => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "component-global-attributes-"));
  const captureRoot = path.join(
    root,
    "target/differential/component-global-attributes",
    path.basename(workspace),
  );
  fs.mkdirSync(captureRoot, { recursive: true });
  const output = path.join(captureRoot, "whole-observations.json");
  const rows: unknown[] = [];
  const capture = {
    schema: "vize.lsp.component-global-attributes.v1",
    complete: false,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    scope: manifest.scope,
    originalQueries: manifest.queries,
    inputs: Object.fromEntries(manifest.files.map((x) => [x.path, read(x.path)])),
    rows,
  };
  const record = (row: unknown): void => {
    rows.push(row);
    fs.writeFileSync(output, `${JSON.stringify(capture, null, 2)}\n`);
  };
  record({ kind: "start", workspace });
  let session: LspSession | undefined;
  try {
    for (const entry of manifest.files) {
      const raw = fs.readFileSync(path.join(corpus, entry.path));
      assert.equal(raw.length, entry.bytes);
      assert.equal(hash(raw), entry.sha256, entry.path);
    }
    fs.mkdirSync(path.join(workspace, "src"));
    for (const name of ["Child", "Comp", "List", "App"])
      fs.writeFileSync(path.join(workspace, "src", `${name}.vue`), read(`${name}.vue.txt`));
    fs.writeFileSync(path.join(workspace, "tsconfig.json"), read("tsconfig.json"));
    const vue = path.dirname(createRequire(import.meta.url).resolve("vue/package.json"));
    fs.mkdirSync(path.join(workspace, "node_modules"));
    const linkType = process.platform === "win32" ? "junction" : "dir";
    fs.symlinkSync(vue, path.join(workspace, "node_modules/vue"), linkType);
    fs.symlinkSync(
      path.join(path.dirname(vue), "@vue"),
      path.join(workspace, "node_modules/@vue"),
      linkType,
    );
    const runtime = resolveTypecheckRuntime(root);
    assert.ok(runtime, "actual pinned native runtime is mandatory without skip");
    const physicalRuntime = fs.realpathSync(runtime);
    const config = { typeChecker: { corsaPath: physicalRuntime } };
    fs.writeFileSync(path.join(workspace, "vize.config.json"), `${JSON.stringify(config)}\n`);
    record({
      kind: "bootstrap",
      config,
      tsconfig: read("tsconfig.json"),
      vue: { path: vue, packageJson: fs.readFileSync(path.join(vue, "package.json"), "utf8") },
      native: { path: physicalRuntime, sha256: hash(fs.readFileSync(physicalRuntime)) },
    });
    const uri = pathToFileURL(path.join(workspace, "src/Comp.vue")).href;
    for (const native of [false, true]) {
      session = new LspSession({ repoRoot: root, binary: path.join(root, "target/ci/vize") });
      const active = session;
      active.responseObservers.push((message) => record({ kind: "response", native, message }));
      active.notificationObservers.push((method, params) =>
        record({ kind: "notification", native, method, params }),
      );
      active.stderrObservers.push((stderr) => record({ kind: "stderr", native, stderr }));
      const request = async (
        method: string,
        params: unknown,
        expected: unknown,
      ): Promise<unknown> => {
        record({ kind: "request", native, method, params, expected });
        let actual: unknown;
        try {
          actual = await active.request(method, params);
        } catch (error) {
          record({ kind: "request-error", native, method, error: String(error) });
          throw error;
        }
        record({ kind: "whole-result", native, method, actual });
        assert.deepEqual(actual, expected, output);
        return actual;
      };
      record({
        kind: "initialize",
        native,
        workspace,
        options: { editor: true, lint: false, typecheck: native },
      });
      const initialized = await active.initialize(workspace, {
        editor: true,
        lint: false,
        typecheck: native,
      });
      record({ kind: "initialized-result", native, initialized });
      active.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: original },
      });
      let version = 1;
      const change = (source: string): void => {
        const params = {
          textDocument: { uri, version: ++version },
          contentChanges: [{ text: source }],
        };
        record({ kind: "change", native, params });
        active.notify("textDocument/didChange", params);
      };
      const complete = async (source: string, name: string, prefix: string): Promise<Item> => {
        const start = source.indexOf(`${name}="wide"`);
        assert.ok(start >= 0);
        const cursor = start + prefix.length;
        const position = offsetToPosition(source, cursor);
        const expected: Item = structuredClone(common.find((item) => item.label === name)!);
        const newText = expected.insertText;
        delete expected.insertText;
        expected.textEdit = {
          range: { start: offsetToPosition(source, start), end: position },
          newText,
        };
        if (source === original && name === "class")
          assert.deepEqual(position, { line: 7, character: 11 });
        const actual = await request(
          "textDocument/completion",
          { textDocument: { uri }, position },
          [expected],
        );
        assert.ok(Array.isArray(actual));
        return expected;
      };
      const first = await complete(original, "class", "cl");
      for (const item of common) {
        if (item.label === "class") continue;
        const changed = original.replace('class="wide"', `${item.label}="wide"`);
        change(changed);
        await complete(changed, item.label, prefixes[item.label]!);
      }
      for (const [source, marker] of [
        [original.replace('class="wide"', 'type="wide"'), 'type="'],
        [original, 'wide"'],
      ]) {
        change(source!);
        const offset = source!.indexOf(marker!) + (marker === 'wide"' ? 2 : 4);
        await request(
          "textDocument/completion",
          { textDocument: { uri }, position: offsetToPosition(source!, offset) },
          native && marker === 'type="' ? [] : null,
        );
      }
      const dirty = read("Comp-dirty-crlf.vue.txt");
      change(dirty);
      await complete(dirty, "class", "cl");
      change(original);
      await complete(original, "class", "cl");
      await request("completionItem/resolve", first, first);
      record({ kind: "close", native, uri });
      active.notify("textDocument/didClose", { textDocument: { uri } });
      await request("completionItem/resolve", first, first);
      const shutdown = await active.shutdown();
      record({ kind: "shutdown", native, shutdown });
      session = undefined;
    }
    assert.equal(
      rows.filter((row) => (row as { kind?: string }).kind === "whole-result").length,
      30,
    );
    capture.complete = true;
  } finally {
    if (session) {
      try {
        const shutdown = await session.shutdown();
        record({ kind: "failure-shutdown", shutdown });
      } catch (error) {
        record({ kind: "failure-shutdown-error", error: String(error) });
      }
    }
    record({ kind: "finish", complete: capture.complete });
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
