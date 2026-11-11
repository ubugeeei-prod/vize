import type { RuleExample } from "./types.ts";

const vue = (source: string) => ({ language: "vue" as const, source });
const script = (source: string) =>
  vue(`<script setup lang="ts">\n${source}\n</script>`);
const pair = (bad: string, good: string, file: string): RuleExample => ({
  bad: script(bad),
  good: script(good),
  evidence: `crates/vize_patina/src/rules/opinionated/type_aware/${file}`,
  typeAware: true,
});

export const typescriptEslintExamplesA: Record<string, RuleExample> = {
  "type/await-thenable": pair(
    "function plainNumber(): number {\n  return 1;\n}\nasync function run(): Promise<number> {\n  return await plainNumber();\n}",
    "async function run(): Promise<number> {\n  return await Promise.resolve(1);\n}",
    "await_thenable.rs",
  ),
  "type/no-array-delete": pair(
    "const numbers = [1, 2, 3];\ndelete numbers[0];",
    "const numbers = [1, 2, 3];\nnumbers.splice(0, 1);",
    "no_array_delete.rs",
  ),
  "type/no-base-to-string": pair(
    "const plain = { a: 1 };\nplain.toString();",
    "const label = \"a\";\nlabel.toString();",
    "no_base_to_string.rs",
  ),
  "type/no-confusing-void-expression": pair(
    "function logVoid(): void {}\nconst value = logVoid();",
    "function logVoid(): void {}\nlogVoid();",
    "no_confusing_void_expression.rs",
  ),
  "type/no-deprecated": pair(
    "/** @deprecated use current() */\nfunction legacy(): number {\n  return 1;\n}\nconst legacyValue = legacy();",
    "function current(): number {\n  return 1;\n}\nconst currentValue = current();",
    "no_deprecated.rs",
  ),
  "type/no-duplicate-type-constituents": pair(
    "type Dup = string | string;",
    "type Dup = string;",
    "no_duplicate_type_constituents.rs",
  ),
  "type/no-for-in-array": pair(
    "const numbers = [1, 2, 3];\nfunction loopsKeys(): void {\n  for (const key in numbers) {\n    console.log(key);\n  }\n}",
    "const numbers = [1, 2, 3];\nfunction loopsKeys(): void {\n  for (const n of numbers) {\n    console.log(n);\n  }\n}",
    "no_for_in_array.rs",
  ),
  "type/no-generated-empty-object-type": pair(
    "type Emptied = Omit<{ a: number }, \"a\">;",
    "type Kept = Omit<{ a: number; b: string }, \"a\">;",
    "no_generated_empty_object_type.rs",
  ),
  "type/no-implied-eval": pair(
    "function logVoid(): void {}\nsetTimeout(\"logVoid()\", 10);",
    "setTimeout(() => {}, 10);",
    "no_implied_eval.rs",
  ),
  "type/no-meaningless-void-operator": pair(
    "function logVoid(): void {}\nvoid logVoid();",
    "function getNumber(): Promise<number> {\n  return Promise.resolve(1);\n}\nvoid getNumber();",
    "no_meaningless_void_operator.rs",
  ),
  "type/no-misused-promises": pair(
    "const numbers = [1, 2, 3];\nnumbers.forEach(async (n) => {\n  console.log(n);\n});",
    "const numbers = [1, 2, 3];\nnumbers.forEach((n) => {\n  console.log(n);\n});",
    "no_misused_promises.rs",
  ),
  "type/no-misused-spread": pair(
    "const label = \"a\";\nconst chars = [...label];",
    "const numbers = [1, 2, 3];\nconst chars = [...numbers];",
    "no_misused_spread.rs",
  ),
  "type/no-mixed-enums": pair(
    "enum Mixed {\n  A = 0,\n  B = \"b\",\n}",
    "enum Mixed {\n  A = 0,\n  B = 1,\n}",
    "no_mixed_enums.rs",
  ),
  "type/no-redundant-type-constituents": pair(
    "type Redundant = unknown | string;",
    "type Redundant = string | number;",
    "no_redundant_type_constituents.rs",
  ),
  "type/no-unnecessary-boolean-literal-compare": pair(
    "function isFlag(flag: boolean): boolean {\n  return flag === true;\n}",
    "function isFlag(flag: boolean): boolean {\n  return flag;\n}",
    "no_unnecessary_boolean_literal_compare.rs",
  ),
  "type/no-unnecessary-condition": pair(
    "function orDefault(value: string): string {\n  return value ?? \"x\";\n}",
    "function orDefault(value: string | undefined): string {\n  return value ?? \"x\";\n}",
    "no_unnecessary_condition.rs",
  ),
  "type/no-unnecessary-template-expression": pair(
    "const label = \"a\";\nconst wrapped = `${label}`;",
    "const label = \"a\";\nconst wrapped = label;",
    "no_unnecessary_template_expression.rs",
  ),
  "type/no-unnecessary-type-arguments": pair(
    "function identity<T = number>(value: T): T {\n  return value;\n}\nconst fromIdentity = identity<number>(1);",
    "function identity<T = number>(value: T): T {\n  return value;\n}\nconst fromIdentity = identity(1);",
    "no_unnecessary_type_arguments.rs",
  ),
  "type/no-unnecessary-type-assertion": pair(
    "const known: number = 1;\nconst asserted = known as number;",
    "const known: number = 1;\nconst asserted = known;",
    "no_unnecessary_type_assertion.rs",
  ),
  "type/no-unnecessary-type-conversion": pair(
    "const label = \"a\";\nconst same = String(label);",
    "const count = 1;\nconst same = String(count);",
    "no_unnecessary_type_conversion.rs",
  ),
};
