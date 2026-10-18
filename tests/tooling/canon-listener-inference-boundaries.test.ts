import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

const implementation = fs.readFileSync(
  new URL(
    "../../crates/vize_canon/src/virtual_ts/scope/component_events/inference_helpers.rs",
    import.meta.url,
  ),
  "utf8",
);
const declaration = implementation.match(/const EMIT_INFERENCE_HELPERS: &str = r#"([\s\S]+?)"#;/);
assert.ok(declaration);
const helper = declaration[1];
const predecessor =
  [
    "declare function __vizeConstructorEmitProps<P, E>(component: new (props: P, ...args: any[]) => { $emit: E }): (props: P) => __EmitProps<E>;",
    "type __VizeNativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? typeof __vizeConstructorEmitProps : (component: C) => (props: any) => {};",
    ...helper.trimEnd().split("\n").slice(2),
  ].join("\n") + "\n";

// This isolates the constructor-input boundary, preserving E without modeling
// Vue's emit overload normalization. The native upstream/generic suites and
// official public-props campaign exercise the complete Vue-owned contract.
function diagnostics(helper: string, exactOptionalPropertyTypes: boolean) {
  const lines = [
    "type __VizeIsAny<T> = 0 extends (1 & T) ? true : false;",
    "type __EmitProps<E> = { emit: E };",
    ...helper.trimEnd().split("\n"),
    "type Own = new (props: { own?: number }) => { $emit: (value: string) => void };",
  ];
  const expected: Array<{ line: number; code: number }> = [];
  const add = (source: string, code?: number) => {
    lines.push(source);
    if (code) expected.push({ line: lines.length, code });
  };
  for (const [name, metadata] of [
    ["absent", ""],
    ["empty", "{}"],
    ["undefined", "undefined"],
    ["null", "null"],
    ["never", "never"],
    ["unknown", "unknown"],
    ["any", "any"],
  ]) {
    add(
      `declare const component_${name}: Own${metadata ? ` & { readonly __vizeFallthroughProps?: ${metadata} }` : ""};`,
    );
    add(
      `const resolve_${name} = (undefined as unknown as __VizeEmitPropsFactory<typeof component_${name}>)(component_${name});`,
    );
    add(`const emit_${name}: (value: string) => void = resolve_${name}({ own: 1 }).emit;`);
    add(`resolve_${name}({ own: "wrong" });`, 2322);
    add(`resolve_${name}({ own: 1, bogus: true });`, 2353);
  }
  add("declare const inherited: Own & { readonly __vizeFallthroughProps?: { bar?: string } };");
  add(
    "const resolve_inherited = (undefined as unknown as __VizeEmitPropsFactory<typeof inherited>)(inherited);",
  );
  add("resolve_inherited({ own: 1, bar: 'valid' });", helper === predecessor ? 2353 : undefined);
  add("resolve_inherited({ own: 1, bar: 123 });", helper === predecessor ? 2353 : 2322);
  add("resolve_inherited({ own: 1, bogus: true });", 2353);
  add("declare const generic: new <T>(props: { value: T }) => { $emit: (value: T) => void };");
  add(
    "const resolve_generic = (undefined as unknown as __VizeEmitPropsFactory<typeof generic>)(generic);",
  );
  add("const generic_emit: (value: string) => void = resolve_generic({ value: 'text' }).emit;");
  add("const wrong_emit: (value: number) => void = resolve_generic({ value: 'text' }).emit;", 2322);
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    types: [],
    skipLibCheck: false,
    target: ts.ScriptTarget.ESNext,
    exactOptionalPropertyTypes,
  };
  const host = ts.createCompilerHost(options);
  const readFile = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? lines.join("\n") : readFile(file));
  const program = ts.createProgram([fixture], options, host);
  const actual = ts.getPreEmitDiagnostics(program).map((d) => {
    assert.equal(d.file?.fileName, fixture, ts.flattenDiagnosticMessageText(d.messageText, "\n"));
    assert.notEqual(d.start, undefined);
    return { line: d.file!.getLineAndCharacterOfPosition(d.start!).line + 1, code: d.code };
  });
  assert.deepEqual(actual, expected);
}

test("listener inference preserves precise inherited props without poisoning own props or generics", () => {
  for (const exactOptionalPropertyTypes of [false, true]) {
    diagnostics(helper, exactOptionalPropertyTypes);
    // A reviewed source counterfactual, not a claim to have run an old binary:
    // the original declaration rejects the valid inherited bar at the same call.
    diagnostics(predecessor, exactOptionalPropertyTypes);
  }
});
