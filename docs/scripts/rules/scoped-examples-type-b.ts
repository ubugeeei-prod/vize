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
  "type/unbound-method": {
    ...pair(
      "class Counter {\n  value = 0;\n  increment(): void {\n    this.value += 1;\n  }\n}\nconst counter = new Counter();\nconst unbound = counter.increment;",
      "class Counter {\n  value = 0;\n  increment(): void {\n    this.value += 1;\n  }\n}\nconst counter = new Counter();\ncounter.increment();",
      "unbound_method.rs",
    ),
    eslintDifferenceHeading: {
      en: "Differences from typescript-eslint",
      ja: "typescript-eslint との違い",
      "zh-CN": "与 typescript-eslint 的差异",
      "pt-BR": "Diferenças em relação ao typescript-eslint",
      fr: "Différences avec typescript-eslint",
    },
    eslintDifference: {
      en: "Library methods read from the global object are not reported. This covers a receiver typed `Window & typeof globalThis` or `typeof globalThis`, including `window.setTimeout`, `const { matchMedia } = window`, `self.requestAnimationFrame` and `document.defaultView?.setTimeout`. Platform operations use the global object when they are called without a receiver, so these references run as written. A method that your own code adds to `Window` is still reported. So are another frame's `Window` (`parent.setTimeout`) and a method of an object reached through the global (`window.localStorage.getItem`). Set `checkGlobalObjectMethods: true` to report these references as typescript-eslint does.\n\nA built-in that is merged into (`declare global { interface Math {} }`) keeps its exemption, and `RegExp.escape` is exempt through any path to the `RegExp` constructor. The exemption follows the TypeScript library declarations, not the Node.js version that runs the linter.",
      ja: "グローバルオブジェクトから読み出したライブラリのメソッドは報告しません。レシーバーの型が `Window & typeof globalThis` または `typeof globalThis` の場合が対象です。`window.setTimeout`、`const { matchMedia } = window`、`self.requestAnimationFrame`、`document.defaultView?.setTimeout` もこれに含まれます。プラットフォームの操作はレシーバーなしで呼ばれるとグローバルオブジェクトを使うため、書いたとおりに動きます。自分のコードで `Window` に追加したメソッドは引き続き報告します。別のフレームの `Window` (`parent.setTimeout`) や、グローバルからたどったオブジェクトのメソッド (`window.localStorage.getItem`) も同様です。`checkGlobalObjectMethods: true` を指定すると、typescript-eslint と同じくこれらも報告します。\n\n宣言をマージした組み込み (`declare global { interface Math {} }`) も除外の対象のままです。`RegExp.escape` は `RegExp` コンストラクターへのどの経路でも除外されます。除外は linter を動かす Node.js のバージョンではなく、TypeScript のライブラリ宣言に従います。",
      "zh-CN":
        "不报告从全局对象读取的库方法。这适用于类型为 `Window & typeof globalThis` 或 `typeof globalThis` 的接收者，包括 `window.setTimeout`、`const { matchMedia } = window`、`self.requestAnimationFrame` 和 `document.defaultView?.setTimeout`。平台操作在没有接收者时会使用全局对象，因此这些引用可以照常运行。你自己的代码添加到 `Window` 上的方法仍会被报告。另一个框架的 `Window`（`parent.setTimeout`）以及通过全局对象取得的对象的方法（`window.localStorage.getItem`）同样会被报告。设置 `checkGlobalObjectMethods: true` 后，这些引用会像在 typescript-eslint 中一样被报告。\n\n被合并了声明的内置对象（`declare global { interface Math {} }`）仍然享有豁免；通过任何指向 `RegExp` 构造函数的路径访问 `RegExp.escape` 都会被豁免。豁免依据的是 TypeScript 库的声明，而不是运行 linter 的 Node.js 版本。",
      "pt-BR":
        "Métodos da biblioteca lidos do objeto global não são reportados. Isso vale para um receptor tipado `Window & typeof globalThis` ou `typeof globalThis`, incluindo `window.setTimeout`, `const { matchMedia } = window`, `self.requestAnimationFrame` e `document.defaultView?.setTimeout`. Uma operação da plataforma chamada sem receptor usa o objeto global, então essas referências funcionam como escritas. Um método que o seu código adiciona a `Window` continua sendo reportado. O mesmo vale para o `Window` de outro frame (`parent.setTimeout`) e para o método de um objeto alcançado pelo objeto global (`window.localStorage.getItem`). Com `checkGlobalObjectMethods: true`, essas referências são reportadas como no typescript-eslint.\n\nUm objeto nativo que recebe uma declaração mesclada (`declare global { interface Math {} }`) mantém a isenção, e `RegExp.escape` fica isento por qualquer caminho até o construtor `RegExp`. A isenção segue as declarações da biblioteca do TypeScript, e não a versão do Node.js que executa o linter.",
      fr: "Les méthodes de la bibliothèque lues sur l’objet global ne sont pas signalées. Cela concerne un receveur typé `Window & typeof globalThis` ou `typeof globalThis`, y compris `window.setTimeout`, `const { matchMedia } = window`, `self.requestAnimationFrame` et `document.defaultView?.setTimeout`. Une opération de la plateforme appelée sans receveur utilise l’objet global, donc ces références fonctionnent telles quelles. Une méthode que votre code ajoute à `Window` reste signalée. Il en va de même pour le `Window` d’un autre cadre (`parent.setTimeout`) et pour la méthode d’un objet atteint par l’objet global (`window.localStorage.getItem`). Avec `checkGlobalObjectMethods: true`, ces références sont signalées comme dans typescript-eslint.\n\nUn objet natif auquel une déclaration est fusionnée (`declare global { interface Math {} }`) garde son exemption, et `RegExp.escape` est exempté par tout chemin vers le constructeur `RegExp`. L’exemption suit les déclarations de la bibliothèque TypeScript, et non la version de Node.js qui exécute le linter.",
    },
  },
  "type/use-unknown-in-catch-callback-variable": pair(
    "function getNumber(): Promise<number> {\n  return Promise.resolve(1);\n}\ngetNumber().catch((error) => {\n  console.log(error);\n});",
    "function getNumber(): Promise<number> {\n  return Promise.resolve(1);\n}\ngetNumber().catch((error: unknown) => {\n  console.log(error);\n});",
    "use_unknown_in_catch_callback_variable.rs",
  ),
};
