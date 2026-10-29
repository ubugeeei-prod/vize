/** Parse every returned virtual source; these AST counts are observer work, not backend allocations. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isRecord } from "./canon-instance-directives-model.ts";
import { hash, save } from "./canon-instance-directives-io.ts";

type Row = {
  file: string;
  sourceBytes: number;
  nodes: number;
  identifiers: number;
  aliases: { name: string; bytes: number; sha256: string; uniqueSymbol: boolean }[];
};
export async function helperCensus(
  provider: string,
  files: { file: string; virtualTs?: string | null }[],
  out: string,
) {
  const imported: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript/lib/typescript.js")).href
  );
  assert.ok(isRecord(imported) && isRecord(imported.default));
  const compiler = imported.default;
  assert.ok(isRecord(compiler.SyntaxKind) && isRecord(compiler.ScriptTarget));
  const aliasKind = compiler.SyntaxKind.TypeAliasDeclaration;
  const identifierKind = compiler.SyntaxKind.Identifier;
  assert.equal(typeof aliasKind, "number");
  assert.equal(typeof identifierKind, "number");
  const invoke = (method: string, args: unknown[]): unknown => {
    const fn = compiler[method];
    assert.ok(typeof fn === "function");
    const value: unknown = Reflect.apply(fn, compiler, args);
    return value;
  };
  const rows: Row[] = [];
  for (const [index, file] of files.entries()) {
    assert.ok(
      typeof file.virtualTs === "string" && file.virtualTs.length > 0,
      "every CLI virtual source must be present",
    );
    const source = file.virtualTs;
    const row: Row = {
      file: file.file,
      sourceBytes: Buffer.byteLength(source),
      nodes: 0,
      identifiers: 0,
      aliases: [],
    };
    const root = invoke("createSourceFile", [
      file.file + ".ts",
      source,
      compiler.ScriptTarget.Latest,
      true,
    ]);
    const visit = (value: unknown): void => {
      assert.ok(isRecord(value) && typeof value.kind === "number");
      row.nodes++;
      if (value.kind === identifierKind) row.identifiers++;
      if (
        value.kind === aliasKind &&
        isRecord(value.name) &&
        typeof value.name.text === "string" &&
        value.name.text.startsWith("__Vize")
      ) {
        assert.ok(typeof value.end === "number");
        const getStart = value.getStart;
        assert.ok(typeof getStart === "function");
        const start = Reflect.apply(getStart, value, [root]);
        assert.ok(typeof start === "number");
        const declaration = source.slice(start, value.end);
        row.aliases.push({
          name: value.name.text,
          bytes: Buffer.byteLength(declaration),
          sha256: hash(declaration),
          uniqueSymbol: /unique\s+symbol/u.test(declaration),
        });
      }
      invoke("forEachChild", [value, visit]);
    };
    visit(root);
    fs.mkdirSync(path.join(out, "virtual"), { recursive: true });
    fs.writeFileSync(path.join(out, "virtual", String(index) + ".ts.txt"), source);
    rows.push(row);
  }
  const result = {
    scope:
      "TS6 observer parser over exact returned per-file virtual sources; AST nodes are not semantic symbols",
    rows,
    total: {
      virtualBytes: rows.reduce((sum, row) => sum + row.sourceBytes, 0),
      nodes: rows.reduce((sum, row) => sum + row.nodes, 0),
      identifiers: rows.reduce((sum, row) => sum + row.identifiers, 0),
      helperAliasBytes: rows.reduce(
        (sum, row) => sum + row.aliases.reduce((n, alias) => n + alias.bytes, 0),
        0,
      ),
    },
  };
  save(path.join(out, "helper-census.json"), result);
  return result;
}
