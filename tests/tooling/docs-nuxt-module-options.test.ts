import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import ts from "typescript";

import { repoRoot } from "./_helpers/tool-commands.ts";

const optionsSource = path.join(repoRoot, "npm/framework/nuxt/src/options.ts");
const checkerSource = path.join(repoRoot, "npm/framework/nuxt/src/lint/checker/options.ts");
const lintSource = path.join(repoRoot, "npm/framework/nuxt/src/lint/options.ts");

/** Property names an exported interface declares itself, in source order. */
function interfaceMembers(file: string, name: string): string[] {
  const source = ts.createSourceFile(file, fs.readFileSync(file, "utf8"), ts.ScriptTarget.Latest);
  const declaration = source.statements.find(
    (statement): statement is ts.InterfaceDeclaration =>
      ts.isInterfaceDeclaration(statement) && statement.name.text === name,
  );
  assert.ok(declaration, `${path.relative(repoRoot, file)} must export ${name}`);
  return declaration.members.flatMap((member) =>
    ts.isPropertySignature(member) && ts.isIdentifier(member.name) ? [member.name.text] : [],
  );
}

/** First-column option names of every Markdown table row in a guide. */
function documentedOptions(locale: "" | "ja/"): Set<string> {
  const guide = fs.readFileSync(
    path.join(repoRoot, `docs/content/${locale}integrations/nuxt.md`),
    "utf8",
  );
  return new Set([...guide.matchAll(/^\| `([A-Za-z.]+)`\s+\|/gm)].map(([, option]) => option));
}

for (const [language, locale] of [
  ["English", ""],
  ["Japanese", "ja/"],
] as const) {
  test(`the ${language} Nuxt guide documents every module option`, () => {
    const documented = documentedOptions(locale);
    // `dev` only groups dev-server switches, so the guide lists its members.
    const expected = [
      ...interfaceMembers(optionsSource, "VizeNuxtOptions").filter((option) => option !== "dev"),
      ...interfaceMembers(optionsSource, "VizeNuxtDevOptions").map((option) => `dev.${option}`),
      ...interfaceMembers(lintSource, "VizeNuxtLintOptions"),
      ...interfaceMembers(checkerSource, "VizeNuxtLintCheckerOptions"),
    ];
    assert.ok(expected.length >= 20, "the module option interfaces were read");
    assert.deepEqual(
      expected.filter((option) => !documented.has(option)),
      [],
      "add a table row for each new @vizejs/nuxt option",
    );
  });
}
