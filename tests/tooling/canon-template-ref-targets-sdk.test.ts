import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// Hoisted projections name the shared template-ref registry targets instead
// of declaring them per file. Under `skipLibCheck` an unresolved name in a
// shared declaration file silently becomes `any`, so these checks read the
// published declarations through a consumer and require exact DOM types and
// real member errors, never just an absence of diagnostics.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sibling = path.join(root, "npm/cli/virtual-ts-refs.d.ts");
const entry = (extension = "ts") =>
  fs.readFileSync(path.join(root, `npm/cli/virtual-ts.d.${extension}`), "utf8");
// The entry forwards each target; its exports must stay implicit-export free.
const forwarders = () =>
  entry()
    .split("\n")
    .filter((line) => line.includes("__vizeRef"))
    .join("\n");
const canonical = path.join(
  root,
  "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-template-refs.d.ts.txt",
);
const names = [
  "$DomElement",
  "$TemplateComponentRef",
  "$RefInstanceFactory",
  "$TemplateRefInstance",
] as const;

const unparenthesize: ts.TransformerFactory<ts.Node> = (context) => {
  const visit = (node: ts.Node): ts.Node =>
    ts.isParenthesizedTypeNode(node) ? visit(node.type) : ts.visitEachChild(node, visit, context);
  return visit;
};

function aliases(file: string, text: string): Map<string, string> {
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
  const printer = ts.createPrinter({ removeComments: true });
  const found = new Map<string, string>();
  const visit = (node: ts.Node) => {
    if (ts.isTypeAliasDeclaration(node) && (names as readonly string[]).includes(node.name.text)) {
      // Formatting may drop redundant type parentheses; the types are the same.
      const bare = ts.transform(node, [unparenthesize]).transformed[0]!;
      found.set(node.name.text, printer.printNode(ts.EmitHint.Unspecified, bare, source));
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
  return found;
}

test("the published template-ref targets are the canonical declarations", () => {
  const published = aliases(sibling, fs.readFileSync(sibling, "utf8"));
  const shared = aliases(canonical, fs.readFileSync(canonical, "utf8"));
  assert.deepEqual([...published.keys()].sort(), [...names].sort());
  assert.deepEqual(published, shared);
  const lines = forwarders().split("\n");
  assert.equal(lines.length, names.length + 1);
  assert.equal(
    lines[0],
    'import type * as __vizeRef from "./virtual-ts-refs.js" with { "resolution-mode": "import" };',
  );
  for (const name of names)
    assert.ok(
      lines.some(
        (line) => line.startsWith(`export type ${name}<`) && line.includes(`= __vizeRef.${name}<`),
      ),
      name,
    );
  for (const extension of ["mts", "cts"]) assert.equal(entry(extension), entry(), extension);
});

const probe = [
  'import type { $DomElement, $TemplateComponentRef, $RefInstanceFactory, $TemplateRefInstance } from "./virtual-ts.js";',
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  'type _Button = Expect<Equal<$DomElement<"button">, HTMLButtonElement>>;',
  'type _SvgAnchor = Expect<Equal<$DomElement<"a", true>, SVGAElement>>;',
  'type _Custom = Expect<Equal<$DomElement<"x-custom">, Element>>;',
  'type _Svg = Expect<Equal<$TemplateComponentRef<"svg">, SVGSVGElement>>;',
  "type _Ctor = Expect<Equal<$TemplateComponentRef<new () => { open(): void }>, { open(): void }>>;",
  "type _Exposed = Expect<Equal<$TemplateComponentRef<(props: {}, ctx: any, expose: (exposed: { focus(): void }) => void) => null>, { focus(): void }>>;",
  "type _AnyFactory = Expect<Equal<$RefInstanceFactory<any>, (component: any) => (props: any) => undefined>>;",
  'type _Instance = Expect<Equal<$TemplateRefInstance<{ k: () => {} }, "k", new () => { open(): void }>, { open(): void }>>;',
  'declare const input: $DomElement<"input">;',
  "input.notAMember;",
  "export {};",
].join("\n");

function check(sdk: string): string[] {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-template-ref-targets-"));
  try {
    fs.writeFileSync(path.join(dir, "virtual-ts-refs.d.ts"), sdk);
    fs.writeFileSync(path.join(dir, "virtual-ts.d.ts"), `${forwarders()}\n`);
    fs.writeFileSync(path.join(dir, "probe.ts"), probe);
    const options: ts.CompilerOptions = {
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      target: ts.ScriptTarget.ESNext,
      module: ts.ModuleKind.ESNext,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      lib: ["lib.esnext.d.ts", "lib.dom.d.ts"],
      types: [],
    };
    const program = ts.createProgram([path.join(dir, "probe.ts")], options);
    return ts.getPreEmitDiagnostics(program).map((diagnostic) => {
      const where = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
      return `${where ? where.line + 1 : "?"} TS${diagnostic.code}`;
    });
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

test("a consumer under skipLibCheck sees exact targets and real member errors", () => {
  assert.deepEqual(check(fs.readFileSync(sibling, "utf8")), ["13 TS2339"]);
});

test("an unresolved name in the shared targets cannot hide as any", () => {
  const sdk = fs.readFileSync(sibling, "utf8");
  for (const name of ["HTMLElementTagNameMap", "SVGElementTagNameMap"]) {
    const broken = sdk.replaceAll(`${name}[`, `${name}Missing[`);
    assert.notEqual(broken, sdk, name);
    assert.notDeepEqual(check(broken), ["13 TS2339"], name);
  }
});
