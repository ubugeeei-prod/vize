import type { RuleExampleOptions } from "./types.ts";

const scope = "Vue SFC scripts and standalone TypeScript (`.ts`) modules";
const heading = "What counts as a value use";
const uses =
  "Every use the component makes of an import counts, through the same binding analysis the compiler reads: a template expression, a component tag in any casing Vue resolves, a custom directive, a template ref, a `<style>` `v-bind()`, and the other script block.";
const shadows =
  "A `v-for` or slot alias of the same name shadows the import. A type position, such as `(item as Foo)` in the template or the `generic` attribute, is a type use.";
const unread =
  "When a block cannot be read (an external `src` block, an expression that does not parse, direct `eval`, Musea `<variant>` markup), the rule reports nothing for that component.";
const inverse =
  "A name imported with `import type` that a template expression or a `<style>` `v-bind()` evaluates is reported as well, because that binding does not exist at runtime. The template read is left to `vue/no-undefined-refs` when that rule is active. Component tags and directives are not reported: a registered or auto-imported component can render them.";
const options =
  "`prefer`, `fixStyle` and `disallowTypeAnnotations` are the typescript-eslint options. `emitDecoratorMetadata` (default `true`) says whether the project emits decorator metadata, which lint does not read from `tsconfig.json`. With `true`, a type that a decorated signature serializes counts as a value use.";
const fixHeading = "The fix";
const fix =
  "A split that keeps a value import preserves its evaluation order. Converting the whole declaration to `import type` can remove module initialization under `verbatimModuleSyntax`. A declaration that must split and contains a comment or import attributes is reported without a fix.";

/** The reference page notes of `script/consistent-type-imports`. */
export const consistentTypeImportsNotes: RuleExampleOptions = {
  scope: { en: scope, ja: "Vue SFC の script ブロックと単独の TypeScript モジュール（`.ts`）" },
  note: [
    `## ${heading}`,
    uses,
    shadows,
    unread,
    inverse,
    "## Options",
    options,
    `## ${fixHeading}`,
    fix,
  ].join("\n\n"),
  noteJa: [
    "## 値としての使用に数えるもの",
    "コンパイラーが読み取るものと同じバインディング解析を使い、コンポーネント全体でのインポートの使用を数えます。テンプレートの式、Vue が解決するすべての表記のコンポーネントタグ、カスタムディレクティブ、テンプレート参照、`<style>` の `v-bind()`、もう一方のスクリプトブロックが対象です。",
    "同じ名前の `v-for` やスロットのエイリアスはインポートを隠します。テンプレート内の `(item as Foo)` や `generic` 属性のような型の位置は、型としての使用です。",
    "読み取れないブロック（外部の `src` ブロック、解析できない式、直接の `eval`、Musea の `<variant>` マークアップ）がある場合、そのコンポーネントについては何も報告しません。",
    "`import type` でインポートした名前をテンプレートの式または `<style>` の `v-bind()` が評価する場合も報告します。そのバインディングは実行時に存在しないためです。テンプレートでの読み取りは、`vue/no-undefined-refs` が有効な場合はそのルールに任せます。コンポーネントタグとディレクティブは、登録済みまたは自動インポートされたコンポーネントが描画できるため報告しません。",
    "## オプション",
    "`prefer`、`fixStyle`、`disallowTypeAnnotations` は typescript-eslint のオプションです。`emitDecoratorMetadata`（既定値 `true`）には、プロジェクトがデコレーターのメタデータを出力するかどうかを指定します。lint は `tsconfig.json` を読みません。`true` の場合、デコレーター付きのシグネチャが直列化する型を値としての使用に数えます。",
    "## 自動修正",
    "分割後も値のインポートが残る場合、その評価順は保たれます。宣言全体を `import type` に変換すると、`verbatimModuleSyntax` ではモジュールの初期化が省略されることがあります。分割が必要で、コメントまたはインポート属性を含む宣言は、修正なしで報告します。",
  ].join("\n\n"),
  eslintDifferenceHeading: {
    en: "Differences from typescript-eslint",
    ja: "typescript-eslint との差異",
    "zh-CN": "与 typescript-eslint 的差异",
    "pt-BR": "Diferenças em relação ao typescript-eslint",
    fr: "Différences avec typescript-eslint",
  },
  eslintDifference: {
    en: "typescript-eslint needs `vue-eslint-parser` to see a template use, and oxlint skips `.vue` files for this rule. A decorated signature counts as a value use unless `emitDecoratorMetadata` is `false`; typescript-eslint reads `tsconfig.json` and then skips every file that contains a decorator. The fix never merges into another declaration. A type-only import that a template expression or a `<style>` `v-bind()` evaluates is reported.",
    ja: "typescript-eslint がテンプレートでの使用を認識するには `vue-eslint-parser` が必要で、oxlint はこのルールで `.vue` ファイルを対象外にします。デコレーター付きのシグネチャは、`emitDecoratorMetadata` が `false` でない限り値としての使用に数えます。typescript-eslint は `tsconfig.json` を読み、該当する場合はデコレーターを含むファイル全体を対象外にします。自動修正はほかの宣言に統合しません。テンプレートの式または `<style>` の `v-bind()` が評価する型のみのインポートを報告します。",
    "zh-CN":
      "typescript-eslint 需要 `vue-eslint-parser` 才能识别模板中的使用，而 oxlint 的这条规则会跳过 `.vue` 文件。除非 `emitDecoratorMetadata` 为 `false`，否则带装饰器的签名计为值使用；typescript-eslint 会读取 `tsconfig.json`，并在这种情况下跳过所有包含装饰器的文件。自动修复不会合并到其他声明。模板表达式或 `<style>` 中的 `v-bind()` 求值的仅类型导入会被报告。",
    "pt-BR":
      "O typescript-eslint precisa do `vue-eslint-parser` para enxergar um uso no template, e o oxlint ignora arquivos `.vue` nesta regra. Uma assinatura decorada conta como uso de valor, a menos que `emitDecoratorMetadata` seja `false`; o typescript-eslint lê o `tsconfig.json` e, nesse caso, ignora todo arquivo que contenha um decorator. A correção nunca mescla com outra declaração. Um import somente de tipo que uma expressão do template ou um `v-bind()` em `<style>` avalia é relatado.",
    fr: "typescript-eslint a besoin de `vue-eslint-parser` pour voir un usage dans le template, et oxlint ignore les fichiers `.vue` pour cette règle. Une signature décorée compte comme un usage de valeur, sauf si `emitDecoratorMetadata` vaut `false` ; typescript-eslint lit `tsconfig.json` et ignore alors tout fichier contenant un décorateur. La correction ne fusionne jamais avec une autre déclaration. Un import de type qu’une expression du template ou un `v-bind()` dans `<style>` évalue est signalé.",
  },
};

