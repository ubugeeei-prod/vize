/** Publish the canonical listener argument relations without widening their inputs. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";
import { format } from "oxfmt";

const names = ["$ListenerMissing", "$ListenerArgs"];
const namespace = "__vizeListenerTypes";
const printer = ts.createPrinter({ removeComments: true });
function aliases(text: string, label: string) {
  const source = ts.createSourceFile(label, text, ts.ScriptTarget.Latest, true) as ts.SourceFile & {
    parseDiagnostics: readonly ts.Diagnostic[];
  };
  assert.deepEqual(source.parseDiagnostics, [], `${label} parses`);
  const found: ts.TypeAliasDeclaration[] = [];
  function visit(node: ts.Node) {
    if (ts.isTypeAliasDeclaration(node) && names.includes(node.name.text)) found.push(node);
    ts.forEachChild(node, visit);
  }
  visit(source);
  assert.deepEqual(
    found.map((node) => node.name.text),
    names,
    `${label} exact family`,
  );
  return { source, found };
}
function identity(node: ts.TypeAliasDeclaration, source: ts.SourceFile) {
  return printer.printNode(ts.EmitHint.Unspecified, node, source).replace(/^export\s+/u, "");
}
export async function listenerArgumentExports(
  root: string,
  pending: ReadonlyMap<string, string>,
): Promise<Map<string, string>> {
  const read = (file: string) => pending.get(file) ?? readFileSync(path.join(root, file), "utf8");
  const base = "crates/vize_canon/src/virtual_ts/helpers/declarations/";
  const embedded = aliases(read(base + "listener-args.d.ts.txt"), "embedded listeners");
  const text = read(base + "shared-listener-args.d.ts.txt");
  const shared = aliases(text, "shared listeners");
  for (const [index, name] of names.entries()) {
    assert.equal(
      identity(embedded.found[index]!, embedded.source),
      identity(shared.found[index]!, shared.source),
      `${name} policy`,
    );
    assert.equal(
      embedded.found[index]!.modifiers?.some((node) => node.kind === ts.SyntaxKind.ExportKeyword) ??
        false,
      false,
    );
    assert.equal(
      shared.found[index]!.modifiers?.some((node) => node.kind === ts.SyntaxKind.ExportKeyword) ??
        false,
      index === 1,
    );
  }
  const body = text.slice(text.indexOf("{") + 1, text.lastIndexOf("}"));
  const formatted = await format(
    "virtual-ts-listeners.d.ts",
    "// Generated from the canonical listener argument relations. Do not edit.\n" + body,
    { printWidth: 100 },
  );
  assert.deepEqual(formatted.errors, []);
  const outputs = new Map([["npm/cli/virtual-ts-listeners.d.ts", formatted.code]]);
  // Every exported shared relation is forwarded under its own name.
  const exported: ts.TypeAliasDeclaration[] = [];
  shared.source.forEachChild(function visit(node) {
    if (
      ts.isTypeAliasDeclaration(node) &&
      node.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword)
    )
      exported.push(node);
    ts.forEachChild(node, visit);
  });
  const forward =
    `import type * as ${namespace} from "./virtual-ts-listeners.js" with { "resolution-mode": "import" };\n` +
    exported
      .map((helper) => {
        const parameters = helper.typeParameters!.map((node) => node.getText(shared.source));
        const arguments_ = helper.typeParameters!.map((node) => node.name.text);
        return `export type ${helper.name.text}<${parameters.join(", ")}> = ${namespace}.${helper.name.text}<${arguments_.join(", ")}>;\n`;
      })
      .join("");
  const forwarded = exported.map((helper) => helper.name.text);
  const forwarding = await format("virtual-ts.d.ts", forward, { printWidth: 100 });
  assert.deepEqual(forwarding.errors, []);
  for (const extension of ["ts", "mts", "cts"]) {
    const file = `npm/cli/virtual-ts.d.${extension}`;
    const source = read(file);
    const parsed = ts.createSourceFile(
      file,
      source,
      ts.ScriptTarget.Latest,
      true,
    ) as ts.SourceFile & { parseDiagnostics: readonly ts.Diagnostic[] };
    assert.deepEqual(parsed.parseDiagnostics, []);
    assert.equal(parsed.statements.some(ts.isExportDeclaration), false, `${file} implicit exports`);
    const bindings = parsed.statements.filter(
      (node) => ts.isTypeAliasDeclaration(node) && forwarded.includes(node.name.text),
    );
    const imports = parsed.statements.filter(
      (node) =>
        ts.isImportDeclaration(node) &&
        ts.isStringLiteral(node.moduleSpecifier) &&
        node.moduleSpecifier.text === "./virtual-ts-listeners.js",
    );
    if (bindings.length === 0 && imports.length === 0) {
      assert.equal(source.includes(namespace), false, `${file} namespace collision`);
      outputs.set(file, source.trimEnd() + "\n\n" + forwarding.code);
    } else {
      assert.equal(imports.length, 1, `${file} listener import count`);
      const start = imports[0]!.getStart(parsed);
      const block = source.slice(start, bindings.at(-1)!.end).trimEnd() + "\n";
      // Only the current forwarding, or the `$ListenerArgs`-only one it extends.
      assert.ok(
        block === forwarding.code || forwarding.code.startsWith(block),
        `${file} refuses unknown listener forwarding`,
      );
      assert.equal(source.slice(start + block.length).trim(), "", `${file} forwarding last`);
      outputs.set(file, source.slice(0, start) + forwarding.code);
    }
  }
  return outputs;
}
