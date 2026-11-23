import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import type { Node } from "yaml";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const actionScript = /^\. "\$GITHUB_ACTION_PATH\/scripts\/([\w.-]+\.(?:sh|ps1))"$/u;

/** Read the executable bodies for source-contract tests while retaining each step's metadata. */
export function readAutomationPrograms(...parts: (string | URL)[]): string {
  if (parts.at(-1) === "utf8") parts.pop();
  const file =
    parts[0] instanceof URL ? fileURLToPath(parts[0]) : path.resolve(root, ...(parts as string[]));
  const source = readFileSync(file, "utf8").replace(/\r\n?/gu, "\n");
  const { isMap, isScalar, isSeq, parseDocument } = createRequire(import.meta.url)(
    "yaml",
  ) as typeof import("yaml");
  const document = parseDocument(source);
  assert.deepEqual(document.errors, []);
  const edits = new Map<number, { end: number; value: string }>();

  function visit(node: Node | null): void {
    if (isMap(node)) {
      for (const pair of node.items) {
        const child = pair.value as Node | null;
        if (
          isScalar(pair.key) &&
          pair.key.value === "run" &&
          isScalar(child) &&
          typeof child.value === "string"
        ) {
          const match = actionScript.exec(child.value);
          if (!match) continue;
          assert.ok(child.range && pair.key.range);
          const script = path.join(path.dirname(file), "scripts", match[1]!);
          const body = readFileSync(script, "utf8")
            .replace(/\r\n?/gu, "\n")
            .replace(/^#![^\n]*\n/u, "")
            .trimEnd();
          const column = pair.key.range[0] - source.lastIndexOf("\n", pair.key.range[0]) - 1;
          const padding = " ".repeat(column + 2);
          const text = body
            .split("\n")
            .map((line) => (line ? padding + line : ""))
            .join("\n");
          // Flow mappings cannot contain YAML block scalars.
          const value = node.flow
            ? JSON.stringify(body + (body.includes("\n") ? "\n" : ""))
            : (body.includes("\n") ? "|\n" : "|-\n") + text;
          edits.set(child.range[0], { end: child.range[1], value });
        } else {
          visit(child);
        }
      }
    } else if (isSeq(node)) {
      for (const child of node.items) visit(child as Node | null);
    }
  }
  visit(document.contents);
  let result = source;
  for (const [start, { end, value }] of [...edits].sort(([a], [b]) => b - a)) {
    result = result.slice(0, start) + value + result.slice(end);
  }
  return result;
}

/** Locate the module the capture step passes to Node. */
export function actionNodeProgramPath(owner: string, command: string | undefined): string {
  const matches = [
    ...(command ?? "").matchAll(
      /\bnode "\$GITHUB_ACTION_PATH\/scripts\/([\w.-]+\.ts)" "\$capture"/gu,
    ),
  ];
  assert.equal(matches.length, 1, "the capture step must run one named Node module");
  return path.join(root, ".github/actions", owner, "scripts", matches[0]![1]!);
}
