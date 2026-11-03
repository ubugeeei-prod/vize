import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// The all-optional short-circuits in `$EmitRequiredFields` and in the exported
// `$RequiredFallthroughProps` `__K` default must compute exactly what the
// per-key scans they skip computed. The previous definitions are the oracle;
// this file is the oracle the public-default law lists for `__K`.
const declarations = new URL(
  "../../crates/vize_canon/src/virtual_ts/helpers/declarations/",
  import.meta.url,
);
const line = (file: URL, name: string): string => {
  const found = fs
    .readFileSync(file, "utf8")
    .split("\n")
    .find((text) =>
      text
        .trimStart()
        .replace(/^export /u, "")
        .startsWith(`type ${name}<`),
    );
  assert.ok(found, name);
  return found.trim().replace(/^export /u, "");
};
const current = [
  line(new URL("emit-inference.d.ts.txt", declarations), "$EmitRequiredKeys"),
  line(new URL("emit-inference.d.ts.txt", declarations), "$EmitRequiredFields"),
  line(
    new URL("../../crates/vize_canon/src/virtual_ts/component_prop_helpers.txt", import.meta.url),
    "$RequiredKeys",
  ),
  line(new URL("component-checks.d.ts.txt", declarations), "$RequiredFallthroughProps"),
];
const previous = [
  "type OldEmitRequiredFields<T> = T extends unknown ? Pick<T, $EmitRequiredKeys<T>> : never;",
  "type OldRequiredFallthroughProps<C, __F = $FallthroughProps<C>, __K = $RequiredKeys<__F>> = [__K] extends [never] ? {} : { [K in keyof __F as K extends __K ? K : never]: __F[K] };",
];

const subjects = [
  "{}",
  "{ a?: string; b?: number }",
  "{ a: string; b?: number }",
  "{ a: string; b: number }",
  "{ readonly a?: string; readonly b: number }",
  "{ a?: undefined; b: undefined; c: string | undefined }",
  "{ a?: unknown; b: unknown }",
  "{ a?: any; b: any }",
  "{ a?: never; b: never }",
  "{ [key: string]: string }",
  "{ [key: string]: string; a: string }",
  "{ [key: string]: string | undefined; a?: string }",
  "{ [key: number]: boolean | undefined; 0?: boolean; 1: boolean }",
  "{ [key: symbol]: string } & { a?: string }",
  "Record<string, number | undefined>",
  "Partial<Record<string, number>>",
  "Partial<Record<`data-${string}`, string>> & { id?: string }",
  "{ [key: `on${string}`]: () => void; onClick: () => void }",
  "{ kind: 'a'; a?: string } | { kind?: 'b'; b: number }",
  "{ a?: 1 } | { b?: 2 }",
  "{ a: 1 } | { b?: 2 }",
  "{ a?: 1 } | null | undefined",
  "{ a?: string } & { b: number }",
  "{ a?: string } & Record<string, unknown>",
  "Partial<{ x: 1; y: 2 }> & { c: 1 }",
  "Partial<CSSStyleDeclaration>",
  "Partial<HTMLElement> & { onClick?: (event: MouseEvent) => void }",
  "string",
  "string[]",
  "[a: 1, b?: 2]",
  "() => void",
  "any",
  "unknown",
  "never",
  "null",
  "undefined",
];

function diagnostics(content: string, exactOptionalPropertyTypes: boolean): string[] {
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    target: ts.ScriptTarget.ESNext,
    lib: ["lib.esnext.d.ts", "lib.dom.d.ts"],
    types: [],
    exactOptionalPropertyTypes,
  };
  const host = ts.createCompilerHost(options);
  const read = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : read(file));
  return ts.getPreEmitDiagnostics(ts.createProgram([fixture], options, host)).map((diagnostic) => {
    const where = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
    const text = ts.flattenDiagnosticMessageText(diagnostic.messageText, " ");
    return `${where ? `${where.line + 1}:${where.character + 1}` : "?"} TS${diagnostic.code} ${text}`;
  });
}

const prelude = [
  "export {};",
  "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
  "type $FallthroughProps<C> = {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
];

function equalities(helpers: string[]): string {
  return [
    ...prelude,
    ...helpers,
    ...previous,
    ...subjects.flatMap((subject, index) => [
      `type _fields${index} = Expect<Equal<$EmitRequiredFields<${subject}>, OldEmitRequiredFields<${subject}>>>;`,
      `type _fallthrough${index} = Expect<Equal<$RequiredFallthroughProps<unknown, ${subject}>, OldRequiredFallthroughProps<unknown, ${subject}>>>;`,
    ]),
  ].join("\n");
}

// A generic `T` stays deferred in these signatures and bodies. The same text
// is checked once through the current helper and once through the previous
// one; inference through either form and every check report the same
// diagnostics with the same text. (`keyof` of a deferred `$EmitRequiredFields<T>`
// is not probed: the short-circuit adds a branch whose keys are `never`; the
// last test pins that nothing reads it.)
function usage(fields: string): string {
  return [
    ...prelude,
    ...current,
    ...previous,
    `type Fields<T> = ${fields}<T>;`,
    "declare function via<T>(props: T & Fields<T>): T;",
    "declare function own<T extends { a?: string; b: number }>(props: Fields<T>): T;",
    "function deferred<T extends { a?: string; b: number }>(fields: Fields<T>, value: T) {",
    "  const fromValue: Fields<T> = value;",
    "  const missing: Fields<T> = {};",
    "  void fields; void fromValue; void missing;",
    "}",
    ...["{ a: 1 }", "{ a: 1, b: 'x' }", "{}", "{ a: undefined }", "{ ...{ a: 1 } }"].flatMap(
      (call, index) => [
        `const viaResult${index} = via(${call});`,
        `const ownResult${index} = own(${call});`,
        `export type Results${index} = [typeof viaResult${index}, typeof ownResult${index}];`,
      ],
    ),
  ].join("\n");
}

