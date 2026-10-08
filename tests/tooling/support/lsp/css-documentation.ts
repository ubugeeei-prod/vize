import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveVizeLaunchCommand } from "./launch.ts";
import { LspSession } from "./session.ts";

type Item = Record<string, unknown> & { label: string };
type Scenario = {
  name: string;
  css: string;
  completion: number;
  hover: number;
  label: string;
  fallback?: boolean;
};
export type CssObservation = {
  scenario: string;
  operation: string;
  documentationMode: "lazy" | "eager";
  samplesMs: number[];
  responseBytes: number;
  result: unknown;
};

function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

export function completionItems(value: unknown): Item[] {
  const rows: unknown = Array.isArray(value) ? value : record(value).items;
  assert.ok(Array.isArray(rows));
  return rows.map((row: unknown) => {
    const item = record(row);
    assert.equal(typeof item.label, "string");
    return item as Item;
  });
}

export function documentation(value: unknown): string {
  const doc = record(value);
  assert.equal(doc.kind, "markdown");
  assert.equal(typeof doc.value, "string");
  return doc.value as string;
}

function scenario(
  name: string,
  css: string,
  needle: string,
  label: string,
  hoverNeedle = label,
): Scenario {
  const start = css.lastIndexOf(needle);
  const hover = css.lastIndexOf(hoverNeedle);
  assert.ok(start >= 0 && hover >= 0);
  return {
    name,
    css,
    completion: start + needle.length,
    hover: hover + Math.min(2, hoverNeedle.length),
    label,
  };
}

function scenarios(): Scenario[] {
  const large = Array.from(
    { length: 700 },
    (_, i) => `.row-${i} { color: red; display: flex; }`,
  ).join("\n");
  return [
    scenario("property", ".demo { color: red; col }", "col", "color"),
    scenario("value", ".demo { display: flex; }", "display: fl", "flex"),
    scenario("color", ".demo { color: red; }", "color: re", "red"),
    scenario("pseudo", ".demo:hover { color: red; }", ":hov", ":hover"),
    scenario("at-rule", "@media (min-width: 600px) { .demo { color: red; } }", "@med", "@media"),
    scenario(
      "long-comment",
      `/*${"x".repeat(48_000)}*/\n.demo { color: red; col }`,
      "col",
      "color",
    ),
    scenario(
      "long-string",
      `.demo { content: "${"x".repeat(48_000)}"; color: red; col }`,
      "col",
      "color",
    ),
    scenario("large-style", `${large}\n.demo { color: red; col }`, "col", "color"),
    scenario("empty-prefix", ".demo { color: red;  }", "; ", "color"),
    {
      ...scenario("unterminated-comment", `/*${"x".repeat(48_000)} col`, "col", "v-bind", "col"),
      fallback: true,
    },
    {
      ...scenario(
        "unterminated-string",
        `.demo { content: "${"x".repeat(48_000)} col`,
        "col",
        "v-bind",
        "col",
      ),
      fallback: true,
    },
  ];
}

function position(source: string, offset: number) {
  const lines = source.slice(0, offset).split("\n");
  return { line: lines.length - 1, character: lines.at(-1)!.length };
}

async function timed(session: LspSession, method: string, params: unknown, samples: number) {
  const samplesMs: number[] = [];
  let result: unknown;
  for (let i = 0; i < samples + 5; i++) {
    const started = performance.now();
    result = await session.request(method, params);
    if (i >= 5) samplesMs.push(performance.now() - started);
  }
  return { samplesMs, result, responseBytes: Buffer.byteLength(JSON.stringify(result)) };
}

