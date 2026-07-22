import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";

const sourceDirectory = new URL(".", import.meta.url);

/**
 * Top-level exported declarations that must carry documentation. Re-export
 * statements (`export {…}`, `export * from`, `export type {…}`) are exempt:
 * they forward symbols documented at their declaration site.
 */
const EXPORT_DECLARATION =
  /^export\s+(?:abstract\s+)?(?:async\s+)?(?:function|const|let|var|class|interface|type|enum)\s+([A-Za-z_$][\w$]*)/;

/** Exported options-bag interfaces whose optional members need `@default`. */
const OPTIONS_INTERFACE = /^export interface [A-Za-z_$][\w$]*Options\b.*\{$/;

/** An optional property declaration inside an interface body. */
const OPTIONAL_PROPERTY = /^\s*(?:readonly\s+)?([\w$]+)\?:/;

function listSourceFiles(): readonly string[] {
  return readdirSync(sourceDirectory)
    .filter((name) => name.endsWith(".ts") && !name.endsWith(".test.ts"))
    .sort();
}

function readSourceLines(file: string): readonly string[] {
  return readFileSync(new URL(file, sourceDirectory), "utf8").split("\n");
}

/**
 * Return the JSDoc block whose closing marker sits directly above the given
 * line, or `undefined` when the declaration is undocumented or preceded only
 * by a plain (non-JSDoc) comment.
 */
function docBlockAbove(lines: readonly string[], declarationLine: number): string | undefined {
  let line = declarationLine - 1;
  const closing = lines[line]?.trim();
  if (closing === undefined || !closing.endsWith("*/")) return undefined;

  const collected: string[] = [];
  while (line >= 0) {
    const text = (lines[line] ?? "").trim();
    collected.unshift(text);
    if (text.includes("/**")) return collected.join("\n");
    if (text.includes("/*") || !text.startsWith("*")) return undefined;
    line -= 1;
  }
  return undefined;
}

void test("every exported declaration is directly preceded by a JSDoc block", () => {
  const problems: string[] = [];
  let declarations = 0;

  for (const file of listSourceFiles()) {
    const lines = readSourceLines(file);
    const seenNames = new Set<string>();

    for (const [index, lineText] of lines.entries()) {
      const match = EXPORT_DECLARATION.exec(lineText);
      if (!match) continue;
      const name = match[1] ?? "";
      // Later declarations of a seen name are overload signatures or the
      // overload implementation; the group is documented once, on its first
      // declaration, matching the package style.
      if (seenNames.has(name)) continue;
      seenNames.add(name);
      declarations += 1;

      if (docBlockAbove(lines, index) === undefined) {
        problems.push(`${file}:${index + 1} export "${name}" has no JSDoc block directly above`);
      }
    }
  }

  // Guard the scanner itself: if the regex rots, the suite must fail loudly
  // instead of silently checking nothing.
  assert.ok(declarations >= 20, `expected >= 20 exported declarations, found ${declarations}`);
  assert.deepEqual(problems, []);
});

void test("optional members of exported options interfaces document their @default", () => {
  const problems: string[] = [];
  let optionalMembers = 0;

  for (const file of listSourceFiles()) {
    const lines = readSourceLines(file);
    let insideOptionsInterface = false;

    for (const [index, lineText] of lines.entries()) {
      if (OPTIONS_INTERFACE.test(lineText)) {
        insideOptionsInterface = true;
        continue;
      }
      if (insideOptionsInterface && lineText === "}") {
        insideOptionsInterface = false;
        continue;
      }
      if (!insideOptionsInterface) continue;

      const property = OPTIONAL_PROPERTY.exec(lineText);
      if (!property) continue;
      optionalMembers += 1;

      const documentation = docBlockAbove(lines, index);
      if (documentation === undefined || !documentation.includes("@default")) {
        problems.push(
          `${file}:${index + 1} optional option "${property[1] ?? ""}" must document @default`,
        );
      }
    }
  }

  assert.ok(optionalMembers >= 15, `expected >= 15 optional options, found ${optionalMembers}`);
  assert.deepEqual(problems, []);
});
