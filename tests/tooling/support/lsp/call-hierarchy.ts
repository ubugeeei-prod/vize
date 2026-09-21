import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { isDiagnosticsForUri, offsetToPosition } from "./assertions.ts";
import { root, testOutputRoot } from "./paths.ts";
import type { LspRange } from "./protocol.ts";
import { LspSession } from "./session.ts";

export type Item = {
  name: string;
  uri: string;
  range: LspRange;
  selectionRange: LspRange;
  data?: unknown;
};
export type Outgoing = { to: Item; fromRanges: LspRange[] };
export type Incoming = { from: Item; fromRanges: LspRange[] };

export async function workspace(
  runtime: string,
  files: Record<string, string>,
  jsx: boolean,
  run: (session: LspSession, directory: string) => Promise<void>,
) {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const directory = fs.mkdtempSync(path.join(testOutputRoot, "call-hierarchy-project-"));
  for (const [name, source] of Object.entries(files))
    fs.writeFileSync(path.join(directory, name), source);
  fs.symlinkSync(
    path.join(root, "tests/node_modules"),
    path.join(directory, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
        jsx: "preserve",
        jsxImportSource: "vue",
        allowJs: true,
        checkJs: true,
        noEmit: true,
      },
      include: ["*.vue", "*.ts", "*.tsx", "*.jsx"],
    }),
  );
  fs.writeFileSync(
    path.join(directory, "vize.config.json"),
    JSON.stringify({
      lsp: { lint: false, typecheck: true },
      typeChecker: { corsaPath: runtime, jsxTypecheck: jsx },
    }),
  );
  const session = new LspSession();
  try {
    await session.initialize(directory, { editor: true, lint: false, typecheck: true });
    await run(session, directory);
  } finally {
    await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

export async function open(session: LspSession, uri: string, text: string, languageId = "vue") {
  session.notify("textDocument/didOpen", { textDocument: { uri, text, languageId, version: 1 } });
  await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri),
    60_000,
  );
}

export async function prepare(
  session: LspSession,
  uri: string,
  source: string,
  marker: string,
): Promise<Item> {
  const items = (await session.request("textDocument/prepareCallHierarchy", {
    textDocument: { uri },
    position: offsetToPosition(source, source.indexOf(marker) + 1),
  })) as Item[];
  assert.ok(Array.isArray(items), `missing hierarchy for ${marker}: ${JSON.stringify(items)}`);
  assert.equal(items.length, 1);
  return items[0];
}

export function span(source: string, marker: string, name: string, within = 0): LspRange {
  assert.ok(source.includes(marker), marker);
  const start = source.indexOf(marker) + within;
  return {
    start: offsetToPosition(source, start),
    end: offsetToPosition(source, start + name.length),
  };
}

export function sourceAt(source: string, range: LspRange): string {
  const offset = (position: { line: number; character: number }) =>
    source
      .split("\n")
      .slice(0, position.line)
      .reduce((sum, line) => sum + line.length + 1, 0) + position.character;
  return source.slice(offset(range.start), offset(range.end));
}
