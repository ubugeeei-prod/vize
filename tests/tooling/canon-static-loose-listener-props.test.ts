import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";
import {
  beforeStaticListenerSource,
  staticListenerRecord,
  staticListenerSource,
} from "./support/static-listener-prop-source.ts";
const root = new URL("../../", import.meta.url);
const generator = fs.readFileSync(
  new URL("crates/vize_canon/src/virtual_ts/scope/component_events/generic_inference.rs", root),
  "utf8",
);
const emitted = generator.match(/"\{indent\}type \{inferred_args\} = ([^\n]+);\\n",/u);
assert.ok(emitted);
function inferred(name: string, value: string, key: string, dynamic: boolean, legacy = false) {
  const template = `type {inferred_args} = ${emitted![1]};`;
  const values: Record<string, string> = {
    inferred_args: name,
    library: "false",
    emit_props: value,
    prop_key: JSON.stringify(key),
    loose_args: dynamic || legacy ? "__L" : "unknown[]",
  };
  return template.replace(/\{\{|\}\}|\{([a-z_]+)\}/gu, (token, field?: string) => {
    if (token === "{{") return "{";
    if (token === "}}") return "}";
    assert.ok(values[field!]);
    return values[field!]!;
  });
}
const body = `
import type { DefineComponent } from "vue";
import type { $EmitPropsFactory } from "../virtual-ts.js" with { "resolution-mode": "import" };
type Equal<A,B>=(<T>()=>T extends A?1:2) extends (<T>()=>T extends B?1:2)?true:false;
type Expect<T extends true>=T;
declare const slider: DefineComponent<{modelValue?:number|null;"onUpdate:modelValue"?:(value:number|null)=>any}>;
declare const sliderFactory:$EmitPropsFactory<typeof slider>;
const sliderEmitProps=sliderFactory(slider)({modelValue:1 as number|null});
${inferred("StaticInferred", "sliderEmitProps", "onUpdate:modelValue", false)}
${inferred("DynamicInferred", "sliderEmitProps", "onUpdate:modelValue", true)}
${inferred("LegacyInferred", "sliderEmitProps", "onUpdate:modelValue", false, true)}
type Props=InstanceType<typeof slider>['$props'];
type PropArgs=Props extends {"onUpdate:modelValue"?:(...args:infer A)=>any}?A:unknown[];
type StaticArgs=unknown[] extends StaticInferred?PropArgs:StaticInferred;
type StaticPayload=Expect<Equal<StaticArgs,[value:number|null]>>;
type DynamicMarkerRemainsDistinct=Expect<Equal<DynamicInferred,[]>>;
type LegacyMarkerRemainsDistinct=Expect<Equal<LegacyInferred,[]>>;
const good:(...args:StaticArgs)=>any=(value:number|null)=>value;
const numberOnly:(...args:StaticArgs)=>any=(value:number)=>{value.toFixed()};
const stringOnly:(...args:StaticArgs)=>any=(value:string)=>value.toUpperCase();
declare const zero:new(props:{})=>{$props:{onClose?:(value:number)=>any};$emit:(event:'close')=>void};
declare const zeroFactory:$EmitPropsFactory<typeof zero>;
const zeroEmitProps=zeroFactory(zero)({});
${inferred("ZeroInferred", "zeroEmitProps", "onClose", false)}
type ActualZeroRemainsZero=Expect<Equal<ZeroInferred,[]>>;
declare const typed:new<T>(props:{value:T})=>{$emit:(event:'save',value:T)=>void};
declare const typedFactory:$EmitPropsFactory<typeof typed>;
const typedEmitProps=typedFactory(typed)({value:1});
${inferred("TypedInferred", "typedEmitProps", "onSave", false)}
type TypedPayloadPreserved=Expect<Equal<TypedInferred,[value:number]>>;
declare const declaredVue: DefineComponent<{}, {}, {}, {}, {}, {}, {}, {save:(value:number)=>void}>;
declare const declaredVueFactory:$EmitPropsFactory<typeof declaredVue>;
const declaredVueEmitProps=declaredVueFactory(declaredVue)({});
${inferred("DeclaredVueInferred", "declaredVueEmitProps", "onSave", false)}
type DeclaredVuePayloadPreserved=Expect<Equal<DeclaredVueInferred,[value:number]>>;
`;
void test("only static loose markers use the certified prop fallback", () => {
  assert.match(
    generator,
    /let loose_args = if dynamic_component \|\| !ctx\.contextual_listener \{\s*"__L"\s*\} else \{\s*"unknown\[\]"\s*\};/u,
  );
  const read = (path: string) => fs.readFileSync(new URL(path, root), "utf8");
  const current = read(staticListenerSource);
  const before = beforeStaticListenerSource(read, staticListenerSource, current);
  assert.ok(before.includes("? __L : typeof {emit_props}"));
  const record = JSON.parse(read(staticListenerRecord)) as { file: { edits: { after: string }[] } };
  for (const changed of [
    current + "\n",
    current + current,
    current.replace(record.file.edits[0]!.after, ""),
  ])
    assert.throws(() => beforeStaticListenerSource(read, staticListenerSource, changed));
  assert.throws(() =>
    beforeStaticListenerSource(
      (path) => read(path) + (path === staticListenerRecord ? " " : ""),
      staticListenerSource,
      current,
    ),
  );
});
for (const exactOptionalPropertyTypes of [false, true]) {
  void test(`static loose emits preserve declared Vue listener props (exact optional: ${exactOptionalPropertyTypes})`, () => {
    const file = fileURLToPath(new URL("npm/cli/tests/static-loose-listener-props.ts", root));
    const options: ts.CompilerOptions = {
      strict: true,
      noEmit: true,
      skipLibCheck: false,
      types: [],
      target: ts.ScriptTarget.ESNext,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
      exactOptionalPropertyTypes,
    };
    const host = ts.createCompilerHost(options),
      read = host.readFile.bind(host);
    host.readFile = (name) => (name === file ? body : read(name));
    const program = ts.createProgram([file], options, host);
    const actual = ts.getPreEmitDiagnostics(program).map((d) => ({
      file: d.file?.fileName,
      code: d.code,
      category: d.category,
      start: d.start,
      length: d.length,
      message: ts.flattenDiagnosticMessageText(d.messageText, "\n"),
    }));
    const expected = [
      { name: "numberOnly", source: "number", method: "void", narrow: "number" },
      { name: "stringOnly", source: "string", method: "string", narrow: "string" },
    ].map(({ name, source, method, narrow }) => ({
      file,
      code: 2322,
      category: ts.DiagnosticCategory.Error,
      start: body.indexOf(`const ${name}:`) + "const ".length,
      length: name.length,
      message: `Type '(value: ${source}) => ${method}' is not assignable to type '(value: number | null) => any'.\n  Types of parameters 'value' and 'value' are incompatible.\n    Type 'number | null' is not assignable to type '${narrow}'.\n      Type 'null' is not assignable to type '${narrow}'.`,
    }));
    assert.deepEqual(actual, expected);
  });
}
