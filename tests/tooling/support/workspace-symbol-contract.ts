import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { root } from "./lsp/paths.ts";

export function assertOriginalSymbolSource(name: string, source: string) {
  const corpus = path.join(
    root,
    "tests/_fixtures/differential/lsp/project-navigation-8013/original-symbol-consumers",
  );
  assert.equal(source, fs.readFileSync(path.join(corpus, name + ".txt"), "utf8"));
}

/** Complete existing Vue script-setup symbol shape, including its zero range. */
export function scriptSetupSymbol(name: string, kind: number, uri: string, line: number) {
  return {
    name,
    kind,
    containerName: "script setup",
    location: {
      uri,
      range: {
        start: { line, character: 0 },
        end: { line, character: 0 },
      },
    },
  };
}
