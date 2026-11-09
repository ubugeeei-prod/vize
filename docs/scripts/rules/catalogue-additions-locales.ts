import type { TranslatedLocale } from "./rule-translations.ts";
import { catalogueAdditionExamples } from "./catalogue-addition-examples.ts";

type Packet = readonly [purpose: string, bad: string, good: string];

export const catalogueAdditionPackets: Record<TranslatedLocale, Record<string, Packet>> = {
  "zh-CN": {
    "vue/comment-directive": [
      "支持 `<template>` 中的诊断控制注释。",
      "这个禁用所有规则的注释没有在后面的 `div` 上抑制任何诊断，因此会被未使用指令检查报告。",
      "删除无用的注释并保留原有元素，让这一行继续接受 lint 检查。",
    ],
    "vue/html-end-tags": [
      "要求显式书写 HTML 中可以省略的结束标签。",
      "HTML 允许在 `ul` 结束时隐式关闭这个 `li`，但本规则要求显式书写结束标签。",
      "在列表项内容之后、外层 `ul` 结束之前加上 `</li>`。",
    ],
    "vue/jsx-uses-vars": [
      "将 JSX 元素名称中引用的变量视为已使用。",
      "`MyButton` 已导入，但模板和 JSX 都没有引用它，因此配合使用的 `vue/no-unused-components` 会报告它。",
      "JSX 标签引用了导入的 `MyButton`；辅助规则会记录这次使用，配合使用的规则便不再报告未使用的组件。",
    ],
    "vue/match-component-file-name": [
      "要求声明的组件名称与文件名匹配。",
      "声明的组件名 `OtherPanel` 即使经过支持的大小写转换，也不匹配 `MyPanel.vue` 去掉扩展名后的文件名。",
      "将组件命名为 `MyPanel`，使声明的名称与所在文件一致。",
    ],
    "vue/no-custom-modifiers-on-v-model": [
      "为兼容 Vue 2，禁止在组件的 `v-model` 上使用自定义修饰符。",
      "组件的 `.capitalize` 是自定义修饰符，违反了这里的 Vue 2 兼容性限制。",
      "使用不带自定义修饰符的普通组件 `v-model` 约定；如果需要首字母大写，应显式实现该处理。",
    ],
    "vue/no-ref-object-reactivity-loss": [
      "禁止在同一作用域中复制 ref 的值而丢失后续响应式更新。",
      "在脚本根作用域将 `count.value` 读入 `snapshot`，只会保存当前数值，不能跟随后续 ref 更新。",
      "计算属性的 getter 会在依赖变化时读取 `count.value`，保留响应式结果，避免根作用域中的值复制。",
    ],
    "vue/no-setup-props-reactivity-loss": [
      "禁止通过复制 setup 的 prop 值而丢失后续响应式更新。",
      "在脚本根作用域复制 `props.count` 会保存当前值，无法跟随后续 prop 更新。",
      "在计算属性的 getter 中读取 `props.count`，让派生结果继续跟随 prop 的变化。",
    ],
    "vue/no-this-in-before-route-enter": [
      "禁止在 `beforeRouteEnter` 中通过 `this` 访问尚未创建的组件实例。",
      "`beforeRouteEnter` 在组件实例可用之前运行，因此 `this.load()` 无法访问该实例。",
      "使用传给 `next` 回调的实例，在组件创建后调用 `vm.load()`。",
    ],
    "vue/no-v-for-template-key": [
      "为兼容 Vue 2，要求将 `template v-for` 的 key 放在实际子元素上。",
      "在 Vue 2 约定中，`<template v-for>` 不会渲染实际元素，其上的 key 会被忽略。",
      "按照这里检查的 Vue 2 约定，将列表项的 key 放在重复渲染的实际子元素 `li` 上。",
    ],
    "vue/prefer-prop-type-boolean-first": [
      "要求在组件 prop 的类型构造函数数组中将 `Boolean` 放在首位。",
      "`String` 位于 `Boolean` 之前，因此不带值的 `enabled` 属性采用字符串优先的转换，而非布尔值优先的约定。",
      "将 `Boolean` 放在首位，让 prop 优先进行布尔值转换，同时保留对字符串类型的支持。",
    ],
    "vue/require-expose": [
      "要求 Options API 组件通过 `expose` 显式声明公开属性。",
      "这个 Options API 组件包含 `focus` 方法，却没有显式声明公开实例的接口。",
      "通过 `expose` 列表显式声明，可从组件的公开实例访问 `focus`。",
    ],
    "vue/valid-model-definition": [
      "要求 Vue 2 的 `model` 选项使用有效的键。",
      "`events` 不是 Vue 2 组件 `model` 选项支持的键。",
      "使用受支持的单数形式 `event` 和 `prop`，描述 Vue 2 的 model 约定。",
    ],
  },
  "pt-BR": {
    "vue/comment-directive": [
      "Aceita comentários de controle de diagnósticos em `<template>`.",
      "O comentário que desativa todas as regras não suprime nenhum diagnóstico no `div` seguinte, por isso a verificação de diretivas não utilizadas o reporta.",
      "Remova o comentário desnecessário e mantenha o elemento, deixando as verificações de lint ativas nessa linha.",
    ],
    "vue/html-end-tags": [
      "Exige tags de fechamento explícitas para elementos HTML cujo fechamento pode ser omitido.",
      "O HTML permite fechar esse `li` implicitamente quando `ul` termina, mas a regra exige a tag de fechamento explícita.",
      "Adicione `</li>` depois do conteúdo do item, antes de fechar o `ul` que o contém.",
    ],
    "vue/jsx-uses-vars": [
      "Considera usadas as variáveis referenciadas nos nomes dos elementos JSX.",
      "`MyButton` é importado, mas nem o template nem o JSX o referenciam, então a regra complementar `vue/no-unused-components` o reporta.",
      "A tag JSX referencia o `MyButton` importado; a regra auxiliar reconhece esse uso e a regra complementar deixa de reportar um componente não utilizado.",
    ],
    "vue/match-component-file-name": [
      "Exige que o nome declarado do componente corresponda ao nome do arquivo.",
      "O nome declarado `OtherPanel` não corresponde ao nome de `MyPanel.vue` sem a extensão, mesmo com as conversões de caixa aceitas.",
      "Nomeie o componente como `MyPanel` para que o nome declarado corresponda ao arquivo que o contém.",
    ],
    "vue/no-custom-modifiers-on-v-model": [
      "Proíbe modificadores personalizados no `v-model` de componentes para manter compatibilidade com Vue 2.",
      "O modificador `.capitalize` do componente é personalizado e viola esta restrição de compatibilidade com Vue 2.",
      "Use o contrato normal de `v-model` do componente, sem um modificador personalizado; implemente explicitamente a capitalização, se necessária.",
    ],
    "vue/no-ref-object-reactivity-loss": [
      "Proíbe copiar o valor de um ref no mesmo escopo e perder suas atualizações reativas.",
      "Ler `count.value` em `snapshot` na raiz do script captura o número atual, sem acompanhar as atualizações posteriores do ref.",
      "Um getter computado lê `count.value` quando sua dependência muda, preservando um resultado reativo em vez de copiar o valor no escopo raiz.",
    ],
    "vue/no-setup-props-reactivity-loss": [
      "Proíbe cópias dos valores das props de setup que perdem as atualizações reativas.",
      "Copiar `props.count` na raiz do script mantém seu valor atual e não acompanha as atualizações posteriores da prop.",
      "Leia `props.count` em um getter computado para manter o resultado derivado ligado às mudanças da prop.",
    ],
    "vue/no-this-in-before-route-enter": [
      "Proíbe acessar com `this` uma instância ainda não criada em `beforeRouteEnter`.",
      "`beforeRouteEnter` executa antes de a instância do componente estar disponível, então `this.load()` não pode acessá-la.",
      "Use a instância recebida pelo callback de `next` e chame `vm.load()` depois que o componente for criado.",
    ],
    "vue/no-v-for-template-key": [
      "Exige que a key de `template v-for` fique em um filho real para manter compatibilidade com Vue 2.",
      "No contrato do Vue 2, a key em `<template v-for>` é ignorada porque o template não renderiza um elemento real.",
      "Coloque a key do item no filho real `li` repetido, conforme a convenção do Vue 2 verificada aqui.",
    ],
    "vue/prefer-prop-type-boolean-first": [
      "Exige `Boolean` na primeira posição da lista de construtores de tipo das props.",
      "`String` vem antes de `Boolean`, então um atributo `enabled` sem valor usa a conversão que prioriza strings em vez da convenção que prioriza booleanos.",
      "Coloque `Boolean` primeiro para priorizar a conversão booleana da prop, mantendo a aceitação do tipo string.",
    ],
    "vue/require-expose": [
      "Exige que componentes da Options API declarem explicitamente suas propriedades públicas com `expose`.",
      "O componente da Options API tem um método `focus` sem declarar explicitamente a interface de sua instância pública.",
      "Uma lista `expose` disponibiliza explicitamente `focus` pela instância pública do componente.",
    ],
    "vue/valid-model-definition": [
      "Exige chaves válidas na opção `model` do Vue 2.",
      "`events` não é uma chave aceita na opção `model` de componentes do Vue 2.",
      "Use a chave aceita no singular, `event`, junto de `prop`, para descrever o contrato de model do Vue 2.",
    ],
  },
  fr: {
    "vue/comment-directive": [
      "Prend en charge les commentaires de contrôle des diagnostics dans `<template>`.",
      "Le commentaire qui désactive toutes les règles ne supprime aucun diagnostic sur le `div` suivant ; le contrôle des directives inutilisées le signale donc.",
      "Supprimez le commentaire inutile et conservez l'élément, afin de laisser les contrôles de lint actifs sur cette ligne.",
    ],
    "vue/html-end-tags": [
      "Exige une balise de fermeture explicite pour les éléments HTML qui permettent de l'omettre.",
      "HTML permet de fermer implicitement ce `li` à la fermeture de `ul`, mais la règle exige une balise de fermeture explicite.",
      "Ajoutez `</li>` après le contenu de l'élément, avant la fermeture du `ul` qui le contient.",
    ],
    "vue/jsx-uses-vars": [
      "Considère comme utilisées les variables référencées dans les noms d'éléments JSX.",
      "`MyButton` est importé, mais ni le template ni le JSX ne le référencent ; la règle complémentaire `vue/no-unused-components` le signale donc.",
      "La balise JSX référence le `MyButton` importé ; la règle auxiliaire reconnaît cet usage et la règle complémentaire ne signale plus de composant inutilisé.",
    ],
    "vue/match-component-file-name": [
      "Exige que le nom déclaré du composant corresponde au nom du fichier.",
      "Le nom déclaré `OtherPanel` ne correspond pas au nom de `MyPanel.vue` sans extension, même après les conversions de casse prises en charge.",
      "Nommez le composant `MyPanel` pour que son nom déclaré corresponde au fichier qui le contient.",
    ],
    "vue/no-custom-modifiers-on-v-model": [
      "Interdit les modificateurs personnalisés sur le `v-model` d'un composant pour assurer la compatibilité avec Vue 2.",
      "Le modificateur `.capitalize` du composant est personnalisé et enfreint cette restriction de compatibilité avec Vue 2.",
      "Utilisez le contrat ordinaire de `v-model` du composant sans modificateur personnalisé ; implémentez explicitement la mise en majuscule si elle est nécessaire.",
    ],
    "vue/no-ref-object-reactivity-loss": [
      "Interdit de copier la valeur d'une ref dans la même portée en perdant ses mises à jour réactives.",
      "Lire `count.value` dans `snapshot` à la racine du script capture le nombre actuel, sans suivre les mises à jour ultérieures de la ref.",
      "Un getter calculé lit `count.value` lorsque sa dépendance change, préservant un résultat réactif plutôt qu'une copie dans la portée racine.",
    ],
    "vue/no-setup-props-reactivity-loss": [
      "Interdit les copies des valeurs des props de setup qui perdent leurs mises à jour réactives.",
      "Copier `props.count` à la racine du script conserve sa valeur actuelle et ne suit pas les mises à jour ultérieures de la prop.",
      "Lisez `props.count` dans un getter calculé pour que le résultat dérivé continue de suivre les changements de la prop.",
    ],
    "vue/no-this-in-before-route-enter": [
      "Interdit d'accéder par `this` à une instance encore inexistante dans `beforeRouteEnter`.",
      "`beforeRouteEnter` s'exécute avant que l'instance du composant soit disponible ; `this.load()` ne peut donc pas y accéder.",
      "Utilisez l'instance passée au callback de `next` et appelez `vm.load()` après la création du composant.",
    ],
    "vue/no-v-for-template-key": [
      "Exige de placer la key de `template v-for` sur un enfant réel pour assurer la compatibilité avec Vue 2.",
      "Dans le contrat de Vue 2, la key sur `<template v-for>` est ignorée, car le template ne produit pas d'élément réel.",
      "Placez la key de l'élément sur l'enfant réel `li` répété, conformément à la convention de Vue 2 vérifiée ici.",
    ],
    "vue/prefer-prop-type-boolean-first": [
      "Exige que `Boolean` soit le premier constructeur dans la liste des types d'une prop.",
      "`String` précède `Boolean` : un attribut `enabled` sans valeur utilise donc une conversion privilégiant les chaînes plutôt que les booléens.",
      "Placez `Boolean` en premier pour privilégier la conversion booléenne de la prop, tout en conservant la prise en charge des chaînes.",
    ],
    "vue/require-expose": [
      "Exige que les composants de l'Options API déclarent explicitement leurs propriétés publiques avec `expose`.",
      "Le composant de l'Options API possède une méthode `focus` sans déclarer explicitement l'interface de son instance publique.",
      "Une liste `expose` rend explicitement `focus` accessible par l'instance publique du composant.",
    ],
    "vue/valid-model-definition": [
      "Exige des clés valides dans l'option `model` de Vue 2.",
      "`events` n'est pas une clé prise en charge dans l'option `model` des composants Vue 2.",
      "Utilisez la clé prise en charge au singulier, `event`, avec `prop`, pour décrire le contrat de model de Vue 2.",
    ],
  },
};

