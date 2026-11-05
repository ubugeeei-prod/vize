import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// A slot host reproduces its usage's `v-if` guard as `(guard) ? props : never`.
// The authored guard is the oracle: every rewrite in the shared case table must
// narrow each reference the props can read exactly as the authored guard does.
const repo = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, repo), "utf8");
const expressions = "crates/vize_canon/src/virtual_ts/expressions/";
type Case = { guard: string; simplified: string | null; naive?: string };
const cases = JSON.parse(read(`${expressions}implied_guard/cases.json`)) as Case[];

// Discriminated by literals only, and with a member whose discriminant is a
// plain string, which equality never removes.
const unions = {
  literal:
    "{ type: 'root'; children: unknown[] } | { type: 'text'; text: string } | { type: 'mfm'; text: string; bold?: boolean } | { type: 'select'; items: { value: string }[] } | { type: 'a' | 'b'; shared: number }",
  open: "{ type: 'root'; children: unknown[] } | { type: 'text'; text: string } | { type: 'select'; items: { value: string }[] } | { type: 'a' | 'b'; shared: number } | { type: string; open: true }",
};
const probe = "[c, c.type, props.c, props.c.type, props.kind, props.n, props.s, flag] as const";

function program(union: string, pairs: [string, string][]): string {
  return [
    "export {};",
    "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
    "type Expect<T extends true> = T;",
    `type Node = ${union};`,
    "declare const c: Node;",
    "declare const props: { c: Node; kind: 'x' | 'y' | 'z'; n: 1 | 2 | 3; s: string };",
    "declare const flag: boolean;",
    ...pairs.flatMap(([authored, rewritten], index) => [
      "// @ts-ignore Inference-only guard.",
      `const authored_${index} = (${authored}) ? ${probe} : (undefined as unknown as never);`,
      "// @ts-ignore Inference-only guard.",
      `const rewritten_${index} = (${rewritten}) ? ${probe} : (undefined as unknown as never);`,
      `type same_${index} = Expect<Equal<typeof authored_${index}, typeof rewritten_${index}>>;`,
    ]),
  ].join("\n");
}

function diagnostics(content: string): string[] {
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    target: ts.ScriptTarget.ESNext,
    lib: ["lib.esnext.d.ts"],
    types: [],
  };
  const host = ts.createCompilerHost(options);
  const readFile = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : readFile(file));
  return ts.getPreEmitDiagnostics(ts.createProgram([fixture], options, host)).map((diagnostic) => {
    const line = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line ?? 0;
    return `TS${diagnostic.code} ${content.split("\n")[line]}`;
  });
}

void test("slot hosts narrow through the shared rewrite", () => {
  const source = read(`${expressions}generic_props_call.rs`);
  assert.equal(source.split("super::implied_guard::drop_implied_exclusions(guard)").length, 2);
  const rewritten = cases.filter((entry) => entry.simplified !== null);
  assert.equal(rewritten.length, 9);
  for (const entry of rewritten) assert.notEqual(entry.simplified, entry.guard);
});

void test("every rewrite narrows exactly as the authored guard", () => {
  const pairs = cases.flatMap((entry) =>
    entry.simplified === null ? [] : [[entry.guard, entry.simplified] as [string, string]],
  );
  for (const [name, union] of Object.entries(unions)) {
    assert.deepEqual(diagnostics(program(union, pairs)), [], name);
  }
});

void test("the oracle rejects the exclusions the rewrite keeps", () => {
  const refused = cases.filter((entry) => entry.naive !== undefined);
  assert.equal(refused.length, 3);
  for (const entry of refused) {
    const failures = Object.values(unions).flatMap((union) =>
      diagnostics(program(union, [[entry.guard, entry.naive!]])),
    );
    assert.ok(
      failures.some((text) => text.startsWith("TS2344 ")),
      `${entry.guard} must not become ${entry.naive}`,
    );
  }
});
