import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { repoRoot } from "../../_helpers/realworld-patch.ts";

export type Position = { line: number; character: number };
export type Range = { start: Position; end: Position };

export type DualLspFixture = {
  workspaceDir: string;
  uri: string;
  sourceV1: string;
  sourceV2: string;
  dispose(): void;
};

const fixtureRoot = path.join(repoRoot, "tests/_fixtures/performance/lsp-dual-server");

export function createDualLspFixture(): DualLspFixture {
  const workspaceDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-dual-"));
  const sourceV1 = fs.readFileSync(path.join(fixtureRoot, "sources/App.v1.vue.txt"), "utf8");
  const sourceV2 = fs.readFileSync(path.join(fixtureRoot, "sources/App.v2.vue.txt"), "utf8");
  fs.copyFileSync(path.join(fixtureRoot, "package.json"), path.join(workspaceDir, "package.json"));
  fs.copyFileSync(path.join(fixtureRoot, "tsconfig.json"), path.join(workspaceDir, "tsconfig.json"));
  fs.mkdirSync(path.join(workspaceDir, "src"));
  const appPath = path.join(workspaceDir, "src/App.vue");
  fs.writeFileSync(appPath, sourceV1, "utf8");
  const providerRoot = path.resolve(
    process.env.LSP_DUAL_PROVIDER_ROOT ?? path.join(fixtureRoot, "provider"),
  );
  const dependencyRoot = path.join(providerRoot, "node_modules");
  assert.ok(
    fs.existsSync(path.join(dependencyRoot, "vue/package.json")),
    "the isolated dual-server provider is not installed",
  );
  fs.mkdirSync(path.join(workspaceDir, "node_modules"));
  fs.symlinkSync(
    fs.realpathSync(path.join(dependencyRoot, "vue")),
    path.join(workspaceDir, "node_modules/vue"),
    "dir",
  );
  return {
    workspaceDir,
    uri: pathToFileURL(appPath).href,
    sourceV1,
    sourceV2,
    dispose: () => fs.rmSync(workspaceDir, { force: true, recursive: true }),
  };
}

export function offsetToPosition(source: string, offset: number): Position {
  assert.ok(offset >= 0 && offset <= source.length, `invalid offset ${offset}`);
  const before = source.slice(0, offset);
  const line = before.split("\n").length - 1;
  const lineStart = before.lastIndexOf("\n") + 1;
  return { line, character: offset - lineStart };
}

export function positionToOffset(source: string, position: Position): number {
  assert.ok(position.line >= 0 && position.character >= 0, `invalid position ${JSON.stringify(position)}`);
  let offset = 0;
  for (let line = 0; line < position.line; line += 1) {
    const newline = source.indexOf("\n", offset);
    assert.notEqual(newline, -1, `line ${position.line} is outside the document`);
    offset = newline + 1;
  }
  const result = offset + position.character;
  const lineEnd = source.indexOf("\n", offset);
  assert.ok(result <= (lineEnd < 0 ? source.length : lineEnd), `character is outside line ${position.line}`);
  return result;
}

export function wordRanges(source: string, word: string): Range[] {
  const ranges: Range[] = [];
  const pattern = new RegExp(`\\b${word}\\b`, "g");
  for (const match of source.matchAll(pattern)) {
    const start = match.index;
    ranges.push({
      start: offsetToPosition(source, start),
      end: offsetToPosition(source, start + word.length),
    });
  }
  return ranges;
}

export function rangeForNeedle(source: string, needle: string): Range {
  const offset = source.indexOf(needle);
  assert.notEqual(offset, -1, `missing ${JSON.stringify(needle)}`);
  return {
    start: offsetToPosition(source, offset),
    end: offsetToPosition(source, offset + needle.length),
  };
}

export function compareRange(left: Range, right: Range): number {
  return (
    left.start.line - right.start.line ||
    left.start.character - right.start.character ||
    left.end.line - right.end.line ||
    left.end.character - right.end.character
  );
}

export function rangeKey(range: Range): string {
  return `${range.start.line}:${range.start.character}-${range.end.line}:${range.end.character}`;
}
