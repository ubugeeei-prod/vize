import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { resolveVuePackagePath } from "../../tools/support/compat/editor-e2e/real-vue-workspace.mjs";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { hoverToText, isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

type Row = {
  tag: string;
  role: string;
  position: { line: number; character: number };
  result: { contents: { kind: string; value: string }; range: unknown } | null;
};
const pack = path.join(root, "tests/_fixtures/differential/lsp");
const fixture = loadLspManifest(path.join(pack, "manifest.json")).cases.find(
  (row) => row.id === "lsp/regression/html-element-hover-documentation",
);
assert.ok(fixture);
const fixtureRoot = path.join(pack, fixture.inputs.root);
const provenance = JSON.parse(
  readPinnedArtifact(fixtureRoot, fixture.data.provenance.witness!).toString("utf8"),
);
function reference(name: string): Buffer {
  const row = provenance.references.find((item: { source: string }) => item.source === name);
  assert.ok(row, `frozen reference ${name}`);
  return readPinnedArtifact(fixtureRoot, { path: row.source, sha256: row.sha256 });
}
const rows = JSON.parse(reference("controls.expected.json").toString("utf8")) as Row[];
const source = fixture.files[0].bytes.toString("utf8");
const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
function configure(workspace: string, typecheck: boolean, corsaPath?: string): void {
  fs.writeFileSync(
    path.join(workspace, "vize.config.json"),
    JSON.stringify({
      lsp: { hover: true, lint: false, typecheck },
      ...(corsaPath ? { typeChecker: { corsaPath } } : {}),
    }),
  );
  fs.writeFileSync(
    path.join(workspace, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        lib: ["ES2022", "DOM", "DOM.Iterable"],
        strict: true,
        noEmit: true,
        target: "ES2022",
      },
      include: ["*.vue"],
    }),
  );
}
async function open(session: LspSession, uri: string, text: string): Promise<void> {
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text },
  });
  await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri),
    120_000,
  );
}
function summary(values: number[]): Record<string, number> {
  const sorted = [...values].sort((a, b) => a - b);
  return {
    count: values.length,
    p50Ms: sorted[Math.ceil(sorted.length * 0.5) - 1],
    p95Ms: sorted[Math.ceil(sorted.length * 0.95) - 1],
    maxMs: sorted.at(-1)!,
  };
}

await test("HTML fallback hovers keep whole contracts and observe bounded selected/unrelated request cost", async (t) => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-html-hover-"));
  const observations: Array<{ tag: string; role: string; elapsedMs: number; result: unknown }> = [];
  let session: LspSession | undefined;
  try {
    configure(workspace, false);
    session = new LspSession({ repoRoot: root, binary });
    await session.initialize(workspace, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: false,
    });
    for (const [name, text] of [
      ["App.vue", source],
      ["App.crlf.vue", reference("App.crlf.vue.txt").toString("utf8")],
    ]) {
      const file = path.join(workspace, name);
      fs.writeFileSync(file, text);
      const uri = pathToFileURL(file).href;
      await open(session, uri, text);
      for (const row of rows) {
        const start = performance.now();
        const result = await session.request("textDocument/hover", {
          textDocument: { uri },
          position: row.position,
        });
        observations.push({
          tag: row.tag,
          role: row.role,
          elapsedMs: performance.now() - start,
          result,
        });
        assert.deepEqual(result, row.result, `${name} ${row.tag} ${row.role}: whole Hover`);
      }
    }
    // Reuse the same ordinary source-built server and original packets. These
    // observations expose request cost, without claiming a before/after speedup.
    const uri = pathToFileURL(path.join(workspace, "App.vue")).href;
    const warm = rows.filter(
      (row) =>
        (row.role === "open" && ["button", "search", "h2"].includes(row.tag)) ||
        row.role === "unrelated",
    );
    for (let iteration = 0; iteration < 10; iteration += 1) {
      for (const row of warm) {
        const start = performance.now();
        const result = await session.request("textDocument/hover", {
          textDocument: { uri },
          position: row.position,
        });
        observations.push({
          tag: row.tag,
          role: row.role,
          elapsedMs: performance.now() - start,
          result,
        });
        assert.deepEqual(result, row.result, `warm ${row.tag} ${row.role}: complete response`);
      }
    }
    const latency = Object.fromEntries(
      [...new Set(observations.map((row) => row.tag))].map((tag) => [
        tag,
        summary(observations.filter((row) => row.tag === tag).map((row) => row.elapsedMs)),
      ]),
    );
    const output = path.join(root, "target/differential/html-element-hover.json");
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(
      output,
      `${JSON.stringify({ sourceAuthority: fixture.data.provenance.witness, binary, serverProcessId: session.processId, scope: "whole LF/CRLF responses and bounded ordinary-server latency; no A/B speedup claim", latency, observations }, null, 2)}\n`,
    );
    t.diagnostic(
      `HTML selected/unrelated hover latency: ${JSON.stringify(latency)}; whole responses: ${output}`,
    );
  } finally {
    await session?.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

await test("successful native Corsa HTML hovers include the selected documentation and original DOM type", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "native DOM hover provider",
    "native DOM hover runtime unavailable",
  );
  if (!corsaPath) return;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-html-native-hover-"));
  let session: LspSession | undefined;
  try {
    configure(workspace, true, corsaPath);
    fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}');
    const nodeModules = path.join(workspace, "node_modules");
    fs.mkdirSync(nodeModules);
    const vue = resolveVuePackagePath();
    fs.symlinkSync(vue, path.join(nodeModules, "vue"), "junction");
    const namespace = path.join(path.dirname(vue), "@vue");
    if (fs.existsSync(namespace))
      fs.symlinkSync(namespace, path.join(nodeModules, "@vue"), "junction");
    const file = path.join(workspace, "App.vue");
    const uri = pathToFileURL(file).href;
    fs.writeFileSync(file, source);
    session = new LspSession({ repoRoot: root, binary });
    await session.initialize(workspace, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });
    await open(session, uri, source);
    const types: Record<string, string> = {
      button: "HTMLButtonElement",
      h2: "HTMLHeadingElement",
      a: "HTMLAnchorElement",
      input: "HTMLInputElement",
      search: "HTMLElement",
      sub: "HTMLElement",
    };
    for (const row of rows.filter((row) => row.role === "open")) {
      const result = (await session.request(
        "textDocument/hover",
        { textDocument: { uri }, position: row.position },
        120_000,
      )) as { contents?: unknown } | null;
      const text = hoverToText(result);
      assert.match(
        text,
        new RegExp(`^(?:const|\\(const\\)) __vizeDomElement: ${types[row.tag]}$`, "m"),
        `native ${row.tag}: exact complete signature line`,
      );
      assert.ok(
        !text.includes("**Editor behavior**"),
        "the Corsa answer must not be the static fallback",
      );
      const expected = row.result!.contents.value;
      const description = expected.split("\n\n")[3];
      const links = expected.slice(expected.indexOf("\n\n**Docs**"));
      assert.ok(
        text.endsWith(`\n\n${description}${links}`),
        `complete native documentation suffix: ${row.tag}`,
      );
    }
  } finally {
    await session?.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
