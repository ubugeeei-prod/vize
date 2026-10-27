import assert from "node:assert/strict";
export type ExpectedError = { code: number; token: string; last?: boolean };
export type DirectiveCase = {
  id: string;
  source: string;
  errors: ExpectedError[];
  original?: boolean;
  local?: boolean;
  augmented?: boolean;
};
export function cases(original: string): DirectiveCase[] {
  const target = "\t\tx\n",
    directive = "\t\t// @ts-expect-error\n";
  assert.equal(original.split(target).length, 2);
  assert.equal(original.split(directive).length, 2);
  const unused: ExpectedError[] = [{ code: 2578, token: "// @ts-expect-error" }];
  const missing: ExpectedError[] = [{ code: 2339, token: "x\n", last: true }];
  const typed = original.replace(target, "\t\tpluginMethod(123)\n");
  const sibling = original.replace("\t}}\n", "\t}}\n\t{{ missingSibling }}\n");
  const rows: DirectiveCase[] = [
    { id: "original", source: original, errors: [], original: true },
    {
      id: "valid-local",
      source: original.replace("defineExpose({ x: 3 });", "const x: number = 3;"),
      local: true,
      errors: unused,
    },
    { id: "valid-augmented", source: original, augmented: true, errors: unused },
    { id: "wrong-typed-expect", source: typed, augmented: true, errors: [] },
    {
      id: "repaired-typed-expect",
      source: typed.replace("(123)", '("ok")'),
      augmented: true,
      errors: unused,
    },
    {
      id: "wrong-typed-no-directive",
      source: typed.replace(directive, ""),
      augmented: true,
      errors: [{ code: 2345, token: "123" }],
    },
    { id: "missing-no-directive", source: original.replace(directive, ""), errors: missing },
    { id: "sibling", source: sibling, errors: [{ code: 2339, token: "missingSibling" }] },
    { id: "ignore", source: original.replace("@ts-expect-error", "@ts-ignore"), errors: [] },
    {
      id: "ignore-sibling",
      source: sibling.replace("@ts-expect-error", "@ts-ignore"),
      errors: [{ code: 2339, token: "missingSibling" }],
    },
    {
      id: "unicode-crlf",
      source: (
        "<!-- 日本 🧭 -->\n" + sibling.replace("{{ missingSibling", "日本 🧭 {{ missingSibling")
      ).replaceAll("\n", "\r\n"),
      errors: [{ code: 2339, token: "missingSibling" }],
    },
    { id: "cr-only", source: original.replaceAll("\n", "\r"), errors: [] },
    {
      id: "unicode-line-separator",
      source: original.replace(directive, directive.replace("\n", "\u2028")),
      errors: [],
    },
    { id: "nbsp-bom", source: original.replace(target, "\u00a0\ufeff" + target), errors: [] },
    {
      id: "inline-block-consumed",
      source: original.replace(target, "\t\t/* 日本 🧭 */ x\n"),
      errors: [],
    },
    {
      id: "suffix-consumed",
      source: original.replace("@ts-expect-error", "@ts-expect-error-more"),
      errors: [],
    },
    {
      id: "suffix-repaired",
      source: original.replace("@ts-expect-error", "@ts-expect-error-more"),
      augmented: true,
      errors: unused,
    },
  ];
  const triviaRows: Array<[string, string]> = [
    ["blank", "\n"],
    ["comment", "\t\t// legitimate trivia\n"],
  ];
  for (const [id, trivia] of triviaRows) {
    const source = original.replace(directive, directive + trivia);
    rows.push({ id: id + "-consumed", source, errors: [] });
    rows.push({ id: id + "-repaired", source, augmented: true, errors: unused });
  }
  const firstBlock = original.replace(
    "// @ts-expect-error",
    "/* @ts-expect-error\n\t\t explanation */",
  );
  rows.push({ id: "first-line-block-bad", source: firstBlock, errors: missing });
  rows.push({ id: "first-line-block-repaired", source: firstBlock, augmented: true, errors: [] });
  const barrier = original.replace(directive, directive + "\t\t/* explanation */\n");
  rows.push({ id: "block-barrier-bad", source: barrier, errors: [...unused, ...missing] });
  rows.push({ id: "block-barrier-repaired", source: barrier, augmented: true, errors: unused });
  assert.equal(rows.length, 25);
  return rows;
}
export function referenceSource(row: DirectiveCase): string {
  const expressions = [...row.source.matchAll(/\{\{([\s\S]*?)\}\}/gu)].map((match) => {
    const expression = match[1];
    assert.ok(expression !== undefined);
    return row.local
      ? expression
      : expression.replace(/\b(x|pluginMethod|missingSibling)\b/gu, "context.$1");
  });
  const prefix = row.local
    ? "const x: number = 3;\n"
    : 'import type { ComponentPublicInstance } from "vue";\ndeclare const context: ComponentPublicInstance;\n';
  const witness = row.augmented
    ? "\ntype Equal<A,B>=(<T>()=>T extends A?1:2) extends (<T>()=>T extends B?1:2)?true:false;\ntype Assert<T extends true>=T;\ntype IsAny<T>=0 extends (1&T)?true:false;\nexport type ExactX=Assert<Equal<typeof context.x,number>>;\nexport type ExactMethod=Assert<Equal<typeof context.pluginMethod,(value:string)=>string>>;\nexport type MethodNotAny=Assert<IsAny<typeof context.pluginMethod> extends false?true:false>;\n"
    : "";
  return prefix + expressions.map((expression) => expression + ";\n").join("") + witness;
}
