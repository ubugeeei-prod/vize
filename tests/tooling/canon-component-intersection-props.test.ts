import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import ts from "typescript";

const root = path.resolve(import.meta.dirname, "../..");
const producer = path.join(
  root,
  "crates/vize_canon/src/virtual_ts/scope/component_prop_checker/check_helpers.rs",
);

// Exercise the shipped declarations, rather than a second implementation of
// their conditional types. These Rust strings are also valid JSON strings.
function helpers(): string {
  const declarations = new Map<string, string>();
  for (const line of fs.readFileSync(producer, "utf8").split("\n")) {
    const literal = line.trim();
    if (!literal.startsWith('"  type $')) continue;
    const declaration = JSON.parse(literal.replace(/,$/, "")) as string;
    const name = /^\s*type (\$\w+)</u.exec(declaration)?.[1];
    if (name) declarations.set(name, declaration);
  }
  return [
    "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
    ...[
      "$IsGeneratedComponent",
      "$EmitListeners",
      "$PublicProps",
      "$InstanceRawProps",
      "$StaticRawProps",
      "$HasInstanceRawProps",
      "$HasStaticRawProps",
      "$HasRawProps",
      "$ComponentRawPropsFor",
      "$ComponentRawProps",
    ].map((name) => {
      const declaration = declarations.get(name);
      assert.ok(declaration, `the producer declares ${name}`);
      if (name === "$ComponentRawProps" || name === "$ComponentRawPropsFor") {
        const shared = fs
          .readFileSync(
            path.join(
              root,
              "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt",
            ),
            "utf8",
          )
          .split("\n")
          .find((text) => text.replace(/^export /u, "").startsWith(`type ${name}<`));
        assert.equal(shared?.replace(/^export /u, ""), declaration.trim());
      }
      return declaration;
    }),
  ].join("\n");
}

test("component intersections preserve public additions without opening raw prop checks", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-intersection-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.symlinkSync(
    path.join(root, "tests/node_modules"),
    path.join(directory, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );
  const source = `
import type { DefineComponent } from "vue";
${helpers()}
type Props = { required: number };
declare const Foo: { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: Props }
  & (new <A = unknown>(props?: A) => { $props: Props; readonly __vizeRawProps?: Props });
declare const EmitsFoo: typeof Foo & { __vizeEmitProps?: { onSelect?: (n: number) => void } }
  & (new (...args: any[]) => { $props: Props & { onSelect?: (n: number) => void } });
type Lazy = DefineComponent<{ hydrateOnIdle?: number | true }>;
const forward: $ComponentRawProps<Lazy & typeof Foo> = { required: 1, hydrateOnIdle: true };
const reverse: $ComponentRawProps<typeof Foo & Lazy> = { required: 1, hydrateOnIdle: 1 };
const union: $ComponentRawProps<(Lazy & typeof Foo) | typeof Foo> = { required: 1, hydrateOnIdle: true };
type Generated<P, Q> = { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: P }
  & (new (...args: any[]) => { $props: Q; readonly __vizeRawProps?: P });
type BranchA = Generated<{ a: number }, { a?: number; extraA: boolean }>;
type BranchB = Generated<{ b: string }, { b?: string; extraB: boolean }>;
const branchA: $ComponentRawProps<BranchA | BranchB> = { a: 1, extraA: true };
const branchB: $ComponentRawProps<BranchB | BranchA> = { b: "ok", extraB: true };
// @ts-expect-error an extension from B cannot satisfy the required contract of A
const mixedBranches: $ComponentRawProps<BranchA | BranchB> = { a: 1, extraB: true };
const emits: $ComponentRawProps<Lazy & typeof EmitsFoo> = { required: 1, hydrateOnIdle: true, onSelect: (n: number) => {} };
// @ts-expect-error the added prop keeps its value type
const wrong: $ComponentRawProps<Lazy & typeof Foo> = { required: 1, hydrateOnIdle: "wrong" };
// @ts-expect-error adding optional hydration props does not relax own requiredness
const missing: $ComponentRawProps<Lazy & typeof Foo> = { hydrateOnIdle: true };
// @ts-expect-error unrelated attributes remain unknown
const unknown: $ComponentRawProps<Lazy & typeof Foo> = { required: 1, unknownAttr: true };
type Narrow = (new (...args: any[]) => { $props: { required: 1 } }) & typeof Foo;
const narrow: $ComponentRawProps<Narrow> = { required: 1 };
// @ts-expect-error an intersection can narrow an existing prop without adding keys
const wrongNarrow: $ComponentRawProps<Narrow> = { required: 2 };
type Indexed = (new (...args: any[]) => { $props: { [key: string]: any; extra?: number } }) & typeof Foo;
const indexed: $ComponentRawProps<Indexed> = { required: 1, extra: 1 };
// @ts-expect-error a public index signature must not reopen generated raw checks
const unknownIndexed: $ComponentRawProps<Indexed> = { required: 1, unknownAttr: true };
// @ts-expect-error known members next to an index signature keep their types
const wrongIndexed: $ComponentRawProps<Indexed> = { required: 1, extra: "wrong" };
type Old<C, P = $HasInstanceRawProps<C> extends true ? $InstanceRawProps<C> : $HasStaticRawProps<C> extends true ? $StaticRawProps<C> : $PublicProps<C>> = unknown extends P ? Record<string, unknown> : P;
type Equal<A, B> = [A] extends [B] ? [B] extends [A] ? true : false : false;
type Assert<T extends true> = T;
type StaticAny = { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: any } & (new () => { $props: { value?: number } });
type InstanceAny = { readonly __vizeComponentMarker: true } & (new () => { $props: { value?: number }; readonly __vizeRawProps?: any });
type AnyConstructor = { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: Props } & (new () => any);
type AnyProps = { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: Props } & (new () => { $props: any });
type External = new () => { $props: Record<string, any> };
type Unmarked = new () => { $props: { value: number }; readonly __vizeRawProps?: { fake: string } };
type Compatible = Assert<Equal<Old<StaticAny>, $ComponentRawProps<StaticAny>>
  & Equal<Old<InstanceAny>, $ComponentRawProps<InstanceAny>>
  & Equal<Old<AnyConstructor>, $ComponentRawProps<AnyConstructor>>
  & Equal<Old<AnyProps>, $ComponentRawProps<AnyProps>>
  & Equal<Old<External>, $ComponentRawProps<External>>
  & Equal<Old<Unmarked>, $ComponentRawProps<Unmarked>>>;
type Overrides = Assert<Equal<$ComponentRawProps<typeof Foo, { custom: number }>, { custom: number }>
  & Equal<$ComponentRawProps<typeof Foo, unknown>, Record<string, unknown>>
  & Equal<$ComponentRawProps<typeof Foo, any>, Record<string, unknown>>
  & Equal<$ComponentRawProps<typeof Foo, never>, never>>;
const external: $ComponentRawProps<External> = { arbitrary: true };
// @ts-expect-error unmarked authored components still check their public values
const unmarked: $ComponentRawProps<Unmarked> = { value: "wrong" };
`;
  const filename = path.join(directory, "case.ts");
  fs.writeFileSync(filename, source);
  const program = ts.createProgram([filename], {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    types: [],
  });
  assert.deepEqual(
    ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
      code: diagnostic.code,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    })),
    [],
  );
});
