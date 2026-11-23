// Replay queries against the ordinary product with its native TypeScript session.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { runHistoryCase } from "../../../../tests/differential/lsp-history.ts";

type Request = { position: { line: number; character: number }; hover: unknown; project: boolean };
type Case = { source: string; previous?: string; requests: Request[] };
const [, , binary, capture] = process.argv;
const cases: Case[] = JSON.parse(readFileSync(capture, "utf8"));
const commit = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
const uri = "src/App.vue";
let matched = 0;
let deferred = 0;
for (const [index, input] of cases.entries()) {
  const { records } = await runHistoryCase(
    {
      version: 1,
      id: `checked-binding-${index}`,
      commit,
      requirement: "Return the complete typed hover at the original declaration",
      files: {
        [uri]: input.previous ?? input.source,
        "tsconfig.json": '{"compilerOptions":{"strict":true},"include":["src/**/*"]}',
      },
      initializationOptions: { editor: true, lint: false, typecheck: true },
      steps: [
        { open: uri },
        ...(input.previous === undefined
          ? []
          : [
              { diagnostics: uri },
              { change: uri, version: 2, contentChanges: [{ text: input.source }] },
            ]),
        { diagnostics: uri },
        ...input.requests.map(({ position }) => ({
          request: "textDocument/hover",
          uri,
          params: { position },
        })),
      ],
    },
    resolve(binary),
    { capture: true },
  );
  const preceding = input.previous === undefined ? 1 : 2;
  assert.equal(records.length, input.requests.length + preceding);
  for (const [i, query] of input.requests.entries()) {
    const wire = records[i + preceding];
    assert.ok("result" in wire && wire.result, JSON.stringify({ index, query, wire }));
    if (query.project) {
      assert.equal(query.hover, null);
      deferred++;
    } else {
      assert.deepEqual(
        query.hover,
        wire.result,
        JSON.stringify({ index, position: query.position }),
      );
      matched++;
    }
  }
}
assert.ok(matched > 0);
console.log(
  `Typed LSP hovers matched=${matched} deferred to project=${deferred} documents=${cases.length}`,
);