const note = (id: string): string => {
  const text = catalogueAdditionExamples[id]?.note;
  if (!text) throw new Error(`Missing catalogue addition note: ${id}`);
  return text;
};

export const catalogueAdditionNotes: Record<TranslatedLocale, Record<string, string>> = {
  "zh-CN": {
    "List component names whose rendered subtree always supplies accessible anchor content. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "列出其渲染子树始终提供可访问链接内容的组件名。PascalCase 与 kebab-case 写法等价。选项不会启用规则；后续配置层会替换整个列表，[] 会清除继承的名称。",
    "List component names that render a native form control and therefore satisfy label nesting. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "列出会渲染原生表单控件、因而满足 label 嵌套条件的组件名。PascalCase 与 kebab-case 写法等价。选项不会启用规则；后续配置层会替换整个列表，[] 会清除继承的名称。",
    "The allowlist replaces eslint-plugin-vue's punctuation defaults when present. Entries are literal substrings or `/pattern/flags` values using Rust regex syntax and are removed before the remaining text is checked; patterns that do not compile remain literals. Omit allowlist to retain the defaults; [] clears them. Options do not enable the rule.":
      "指定 allowlist 时，它会替换 eslint-plugin-vue 默认的标点列表。各项是字面子字符串或使用 Rust 正则表达式语法的 `/pattern/flags` 值，并在检查剩余文本前移除；无法编译的模式会按字面量处理。省略 allowlist 可保留默认值；[] 会清空它们。选项不会启用规则。",
    "List exact globally registered directive names without the v- prefix. Local script-setup vName bindings are still discovered automatically. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "列出全局注册指令的精确名称，不包含 v- 前缀。仍会自动发现 script-setup 中本地的 vName 绑定。选项不会启用规则；后续配置层会替换整个列表，[] 会清除继承的名称。",
    [note("vue/comment-directive")]:
      "只有启用 `reportUnusedDisableDirectives: true` 时，才会报告未使用的模板抑制注释。示例删除了无用的注释；实际抑制诊断的注释仍然有用。列出的规则名称只有在 Vize 执行这些规则时才参与使用情况判定。",
    [note("vue/html-end-tags")]:
      "本规则要求显式书写 HTML 解析器允许省略的结束标签，例如 `li`、`p` 和 `td`。缺失的必需结束标签由 `parser/template` 诊断；空元素不需要结束标签。",
    [note("vue/jsx-uses-vars")]:
      "这个辅助规则本身不会发出 `vue/jsx-uses-vars` 诊断。它会在 `vue/no-unused-components` 判断组件是否已使用时，将 JSX 标签绑定计为已使用。本示例需要同时启用这两条规则。`<MyButton.Part />` 将根绑定 `MyButton` 计为已使用；带命名空间的标签和 `this` 成员不提供导入的组件绑定。",
    [note("vue/match-component-file-name")]:
      "文件名也是检查的一部分。默认比较方式是将声明的组件名转换为 PascalCase 或 kebab-case，再与未作变换、去掉扩展名的文件名比较。包含多个组件定义的文件不作检查。",
    [note("vue/no-custom-modifiers-on-v-model")]:
      "这条按需启用的规则对组件 `v-model` 施加 Vue 2 兼容性约定。只允许 `lazy`、`number` 和 `trim` 修饰符。Vue 3 组件可以实现自定义修饰符，因此 Vue 3 预设不启用此限制。原生元素不在这条规则的检查范围内。",
    [note("vue/no-ref-object-reactivity-loss")]:
      "检查会跟踪可识别的 ref 工厂函数以及同一作用域中的值复制。其他函数内部的读取和对 `.value` 的写入不作检查。这是一条基于源码、按需启用的规则，不需要原生类型查询。",
    [note("vue/no-setup-props-reactivity-loss")]:
      "检查 Options API 的 `setup` 和 `<script setup>` 中根作用域的 prop 值复制。Vue 3.5 的响应式 props 解构本身仍被允许；之后在脚本根作用域复制解构出的 prop 会被报告。允许在其他函数内部读取。",
    [note("vue/no-v-for-template-key")]:
      "这是一条按需启用的 Vue 2 兼容性规则。Vue 2 要求将 key 放在实际子元素上；Vue 3 则要求放在 `<template v-for>` 上。要检查 Vue 3 的约定，请使用 `vue/no-v-for-template-key-on-child`，不要同时启用这两种约定。",
    [note("vue/prefer-prop-type-boolean-first")]:
      "更改构造函数顺序可能改变运行时的布尔值转换。这条按需启用的规则检查运行时的 `defineProps` 和 Options API 的 prop 类型数组；不会重排 TypeScript 联合类型。",
    [note("vue/require-expose")]:
      "这项检查针对 Options API 的组件对象。显式声明空的 `expose` 列表也是有效的。setup 中调用 `expose` 或返回渲染函数也能满足检查；mixin 不在检查范围内。这个脚本检查不会查看并列的 `<script setup>` 块，`defineExpose` 也不是此规则的目标。",
    [note("vue/valid-model-definition")]:
      "这条按需启用的规则检查 Vue 2 的 `model` 选项。只接受静态键 `prop` 和 `event`，不验证它们的值。Vue 3 使用较新的组件 `v-model` 约定，而不是这个选项。",
  },
  "pt-BR": {
    "List component names whose rendered subtree always supplies accessible anchor content. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "Liste os componentes cuja subárvore renderizada sempre fornece conteúdo acessível ao link. As grafias PascalCase e kebab-case são equivalentes. As opções não ativam a regra; camadas posteriores substituem a lista inteira, e [] remove os nomes herdados.",
    "List component names that render a native form control and therefore satisfy label nesting. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "Liste os componentes que renderizam um controle de formulário nativo e satisfazem o aninhamento do label. As grafias PascalCase e kebab-case são equivalentes. As opções não ativam a regra; camadas posteriores substituem a lista inteira, e [] remove os nomes herdados.",
    "The allowlist replaces eslint-plugin-vue's punctuation defaults when present. Entries are literal substrings or `/pattern/flags` values using Rust regex syntax and are removed before the remaining text is checked; patterns that do not compile remain literals. Omit allowlist to retain the defaults; [] clears them. Options do not enable the rule.":
      "Quando presente, allowlist substitui os padrões de pontuação do eslint-plugin-vue. As entradas são substrings literais ou valores `/pattern/flags` na sintaxe de regex do Rust, removidos antes da verificação do texto restante; padrões que não compilam permanecem literais. Omita allowlist para manter os padrões; [] os remove. As opções não ativam a regra.",
    "List exact globally registered directive names without the v- prefix. Local script-setup vName bindings are still discovered automatically. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "Liste os nomes exatos das diretivas registradas globalmente, sem o prefixo v-. Bindings vName locais de script-setup continuam sendo descobertos automaticamente. As opções não ativam a regra; camadas posteriores substituem a lista inteira, e [] remove os nomes herdados.",
    [note("vue/comment-directive")]:
      "Comentários de supressão não utilizados no template só são reportados quando `reportUnusedDisableDirectives: true` está ativado. O exemplo remove um comentário sem uso; comentários que realmente suprimem um diagnóstico continuam úteis. Nomes de regras listados só contam quando o Vize executa essas regras.",
    [note("vue/html-end-tags")]:
      "Esta regra exige tags de fechamento explícitas onde o parser HTML permite omiti-las, como em `li`, `p` e `td`. A ausência de tags de fechamento obrigatórias é diagnosticada por `parser/template`; elementos vazios não precisam de tag de fechamento.",
    [note("vue/jsx-uses-vars")]:
      "Esta regra auxiliar não emite um diagnóstico `vue/jsx-uses-vars`. Ela reconhece os bindings de tags JSX quando `vue/no-unused-components` decide se um componente é usado. Ative as duas regras neste exemplo. `<MyButton.Part />` reconhece o binding raiz `MyButton`; tags com namespace e membros de `this` não fornecem um binding de componente importado.",
    [note("vue/match-component-file-name")]:
      "O nome do arquivo faz parte desta verificação. A comparação padrão converte o nome declarado do componente para PascalCase ou kebab-case e o compara com o nome do arquivo sem a extensão e sem outras transformações. Arquivos com várias definições de componentes não são verificados.",
    [note("vue/no-custom-modifiers-on-v-model")]:
      "Esta regra opcional aplica um contrato de compatibilidade com Vue 2 ao `v-model` de componentes. Apenas os modificadores `lazy`, `number` e `trim` são aceitos. Componentes do Vue 3 podem implementar modificadores personalizados, por isso as predefinições do Vue 3 não ativam esta restrição. Elementos nativos ficam fora desta verificação.",
    [note("vue/no-ref-object-reactivity-loss")]:
      "A verificação acompanha funções reconhecidas que criam refs e cópias de valores no mesmo escopo. Leituras dentro de outra função e escritas em `.value` não são verificadas. É uma verificação opcional baseada no código-fonte, sem necessidade de consultas nativas de tipos.",
    [note("vue/no-setup-props-reactivity-loss")]:
      "Verifica cópias de props no escopo raiz de `setup` da Options API e de `<script setup>`. A própria desestruturação reativa de props do Vue 3.5 é permitida; uma cópia posterior de uma prop desestruturada na raiz do script é reportada. Leituras dentro de outra função são permitidas.",
    [note("vue/no-v-for-template-key")]:
      "Esta é uma regra opcional de compatibilidade com Vue 2. O Vue 2 espera a key em um filho real; o Vue 3 a espera em `<template v-for>`. Para a convenção do Vue 3, use `vue/no-v-for-template-key-on-child`; não ative as duas convenções ao mesmo tempo.",
    [note("vue/prefer-prop-type-boolean-first")]:
      "Alterar a ordem dos construtores pode mudar a conversão booleana em tempo de execução. Esta verificação opcional abrange o `defineProps` em tempo de execução e as listas de tipos de props da Options API; ela não reordena uniões de tipos do TypeScript.",
    [note("vue/require-expose")]:
      "Esta verificação trata de objetos de componentes da Options API. Uma lista `expose` explicitamente vazia é válida. Chamar `expose` em setup ou retornar uma função de renderização também satisfaz a verificação; mixins são isentos. A passagem pelo script não inspeciona um bloco `<script setup>` irmão, e `defineExpose` não é o alvo desta regra.",
    [note("vue/valid-model-definition")]:
      "Esta regra opcional verifica a opção `model` do Vue 2. Apenas as chaves estáticas `prop` e `event` são aceitas; seus valores não são validados. O Vue 3 usa o contrato mais recente de `v-model` de componentes em vez dessa opção.",
  },
  fr: {
    "List component names whose rendered subtree always supplies accessible anchor content. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "Listez les composants dont le sous-arbre rendu fournit toujours un contenu de lien accessible. Les écritures PascalCase et kebab-case sont équivalentes. Les options n'activent pas la règle ; les couches suivantes remplacent la liste entière et [] efface les noms hérités.",
    "List component names that render a native form control and therefore satisfy label nesting. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "Listez les composants qui rendent un contrôle de formulaire natif et satisfont donc l'imbrication du label. Les écritures PascalCase et kebab-case sont équivalentes. Les options n'activent pas la règle ; les couches suivantes remplacent la liste entière et [] efface les noms hérités.",
    "The allowlist replaces eslint-plugin-vue's punctuation defaults when present. Entries are literal substrings or `/pattern/flags` values using Rust regex syntax and are removed before the remaining text is checked; patterns that do not compile remain literals. Omit allowlist to retain the defaults; [] clears them. Options do not enable the rule.":
      "Lorsqu'elle est présente, allowlist remplace les valeurs de ponctuation par défaut d'eslint-plugin-vue. Les entrées sont des sous-chaînes littérales ou des valeurs `/pattern/flags` utilisant la syntaxe regex de Rust, retirées avant la vérification du texte restant ; les motifs qui ne se compilent pas restent littéraux. Omettez allowlist pour conserver les valeurs par défaut ; [] les efface. Les options n'activent pas la règle.",
    "List exact globally registered directive names without the v- prefix. Local script-setup vName bindings are still discovered automatically. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.":
      "Listez les noms exacts des directives enregistrées globalement, sans le préfixe v-. Les bindings vName locaux de script-setup restent détectés automatiquement. Les options n'activent pas la règle ; les couches suivantes remplacent la liste entière et [] efface les noms hérités.",
    [note("vue/comment-directive")]:
      "Les commentaires de suppression inutilisés dans le template ne sont signalés que si `reportUnusedDisableDirectives: true` est activé. L'exemple supprime un commentaire inutile ; les commentaires qui suppriment réellement un diagnostic restent utiles. Les noms de règles listés ne comptent que lorsque Vize exécute ces règles.",
    [note("vue/html-end-tags")]:
      "Cette règle exige des balises de fermeture explicites là où le parseur HTML autorise leur omission, par exemple pour `li`, `p` et `td`. Les balises de fermeture obligatoires manquantes relèvent de `parser/template` ; les éléments vides n'ont pas besoin de balise de fermeture.",
    [note("vue/jsx-uses-vars")]:
      "Cette règle auxiliaire n'émet pas de diagnostic `vue/jsx-uses-vars`. Elle reconnaît les liaisons des balises JSX lorsque `vue/no-unused-components` détermine si un composant est utilisé. Activez les deux règles pour cet exemple. `<MyButton.Part />` fait reconnaître la liaison racine `MyButton` ; les balises avec espace de noms et les membres de `this` ne fournissent pas de liaison de composant importé.",
    [note("vue/match-component-file-name")]:
      "Le nom du fichier fait partie du contrôle. La comparaison par défaut convertit le nom déclaré du composant en PascalCase ou en kebab-case, puis le compare au nom du fichier sans extension et sans autre transformation. Les fichiers contenant plusieurs définitions de composants ne sont pas contrôlés.",
    [note("vue/no-custom-modifiers-on-v-model")]:
      "Cette règle à activer explicitement impose un contrat de compatibilité avec Vue 2 au `v-model` des composants. Seuls les modificateurs `lazy`, `number` et `trim` sont autorisés. Les composants Vue 3 peuvent implémenter des modificateurs personnalisés ; les préréglages Vue 3 n'activent donc pas cette restriction. Les éléments natifs sont exclus de ce contrôle.",
    [note("vue/no-ref-object-reactivity-loss")]:
      "Le contrôle suit les fonctions reconnues qui créent des refs et les copies de valeurs dans la même portée. Les lectures dans une autre fonction et les écritures dans `.value` ne sont pas contrôlées. Ce contrôle à activer explicitement analyse le code source et ne nécessite pas de requête de type native.",
    [note("vue/no-setup-props-reactivity-loss")]:
      "Contrôle les copies de props dans la portée racine de `setup` de l'Options API et de `<script setup>`. La déstructuration réactive des props de Vue 3.5 reste autorisée ; une copie ultérieure d'une prop déstructurée à la racine du script est signalée. Les lectures dans une autre fonction sont autorisées.",
    [note("vue/no-v-for-template-key")]:
      "Cette règle de compatibilité avec Vue 2 doit être activée explicitement. Vue 2 attend la key sur un enfant réel ; Vue 3 l'attend sur `<template v-for>`. Pour la convention Vue 3, utilisez `vue/no-v-for-template-key-on-child` ; n'activez pas les deux conventions ensemble.",
    [note("vue/prefer-prop-type-boolean-first")]:
      "Changer l'ordre des constructeurs peut modifier la conversion booléenne à l'exécution. Ce contrôle à activer explicitement couvre `defineProps` à l'exécution et les listes de types de props de l'Options API ; il ne réordonne pas les unions de types TypeScript.",
    [note("vue/require-expose")]:
      "Ce contrôle concerne les objets de composants de l'Options API. Une liste `expose` explicitement vide est valide. Un appel à `expose` dans setup ou le retour d'une fonction de rendu satisfait également le contrôle ; les mixins sont exclus. Cette passe sur le script n'inspecte pas un bloc `<script setup>` voisin, et `defineExpose` n'est pas la cible de cette règle.",
    [note("vue/valid-model-definition")]:
      "Cette règle à activer explicitement contrôle l'option `model` de Vue 2. Seules les clés statiques `prop` et `event` sont acceptées ; leurs valeurs ne sont pas validées. Vue 3 utilise le contrat plus récent de `v-model` des composants à la place de cette option.",
  },
};
