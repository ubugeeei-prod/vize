import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const rustPath = path.join(root, "crates/vize_vitrine/src/wasm/options.rs");
const declarationPath = path.join(root, "npm/wasm/index.d.ts");

interface InventoryEntry {
  field: string;
  name: string;
  type: string;
}

function inventory(source: string): InventoryEntry[] {
  const block = source.slice(
    source.indexOf("define_compiler_option_inventory! {"),
    source.indexOf("pub(crate) struct ParsedCompilerOptions"),
  );
  return block
    .split("\n")
    .map((line) => line.match(/^\s*(\w+) => \("([^"]+)", (?:r#"(.+)"#|"([^"]+)")\),$/))
    .filter((match): match is RegExpMatchArray => match !== null)
    .map((match) => ({ field: match[1], name: match[2], type: match[3] ?? match[4] }));
}

function interfaceBody(source: string, name: string): string {
  const declaration = source.indexOf(`export interface ${name}`);
  assert.notEqual(declaration, -1, `${name} is not exported`);
  const start = source.indexOf("{", declaration);
  let depth = 0;
  for (let index = start; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(start + 1, index);
  }
  throw new Error(`${name} has no closing brace`);
}

function classBody(source: string, name: string): string {
  const declaration = source.indexOf(`export declare class ${name}`);
  assert.notEqual(declaration, -1, `${name} is not exported`);
  const start = source.indexOf("{", declaration);
  const end = source.indexOf("\n}", start);
  assert.notEqual(end, -1, `${name} has no closing brace`);
  return source.slice(start + 1, end);
}

function properties(body: string): Map<string, string> {
  const withoutComments = body.replace(/\/\*\*[\s\S]*?\*\//g, "");
  return new Map(
    [...withoutComments.matchAll(/^\s*(\w+)\?:\s*([^;]+);/gm)].map((match) => [
      match[1],
      match[2].replace(/\s+/g, " ").trim(),
    ]),
  );
}

test("WASM compiler option inventory matches parser and root declarations bidirectionally", () => {
  const rust = fs.readFileSync(rustPath, "utf8");
  const declarations = fs.readFileSync(declarationPath, "utf8");
  const entries = inventory(rust);
  assert.ok(entries.length > 0, "compiler option inventory is empty");

  const inventoryFields = new Set(entries.map(({ field }) => field));
  const inventoryNames = new Set(entries.map(({ name }) => name));
  assert.equal(inventoryFields.size, entries.length, "inventory field variants must be unique");
  assert.equal(inventoryNames.size, entries.length, "inventory JS names must be unique");

  const parser = rust.slice(
    rust.indexOf("pub(crate) fn parse_compiler_options"),
    rust.indexOf("/// Parse CSS options"),
  );
  const parsedFields = new Set(
    [...parser.matchAll(/CompilerOption::(\w+)/g)].map((match) => match[1]),
  );
  assert.doesNotMatch(parser, /JsValue::from_str\("/, "parser keys must use the inventory");
  assert.deepEqual(parsedFields, inventoryFields, "parser and inventory fields drifted");

  const declared = properties(interfaceBody(declarations, "CompilerOptions"));
  assert.deepEqual(
    new Set(declared.keys()),
    inventoryNames,
    "declaration and inventory keys drifted",
  );
  for (const entry of entries) {
    assert.equal(declared.get(entry.name), entry.type, `${entry.name} has the wrong public type`);
  }
});

test("every compiler facade uses the checked option declarations", () => {
  const declarations = fs.readFileSync(declarationPath, "utf8");
  const compiler = classBody(declarations, "Compiler");
  const facades: Array<[string, string, RegExp]> = [
    [
      "Compiler.compile",
      compiler,
      /compile\(template: string, options\?: CompilerOptions\): CompileResult;/,
    ],
    [
      "Compiler.compileVapor",
      compiler,
      /compileVapor\(template: string, options\?: CompilerOptions\): CompileResult;/,
    ],
    ["Compiler.parse", compiler, /parse\(template: string, options\?: CompilerOptions\): object;/],
    [
      "Compiler.compileSfc",
      compiler,
      /compileSfc\(source: string, options\?: SfcCompileOptions\): SfcCompileResult;/,
    ],
    [
      "compile",
      declarations,
      /export declare function compile\(template: string, options\?: CompilerOptions\): CompileResult;/,
    ],
    [
      "compileVapor",
      declarations,
      /export declare function compileVapor\(template: string, options\?: CompilerOptions\): CompileResult;/,
    ],
    [
      "parseTemplate",
      declarations,
      /export declare function parseTemplate\(template: string, options\?: CompilerOptions\): object;/,
    ],
    [
      "compileSfc",
      declarations,
      /export declare function compileSfc\(source: string, options\?: SfcCompileOptions\): SfcCompileResult;/,
    ],
  ];
  for (const [facade, source, declaration] of facades) {
    assert.match(source, declaration, `${facade} must use its checked option type`);
  }
  assert.match(declarations, /@deprecated Use `templateSyntax: "quirks"`/);
  assert.doesNotMatch(interfaceBody(declarations, "CompilerOptions"), /experimentalServerScript/);
});
