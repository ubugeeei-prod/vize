import { test } from "node:test";
import { stripTypeScript } from "./strip-types.ts";

void test("stripTypeScript elides type-only imports and type annotations", async (t) => {
  const result = await stripTypeScript(
    "fixture.ts",
    'import type { Foo } from "./x";\nconst a: number = 1;\n',
    false,
  );

  t.assert.equal(result.code.includes("import type"), false);
  t.assert.equal(result.code.includes(": number"), false);
  t.assert.equal(result.code.includes("Foo"), false);
  t.assert.match(result.code, /const a = 1/);
});

void test("stripTypeScript removes the satisfies operator", async (t) => {
  const result = await stripTypeScript(
    "fixture.ts",
    "const x = { a: 1 } satisfies Record<string, number>;\n",
    false,
  );

  t.assert.equal(result.code.includes("satisfies"), false);
  t.assert.equal(result.code.includes("Record"), false);
  t.assert.match(result.code, /const x = \{ a: 1 \}/);
});

void test("stripTypeScript strips as-casts and non-null assertions", async (t) => {
  const result = await stripTypeScript("fixture.ts", "const y = (z as string)!;\n", false);

  t.assert.equal(result.code.includes(" as "), false);
  t.assert.equal(result.code.includes("!"), false);
  t.assert.equal(result.code.includes("string"), false);
  t.assert.match(result.code, /const y = z/);
});

void test("stripTypeScript removes generics, interfaces, and type aliases", async (t) => {
  const result = await stripTypeScript(
    "fixture.ts",
    "interface I { a: number }\ntype T = string;\nfunction id<X>(v: X): X { return v; }\nconst r = id<number>(1);\n",
    false,
  );

  t.assert.equal(result.code.includes("interface"), false);
  t.assert.equal(result.code.includes("type T"), false);
  // generic type parameters and arguments are removed
  t.assert.equal(result.code.includes("<X>"), false);
  t.assert.equal(result.code.includes("<number>"), false);
  t.assert.match(result.code, /function id\(v\)/);
  t.assert.match(result.code, /const r = id\(1\)/);
});

void test("stripTypeScript downlevels enum to a runtime object without the enum keyword", async (t) => {
  const result = await stripTypeScript("fixture.ts", "enum E { A, B }\nconst v = E.A;\n", false);

  // oxc downlevels enums to a runtime IIFE rather than removing them entirely.
  t.assert.equal(/\benum\b/.test(result.code), false);
  t.assert.match(result.code, /var E/);
  t.assert.match(result.code, /const v = E\.A/);
});

void test("stripTypeScript downlevels namespace to runtime code without the namespace keyword", async (t) => {
  const result = await stripTypeScript(
    "fixture.ts",
    "namespace N { export const x = 1; }\nconst z = N.x;\n",
    false,
  );

  // oxc downlevels namespaces to a runtime IIFE rather than removing them entirely.
  t.assert.equal(/\bnamespace\b/.test(result.code), false);
  t.assert.match(result.code, /const z = N\.x/);
});

void test("stripTypeScript keeps used value imports while eliding type-only imports", async (t) => {
  const result = await stripTypeScript(
    "fixture.ts",
    'import { foo } from "./x";\nimport type { Bar } from "./y";\nconst a = foo();\n',
    false,
  );

  t.assert.match(result.code, /import \{ foo \} from "\.\/x"/);
  t.assert.equal(result.code.includes("Bar"), false);
  t.assert.equal(result.code.includes("./y"), false);
});

void test("stripTypeScript returns a source map only when sourceMap is true", async (t) => {
  const withMap = await stripTypeScript("fixture.ts", "const count: number = 1;\n", true);
  const withoutMap = await stripTypeScript("fixture.ts", "const count: number = 1;\n", false);

  t.assert.notEqual(withMap.map, null);
  t.assert.equal(withoutMap.map, null);
  // code is identical regardless of the sourceMap flag
  t.assert.equal(withMap.code, withoutMap.code);
  t.assert.match(withMap.code, /const count = 1/);
});

void test("stripTypeScript rejects on syntactically invalid TypeScript", async (t) => {
  await t.assert.rejects(() => stripTypeScript("fixture.ts", "const a: = ;\n", false), /Error/);
});
