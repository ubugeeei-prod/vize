/** Synchronize only the source-owned slot payload family, refusing unknown exports. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";
import { format } from "oxfmt";

const names = ["$SilentAny", "$RegisteredSlotProvider", "$NamedSlotPayload", "$SlotPayload"];
const printer = ts.createPrinter({ removeComments: true });
function aliases(text: string, label: string) {
  const source = ts.createSourceFile(label, text, ts.ScriptTarget.Latest, true);
  const found: ts.TypeAliasDeclaration[] = [];
  function visit(node: ts.Node) {
    if (ts.isTypeAliasDeclaration(node) && names.includes(node.name.text)) found.push(node);
    ts.forEachChild(node, visit);
  }
  visit(source);
  assert.equal(
    new Set(found.map((node) => node.name.text)).size,
    found.length,
    `${label}: duplicates`,
  );
  return { source, found };
}
function identity(node: ts.TypeAliasDeclaration, source: ts.SourceFile) {
  const declaration = node.getText(source).replace(/^export\s+/u, "");
  // Only private type parameter identifiers differ between the two canonical assets.
  const transformed = ts.transform(
    ts.createSourceFile("alias.ts", declaration, ts.ScriptTarget.Latest, true),
    [
      (context) => {
        const visit: ts.Visitor = (node) =>
          ts.isParenthesizedTypeNode(node)
            ? ts.visitNode(node.type, visit)
            : ts.isIdentifier(node) && /^__(?:G|K|F|L|S|N|C|P)$/u.test(node.text)
              ? context.factory.createIdentifier(node.text.slice(2))
              : ts.visitEachChild(node, visit, context);
        return (file) => ts.visitNode(file, visit) as ts.SourceFile;
      },
    ],
  );
  try {
    return printer.printFile(transformed.transformed[0]!);
  } finally {
    transformed.dispose();
  }
}
export async function slotPayloadExports(root: string): Promise<Map<string, string>> {
  const base = "crates/vize_canon/src/virtual_ts/helpers/declarations/";
  const embedded = aliases(
    readFileSync(path.join(root, base, "slot-payload.d.ts.txt"), "utf8"),
    "embedded payload",
  );
  const sharedText = readFileSync(path.join(root, base, "shared-slot-payload.d.ts.txt"), "utf8");
  const shared = aliases(sharedText, "shared payload");
  for (const family of [embedded, shared]) {
    assert.deepEqual(
      family.found.map((node) => node.name.text),
      names,
      "complete canonical payload family",
    );
  }
  for (let index = 0; index < names.length; index++) {
    assert.equal(
      identity(embedded.found[index]!, embedded.source),
      identity(shared.found[index]!, shared.source),
      `${names[index]} canonical policies drifted`,
    );
    for (const [family, exported] of [
      [embedded, false],
      [shared, index === 1 || index === 3],
    ] as const) {
      assert.equal(
        family.found[index]!.modifiers?.some((node) => node.kind === ts.SyntaxKind.ExportKeyword) ??
          false,
        exported,
        `${names[index]} canonical visibility`,
      );
    }
  }
  const legacy = aliases(
    `
    type $SilentAny = {}["__vizeSilentAny"];
    type $NamedSlotPayload<S, N> = { [K in keyof S & N]-?: NonNullable<S[K]> extends (props: infer P, ...args: any[]) => any ? P : never }[keyof S & N] extends infer P ? ([P] extends [never] ? any : P) : any;
    export type $SlotPayload<S, N> = $IsAny<S> extends true ? $SilentAny : string extends N ? string extends keyof S ? $NamedSlotPayload<S, N> : any : $NamedSlotPayload<S, N>;
  `,
    "previous public payload",
  );
  // The registered-provider payload before it distributed over a union host: it
  // inferred one parameter from the union of the members' slot functions, which
  // intersects their payloads.
  const intersecting = aliases(
    `
    type $SilentAny = {}["__vizeSilentAny"];
    export type $RegisteredSlotProvider<G, K extends string | number | symbol, F extends string | number | symbol = K, L extends string | number | symbol = F> = $IsAny<G> extends true ? $SilentAny : K extends keyof G ? G[K] : F extends keyof G ? G[F] : L extends keyof G ? G[L] : $SilentAny;
    type $NamedSlotPayload<S, N> = { [K in keyof S & N]-?: NonNullable<S[K]> extends (props: infer P, ...args: any[]) => any ? P : never }[keyof S & N] extends infer P ? ([P] extends [never] ? any : P) : any;
    export type $SlotPayload<S, N, C = $RegisteredSlotProvider<any, never>> = $IsAny<S> extends true ? $IsAny<C> extends true ? C : any : string extends N ? string extends keyof S ? $NamedSlotPayload<S, N> : any : $NamedSlotPayload<S, N>;
  `,
    "intersecting public payload",
  );
  const sameFamily = (
    current: ReturnType<typeof aliases>,
    family: ReturnType<typeof aliases>,
  ): boolean =>
    current.found.length === family.found.length &&
    current.found.every(
      (node, index) =>
        node.name.text === family.found[index]!.name.text &&
        identity(node, current.source) === identity(family.found[index]!, family.source),
    );
  const body = sharedText.slice(sharedText.indexOf("{") + 1, sharedText.lastIndexOf("}"));
  const formatted = await format("virtual-ts.d.ts", body, { printWidth: 100 });
  assert.deepEqual(formatted.errors, [], "source-owned payload exports must parse");
  const output = new Map<string, string>();
  for (const extension of ["ts", "mts", "cts"]) {
    const relative = `npm/cli/virtual-ts.d.${extension}`;
    const text = readFileSync(path.join(root, relative), "utf8");
    const current = aliases(text, relative);
    // A known earlier policy is rewritten to the source's; anything else is refused.
    const expected =
      [legacy, intersecting, shared].find((family) => sameFamily(current, family)) ??
      (current.found.length === legacy.found.length ? legacy : shared);
    assert.deepEqual(
      current.found.map((node) => node.name.text),
      expected.found.map((node) => node.name.text),
      `${relative}: complete known payload family`,
    );
    for (let index = 0; index < current.found.length; index++) {
      const node = current.found[index]!;
      // Layout first: a foreign alias inside the family is refused as such, not
      // as whichever policy it happens to break.
      if (index > 0)
        assert.equal(
          current.source.statements.indexOf(node),
          current.source.statements.indexOf(current.found[index - 1]!) + 1,
          `${relative}: payload aliases must remain adjacent`,
        );
      assert.equal(
        identity(node, current.source),
        identity(expected.found[index]!, expected.source),
        `${relative}: unknown payload policy`,
      );
      assert.equal(
        node.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword) ?? false,
        node.name.text === "$RegisteredSlotProvider" || node.name.text === "$SlotPayload",
        `${relative}: payload visibility`,
      );
    }
    const first = current.found[0]!,
      last = current.found.at(-1)!;
    output.set(
      relative,
      text.slice(0, first.getFullStart()) +
        "\n\n" +
        formatted.code.trimEnd() +
        text.slice(last.end),
    );
  }
  return output;
}
