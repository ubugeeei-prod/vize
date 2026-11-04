import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

const repo = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, repo), "utf8");
const declarations = "crates/vize_canon/src/virtual_ts/helpers/declarations/";
const ownership = ["$ComponentMetadataKeys", "$ComponentOwnKeys", "$UnownedComponentAttrs"]
  .map((name) => {
    const line = read("crates/vize_canon/src/virtual_ts/component_prop_helpers.txt")
      .split("\n")
      .find((line) => line.startsWith(`type ${name}<`));
    assert.ok(line, name);
    return line;
  })
  .join("\n");
const helpers = [
  "export {};",
  read(`${declarations}core-types.d.ts.txt`),
  "type $PublicComponentInputProps = { hidden?: boolean };",
  read(`${declarations}emit-overloads.d.ts.txt`),
  read(`${declarations}emit-props.d.ts.txt`),
  ownership,
  read(`${declarations}emit-inference.d.ts.txt`),
].join("\n");

// Hand-authored external constructors exercise the actual production mapper.
// These are structural type-law controls, not generated SFC observations.
const body = `
type Equal<A,B> = (<T>()=>T extends A ? 1 : 2) extends (<T>()=>T extends B ? 1 : 2) ? true : false;
type Expect<T extends true> = T;
declare const payload: new<T>(props:{value:T})=>{$emit:(event:'save',value:T)=>void};
declare const payloadFactory: $EmitPropsFactory<typeof payload>;
const payloadResult=payloadFactory(payload)({value:'text'});
payloadResult.onSave!('ok');
payloadResult.onSave!(123);
declare const genericEvent: new<T extends string>(props:{event:T})=>{$emit:(event:T,value:number)=>void};
declare const eventFactory: $EmitPropsFactory<typeof genericEvent>;
const eventResult=eventFactory(genericEvent)({event:'save' as const});
eventResult.onSave!(123);
eventResult.onSave!('wrong');
declare const wide: new(props:{})=>{$emit:(event:string,...args:any[])=>void};
declare const wideFactory: $EmitPropsFactory<typeof wide>;
const wideResult=wideFactory(wide)({});
type ProvenWideRemainsLoose = Expect<Equal<typeof wideResult,{readonly __vizeLooseListener:[]}>>;
`;

for (const exactOptionalPropertyTypes of [false, true]) {
  test(`generic event names specialize before loose classification (exact optional: ${exactOptionalPropertyTypes})`, () => {
    const content = `${helpers}\n${body}`;
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
    host.readFile = (file) => (file === fixture ? content : readFile(file));
    const program = ts.createProgram([fixture], options, host);
    const actual = ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
      file: diagnostic.file?.fileName,
      code: diagnostic.code,
      category: diagnostic.category,
      start: diagnostic.start,
      length: diagnostic.length,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    }));
    const expected = [
      {
        file: fixture,
        code: 2345,
        category: ts.DiagnosticCategory.Error,
        start: content.indexOf("payloadResult.onSave!(123)") + "payloadResult.onSave!(".length,
        length: 3,
        message: "Argument of type 'number' is not assignable to parameter of type 'string'.",
      },
      {
        file: fixture,
        code: 2345,
        category: ts.DiagnosticCategory.Error,
        start: content.indexOf("eventResult.onSave!('wrong')") + "eventResult.onSave!(".length,
        length: 7,
        message: "Argument of type 'string' is not assignable to parameter of type 'number'.",
      },
    ];
    assert.deepEqual(actual, expected);
  });
}