// The same for the exported `$RequiredFallthroughProps`. A deferred instance
// prints its `__K` default, so inside the generic body only position and code
// are compared; inference and every later check must match in full text.
const FALL_BODY_END = prelude.length + current.length + previous.length + 8;

function fallUsage(fallthrough: string): string {
  return [
    ...prelude,
    ...current,
    ...previous,
    `type Fall<T> = ${fallthrough}<unknown, T>;`,
    "declare function fall<T>(props: Fall<T>): T;",
    "function deferred<T extends { a?: string; b: number }>(rest: Fall<T>, value: T) {",
    "  const key: keyof typeof rest = 'b';",
    "  const fromValue: Fall<T> = value;",
    "  const missing: Fall<T> = {};",
    "  void key; void fromValue; void missing;",
    "}",
    ...["{ a: 1 }", "{ a: 1, b: 'x' }", "{}", "{ a: undefined }", "{ ...{ a: 1 } }"].flatMap(
      (call, index) => [
        `const fallResult${index} = fall(${call});`,
        `export type FallResult${index} = typeof fallResult${index};`,
      ],
    ),
  ].join("\n");
}

void test("the $RequiredFallthroughProps __K default keeps every result identical", () => {
  for (const exactOptionalPropertyTypes of [false, true]) {
    const fresh = diagnostics(fallUsage("$RequiredFallthroughProps"), exactOptionalPropertyTypes);
    const old = diagnostics(
      fallUsage("OldRequiredFallthroughProps"),
      exactOptionalPropertyTypes,
    ).map((text) => text.replaceAll("OldRequiredFallthroughProps", "$RequiredFallthroughProps"));
    const body = (text: string) => Number(text.split(":")[0]) <= FALL_BODY_END;
    const shape = (text: string) => text.split(" ").slice(0, 2).join(" ");
    assert.deepEqual(fresh.filter(body).map(shape), old.filter(body).map(shape));
    assert.deepEqual(
      fresh.filter((text) => !body(text)),
      old.filter((text) => !body(text)),
    );
    assert.ok(fresh.length > 0);
  }
});

void test("all-optional short-circuits keep every required-field result identical", () => {
  for (const exactOptionalPropertyTypes of [false, true]) {
    assert.deepEqual(
      diagnostics(equalities(current), exactOptionalPropertyTypes),
      [],
      `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`,
    );
    const fresh = diagnostics(usage("$EmitRequiredFields"), exactOptionalPropertyTypes);
    const old = diagnostics(usage("OldEmitRequiredFields"), exactOptionalPropertyTypes).map(
      (text) => text.replaceAll("OldEmitRequiredFields", "$EmitRequiredFields"),
    );
    assert.deepEqual(fresh, old);
    // The usage program exercises rejections too, so equal lists are not vacuous.
    assert.ok(fresh.length > 0);
  }
});

void test("the oracle rejects short-circuits that drop required keys", () => {
  // Negative controls: guards answering "nothing required" without the per-member test.
  const fields = [...current];
  fields[1] = "type $EmitRequiredFields<T> = T extends unknown ? Pick<T, never> : never;";
  const fallthrough = [...current];
  fallthrough[3] =
    "type $RequiredFallthroughProps<C, __F = $FallthroughProps<C>, __K = never> = [__K] extends [never] ? {} : { [K in keyof __F as K extends __K ? K : never]: __F[K] };";
  for (const broken of [fields, fallthrough]) {
    assert.ok(diagnostics(equalities(broken), false).some((text) => / TS2344 /u.test(text)));
  }
});

void test("generated code and helpers only intersect $EmitRequiredFields, never read its keys", () => {
  // The short-circuit changes `keyof` of a still-deferred `$EmitRequiredFields<T>`
  // (see above). Every consumer must stay an intersection member so nothing
  // depends on that.
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const tracked = execFileSync(
    "git",
    ["ls-files", "-z", "--", "crates/**/*.rs", "crates/**/*.txt", "crates/**/*.snap", "npm/cli"],
    {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    },
  )
    .split("\0")
    .filter((file) => /\.(?:rs|txt|ts|mts|cts|snap)$/u.test(file));
  const uses: string[] = [];
  const name = "$EmitRequiredFields<";
  for (const file of tracked) {
    const text = fs.readFileSync(`${root}${file}`, "utf8");
    for (let at = text.indexOf(name); at >= 0; at = text.indexOf(name, at + name.length)) {
      const before = text.slice(Math.max(0, at - 40), at).trimEnd();
      uses.push(`${file}: ${before.slice(before.search(/\S+$/u))}`);
    }
  }
  assert.ok(uses.length > 0);
  for (const use of uses) {
    assert.match(
      use,
      /: (?:(?:r#)?"?type|&)$/u,
      `only definitions and intersection members: ${use}`,
    );
  }
});