export async function cssSession(binary: string, sourceRequired: boolean, lazy: boolean) {
  const oldBinary = process.env.VIZE_LSP_BIN;
  const oldRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  let session: LspSession;
  try {
    process.env.VIZE_LSP_BIN = binary;
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = sourceRequired ? "1" : "0";
    const command = resolveVizeLaunchCommand(undefined, binary, { required: sourceRequired });
    assert.equal(command[0], binary, "CSS observation must use the requested executable");
    session = new LspSession();
  } finally {
    if (oldBinary === undefined) delete process.env.VIZE_LSP_BIN;
    else process.env.VIZE_LSP_BIN = oldBinary;
    if (oldRequired === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
    else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = oldRequired;
  }
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-docs-"));
  fs.writeFileSync(
    path.join(workspace, "vize.config.json"),
    JSON.stringify({ lsp: { completion: true, hover: true, typecheck: false, lint: false } }),
  );
  const uri = pathToFileURL(path.join(workspace, "App.vue")).href;
  try {
    await session.request("initialize", {
      processId: process.pid,
      rootUri: pathToFileURL(workspace).href,
      workspaceFolders: [{ uri: pathToFileURL(workspace).href, name: "CSS documentation" }],
      capabilities: {
        textDocument: {
          completion: {
            completionItem: {
              documentationFormat: ["markdown"],
              ...(lazy ? { resolveSupport: { properties: ["documentation"] } } : {}),
            },
          },
        },
      },
      initializationOptions: {
        editor: true,
        completion: true,
        hover: true,
        typecheck: false,
        lint: false,
      },
    });
    session.notify("initialized", {});
  } catch (error) {
    await session.kill();
    fs.rmSync(workspace, { recursive: true, force: true });
    throw error;
  }
  return {
    session,
    uri,
    open(source: string, version: number) {
      if (version === 1)
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version, text: source },
        });
      else
        session.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text: source }],
        });
    },
    async close() {
      try {
        await session.shutdown();
      } finally {
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    },
  };
}

/** Existing stdio session path; elapsed measurements are observations, never a speed gate. */
export async function observeCss(
  binary: string,
  sourceRequired: boolean,
  assertProduct: boolean,
  samples = 20,
  lazy = true,
): Promise<CssObservation[]> {
  const client = await cssSession(binary, sourceRequired, lazy);
  const observations: CssObservation[] = [];
  let version = 0;
  try {
    for (const row of scenarios()) {
      const prefix = "<template><div /></template>\n<style>";
      const source = `${prefix}${row.css}</style>`;
      client.open(source, ++version);
      const params = {
        textDocument: { uri: client.uri },
        position: position(source, prefix.length + row.completion),
      };
      const completion = await timed(client.session, "textDocument/completion", params, samples);
      const items = completionItems(completion.result);
      assert.deepEqual(
        items.slice(0, 4).map((item) => item.label),
        ["v-bind", ":deep", ":slotted", ":global"],
      );
      const item = items.find((item) => item.label === row.label) ?? items[0]!;
      if (assertProduct) {
        assert.equal(item.label, row.label);
        if (lazy) {
          assert.equal(item.documentation, undefined);
          assert.ok(record(item.data).vizeCss);
        } else {
          assert.ok(documentation(item.documentation).includes("**Docs**"));
          assert.equal(item.data, undefined);
        }
        if (row.fallback) assert.equal(items.length, 4);
        if (row.name === "empty-prefix") {
          assert.equal(items.length, 888 + 4);
          assert.ok(items.slice(4).every((entry) => entry.kind === 10));
        }
      }
      const resolved = lazy
        ? await timed(client.session, "completionItem/resolve", item, samples)
        : undefined;
      const hover = await timed(
        client.session,
        "textDocument/hover",
        { ...params, position: position(source, prefix.length + row.hover) },
        samples,
      );
      if (assertProduct) {
        let text = lazy ? "" : documentation(item.documentation);
        if (resolved) {
          const { documentation: resolvedDoc, ...remaining } = record(resolved.result);
          assert.deepEqual(remaining, item, "resolve preserves every insertion and identity field");
          text = documentation(resolvedDoc);
        }
        assert.ok(text.includes(`**${row.label}**`));
        assert.ok(text.includes("**Docs**"));
        if (row.fallback) assert.equal(hover.result, null);
        else {
          const hoverText = documentation(record(hover.result).contents);
          assert.ok(hoverText.includes(`**${row.label}**`));
          assert.ok(hoverText.includes("**Docs**"));
        }
      }
      for (const [operation, result] of [
        ["completion", completion],
        ...(resolved ? ([["resolve", resolved]] as const) : []),
        ["hover", hover],
      ] as const)
        observations.push({
          scenario: row.name,
          operation,
          documentationMode: lazy ? "lazy" : "eager",
          ...result,
        });
    }
    assert.ok(
      !client.session.stderrText.includes("Corsa initialization failed"),
      "CSS must not initialize a checker",
    );
    return observations;
  } finally {
    await client.close();
  }
}
