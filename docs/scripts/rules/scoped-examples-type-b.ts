import type { RuleExample } from "./types.ts";

const vue = (source: string) => ({ language: "vue" as const, source });
const script = (source: string) => vue(`<script setup lang="ts">\n${source}\n</script>`);
const pair = (bad: string, good: string, file: string): RuleExample => ({
  bad: script(bad),
  good: script(good),
  evidence: `crates/vize_patina/src/rules/opinionated/type_aware/${file}`,
  typeAware: true,
});

export const typescriptEslintExamplesB: Record<string, RuleExample> = {
  "type/no-unsafe-argument": pair(
    "declare const anyValue: any;\nfunction takesNumber(value: number): number {\n  return value;\n}\ntakesNumber(anyValue);",
    "function takesNumber(value: number): number {\n  return value;\n}\ntakesNumber(1);",
    "no_unsafe_argument.rs",
  ),
  "type/no-unsafe-assignment": pair(
    "declare const anyValue: any;\nconst assigned: number = anyValue;",
    "const assigned: number = 1;",
    "no_unsafe_assignment.rs",
  ),
  "type/no-unsafe-call": pair(
    "declare const anyValue: any;\nanyValue();",
    "function plainNumber(): number {\n  return 1;\n}\nplainNumber();",
    "no_unsafe_call.rs",
  ),
  "type/no-unsafe-enum-assignment": pair(
    "enum Color {\n  Red = 0,\n  Green = 1,\n}\nconst color: Color = 1;",
    "enum Color {\n  Red = 0,\n  Green = 1,\n}\nconst color: Color = Color.Red;",
    "no_unsafe_enum_assignment.rs",
  ),
  "type/no-unsafe-enum-comparison": pair(
    "enum Color {\n  Red = 0,\n  Green = 1,\n}\nfunction isRed(value: number): boolean {\n  return value === Color.Red;\n}",
    "enum Color {\n  Red = 0,\n  Green = 1,\n}\nfunction isRed(value: Color): boolean {\n  return value === Color.Red;\n}",
    "no_unsafe_enum_comparison.rs",
  ),
  "type/no-unsafe-member-access": pair(
    "declare const anyValue: any;\nconsole.log(anyValue.member);",
    "const box = { member: 1 };\nconsole.log(box.member);",
    "no_unsafe_member_access.rs",
  ),
  "type/no-unsafe-return": pair(
    "declare const anyValue: any;\nfunction returnsAny(): number {\n  return anyValue;\n}",
    "function returnsAny(): number {\n  return 1;\n}",
    "no_unsafe_return.rs",
  ),
  "type/no-unsafe-unary-minus": pair(
    'const label = "a";\nconst negated = -label;',
    "const count = 1;\nconst negated = -count;",
    "no_unsafe_unary_minus.rs",
  ),
  "type/only-throw-error": pair(
    'function fail(): never {\n  throw "failure";\n}',
    'function fail(): never {\n  throw new Error("failure");\n}',
    "only_throw_error.rs",
  ),
  "type/prefer-promise-reject-errors": pair(
    'const rejected = Promise.reject("failure");',
    'const rejected = Promise.reject(new Error("failure"));',
    "prefer_promise_reject_errors.rs",
  ),
  "type/prefer-reduce-type-parameter": pair(
    "const numbers = [1, 2, 3];\nconst collected = numbers.reduce((all, n) => [...all, n], [] as number[]);",
    "const numbers = [1, 2, 3];\nconst collected = numbers.reduce<number[]>((all, n) => [...all, n], []);",
    "prefer_reduce_type_parameter.rs",
  ),
  "type/prefer-return-this-type": pair(
    "class Builder {\n  setValue(): Builder {\n    return this;\n  }\n}",
    "class Builder {\n  setValue(): this {\n    return this;\n  }\n}",
    "prefer_return_this_type.rs",
  ),
  "type/related-getter-setter-pairs": pair(
    "interface Box {\n  get size(): string;\n  set size(value: number);\n}",
    "interface Box {\n  get size(): string;\n  set size(value: string);\n}",
    "related_getter_setter_pairs.rs",
  ),
  "type/require-await": pair(
    "async function noAwait(): Promise<number> {\n  return 1;\n}",
    "async function withAwait(): Promise<number> {\n  return await Promise.resolve(1);\n}",
    "require_await.rs",
  ),
  "type/restrict-plus-operands": pair(
    'const label = "a";\nconst numbers = [1, 2, 3];\nconst mixedSum = label + numbers;',
    'const label = "a";\nconst extra = "b";\nconst mixedSum = label + extra;',
    "restrict_plus_operands.rs",
  ),
  "type/restrict-template-expressions": pair(
    "const numbers = [1, 2, 3];\nconst listText = `${numbers}`;",
    'const label = "a";\nconst listText = `${label}`;',
    "restrict_template_expressions.rs",
  ),
  "type/return-await": pair(
    "function getNumber(): Promise<number> {\n  return Promise.resolve(1);\n}\nasync function returnsInTry(): Promise<number> {\n  try {\n    return getNumber();\n  } catch {\n    return 0;\n  }\n}",
    "function getNumber(): Promise<number> {\n  return Promise.resolve(1);\n}\nasync function returnsInTry(): Promise<number> {\n  try {\n    return await getNumber();\n  } catch {\n    return 0;\n  }\n}",
    "return_await.rs",
  ),
  "type/unbound-method": pair(
    "class Counter {\n  value = 0;\n  increment(): void {\n    this.value += 1;\n  }\n}\nconst counter = new Counter();\nconst unbound = counter.increment;",
    "class Counter {\n  value = 0;\n  increment(): void {\n    this.value += 1;\n  }\n}\nconst counter = new Counter();\ncounter.increment();",
    "unbound_method.rs",
  ),
  "type/use-unknown-in-catch-callback-variable": pair(
    "function getNumber(): Promise<number> {\n  return Promise.resolve(1);\n}\ngetNumber().catch((error) => {\n  console.log(error);\n});",
    "function getNumber(): Promise<number> {\n  return Promise.resolve(1);\n}\ngetNumber().catch((error: unknown) => {\n  console.log(error);\n});",
    "use_unknown_in_catch_callback_variable.rs",
  ),
};