/** `[zh-CN, pt-BR, fr]` for each English line of the note above. */
const translated: Record<string, readonly [string, string, string]> = {
  [scope]: [
    "Vue 单文件组件中的脚本和独立的 TypeScript（`.ts`）模块",
    "Scripts em SFCs Vue e módulos TypeScript independentes (`.ts`)",
    "Scripts des SFC Vue et modules TypeScript autonomes (`.ts`)",
  ],
  [heading]: [
    "哪些情况计为值使用",
    "O que conta como uso de valor",
    "Ce qui compte comme un usage de valeur",
  ],
  [uses]: [
    "规则通过编译器读取的同一份绑定分析，统计组件对导入的每一次使用：模板表达式、Vue 能解析的任意大小写形式的组件标签、自定义指令、模板引用、`<style>` 中的 `v-bind()`，以及另一个脚本块。",
    "Todo uso que o componente faz de um import conta, pela mesma análise de bindings que o compilador lê: uma expressão do template, uma tag de componente em qualquer grafia que o Vue resolve, uma diretiva personalizada, uma ref de template, um `v-bind()` em `<style>` e o outro bloco de script.",
    "Chaque usage qu’un composant fait d’un import compte, d’après la même analyse des liaisons que lit le compilateur : une expression du template, une balise de composant dans toute casse que Vue résout, une directive personnalisée, une ref de template, un `v-bind()` dans `<style>` et l’autre bloc de script.",
  ],
  [shadows]: [
    "同名的 `v-for` 或插槽别名会遮蔽该导入。类型位置属于类型使用，例如模板中的 `(item as Foo)` 或 `generic` 属性。",
    "Um alias de `v-for` ou de slot com o mesmo nome oculta o import. Uma posição de tipo, como `(item as Foo)` no template ou o atributo `generic`, é um uso de tipo.",
    "Un alias de `v-for` ou de slot du même nom masque l’import. Une position de type, comme `(item as Foo)` dans le template ou l’attribut `generic`, est un usage de type.",
  ],
  [unread]: [
    "当某个块无法读取时（外部 `src` 块、无法解析的表达式、直接调用 `eval`、Musea 的 `<variant>` 标记），规则不会对该组件报告任何问题。",
    "Quando um bloco não pode ser lido (um bloco externo com `src`, uma expressão que não é analisável, `eval` direto, marcação `<variant>` do Musea), a regra não relata nada para esse componente.",
    "Lorsqu’un bloc ne peut pas être lu (un bloc externe `src`, une expression qui ne s’analyse pas, un `eval` direct, le balisage `<variant>` de Musea), la règle ne signale rien pour ce composant.",
  ],
  [inverse]: [
    "模板表达式或 `<style>` 中的 `v-bind()` 对使用 `import type` 导入的名称求值时也会被报告，因为该绑定在运行时并不存在。启用 `vue/no-undefined-refs` 时，模板中的读取交由该规则报告。组件标签和指令不会被报告：已注册或自动导入的组件可以渲染它们。",
    "Um nome importado com `import type` que uma expressão do template ou um `v-bind()` em `<style>` avalia também é relatado, porque esse binding não existe em tempo de execução. A leitura no template fica a cargo de `vue/no-undefined-refs` quando essa regra está ativa. Tags de componente e diretivas não são relatadas: um componente registrado ou importado automaticamente pode renderizá-las.",
    "Un nom importé avec `import type` qu’une expression du template ou un `v-bind()` dans `<style>` évalue est également signalé, car cette liaison n’existe pas à l’exécution. La lecture dans le template est laissée à `vue/no-undefined-refs` lorsque cette règle est active. Les balises de composant et les directives ne sont pas signalées : un composant enregistré ou importé automatiquement peut les rendre.",
  ],
  [options]: [
    "`prefer`、`fixStyle` 和 `disallowTypeAnnotations` 是 typescript-eslint 的选项。`emitDecoratorMetadata`（默认 `true`）表示项目是否输出装饰器元数据，lint 不会从 `tsconfig.json` 读取它。为 `true` 时，带装饰器的签名所序列化的类型计为值使用。",
    "`prefer`, `fixStyle` e `disallowTypeAnnotations` são as opções do typescript-eslint. `emitDecoratorMetadata` (padrão `true`) indica se o projeto emite metadados de decorators, o que o lint não lê do `tsconfig.json`. Com `true`, um tipo que uma assinatura decorada serializa conta como uso de valor.",
    "`prefer`, `fixStyle` et `disallowTypeAnnotations` sont les options de typescript-eslint. `emitDecoratorMetadata` (`true` par défaut) indique si le projet émet les métadonnées des décorateurs, ce que le lint ne lit pas dans `tsconfig.json`. Avec `true`, un type qu’une signature décorée sérialise compte comme un usage de valeur.",
  ],
  [fixHeading]: ["自动修复", "A correção", "La correction"],
  [fix]: [
    "拆分后若保留值导入，其求值顺序保持不变。在 `verbatimModuleSyntax` 下，将整个声明转换为 `import type` 可能会跳过模块初始化。必须拆分且包含注释或导入属性的声明只报告，不提供修复。",
    "Quando a divisão mantém uma importação de valor, sua ordem de avaliação é preservada. Converter toda a declaração em `import type` pode eliminar a inicialização do módulo com `verbatimModuleSyntax`. Uma declaração que precisa ser dividida e contém um comentário ou atributos de import é relatada sem correção.",
    "Si la déclaration scindée conserve un import de valeur, son ordre d’évaluation reste inchangé. Convertir toute la déclaration en `import type` peut supprimer l’initialisation du module avec `verbatimModuleSyntax`. Une déclaration à scinder qui contient un commentaire ou des attributs d’import est signalée sans correction.",
  ],
};

const locale = (index: 0 | 1 | 2): Record<string, string> =>
  Object.fromEntries(Object.entries(translated).map(([english, texts]) => [english, texts[index]]));

export const consistentTypeImportsPhrases = {
  "zh-CN": locale(0),
  "pt-BR": locale(1),
  fr: locale(2),
};
