// The L4 type-check target names ambient helpers but never declares them.
// Every name in its helper table must be declared by the published ambient set.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const targetDir = "levels/vize_l4/src/targets/ts";
const helpersFile = "levels/vize_l4/src/targets/ts/helpers.rs";
const globalsFile = "npm/cli/virtual-ts-globals.d.ts";

function rustFiles(dir: string): string[] {
  return fs.readdirSync(path.join(root, dir), { withFileTypes: true }).flatMap((entry) => {
    const relative = `${dir}/${entry.name}`;
    if (entry.isDirectory()) return rustFiles(relative);
    return entry.name.endsWith(".rs") ? [relative] : [];
  });
}

function helperTable(): string[] {
  const source = fs.readFileSync(path.join(root, helpersFile), "utf8");
  const constants = new Map(
    [...source.matchAll(/pub const ([A-Z_]+): &str = "([^"]+)";/g)].map(([, name, value]) => [
      name,
      value,
    ]),
  );
  const table = /pub const HELPERS: &\[&str\] = &\[([^\]]*)\];/.exec(source);
  assert.ok(table, "the helper table");
  return table[1]
    .split(",")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0)
    .map((entry) => {
      const value = constants.get(entry);
      assert.ok(value, `${entry} is a string constant`);
      return value;
    });
}

function ambientDeclarations(): Set<string> {
  const text = fs.readFileSync(path.join(root, globalsFile), "utf8");
  const file = ts.createSourceFile(globalsFile, text, ts.ScriptTarget.Latest, true);
  const names = new Set<string>();
  for (const statement of file.statements) {
    if (!ts.isModuleDeclaration(statement) || statement.name.getText(file) !== "global") continue;
    const body = statement.body;
    if (!body || !ts.isModuleBlock(body)) continue;
    for (const member of body.statements) {
      if (
        (ts.isFunctionDeclaration(member) ||
          ts.isTypeAliasDeclaration(member) ||
          ts.isInterfaceDeclaration(member)) &&
        member.name
      ) {
        names.add(member.name.text);
      }
    }
  }
  return names;
}

test("every helper the L4 TS target writes is declared by the published ambient set", () => {
  const helpers = helperTable();
  assert.deepEqual(helpers, ["__vizeNativeElementProp"]);
  const declared = ambientDeclarations();
  assert.deepEqual(
    helpers.filter((name) => !declared.has(name)),
    [],
  );
});

test("only the helper table spells helper names in the L4 TS target", () => {
  const spelled = rustFiles(targetDir).flatMap((file) =>
    [
      ...fs.readFileSync(path.join(root, file), "utf8").matchAll(/"((?:__vize|\$[A-Z])[\w$]*)/g),
    ].map(([, name]) => `${file}: ${name}`),
  );
  assert.deepEqual(
    spelled,
    helperTable().map((name) => `${helpersFile}: ${name}`),
  );
});
