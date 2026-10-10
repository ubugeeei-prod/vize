export const portugueseRules2: Record<string, readonly [string, string, string]> = {
  "script/define-macros-order": [
    "Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt;",
    "`defineProps` aparece antes de `defineModel`, embora `defineModel` venha primeiro na ordem canônica das macros.",
    "As declarações seguem a sequência exata `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, `defineSlots`, antes de instruções não relacionadas em tempo de execução.",
  ],
  "script/define-props-declaration": [
    "Exigir defineProps&lt;{ ... }&gt;() baseado em tipos em vez da forma com objeto em tempo de execução",
    "`defineProps({ title: String })` fornece um objeto em tempo de execução, contrariando a preferência desta regra por props baseadas em tipos.",
    "`defineProps<{ title: string }>()` declara `title` no argumento de tipo e mantém o acesso a `props.title` sem um argumento de declaração em tempo de execução.",
  ],
  "script/define-props-destructuring": [
    "Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt;",
    "`defineProps` é atribuído à variável única `props` em vez de ser desestruturado, contrariando a preferência padrão pela desestruturação.",
    "O padrão de objeto vincula `foo` e `bar` diretamente e fornece um valor padrão para o `bar` opcional. Isso depende da desestruturação reativa de props do Vue 3.5+; o modo configurável `never` prefere a forma oposta.",
  ],
  "script/no-arrow-functions-in-watch": [
    "Proibir funções de seta como manipuladores de watch na Options API",
    "O observador `value` da Options API e o `other.handler` aninhado são funções de seta. Uma função de seta captura o `this` do contexto ao redor em vez de receber a instância do componente.",
    "Ambos os manipuladores passam a ser métodos comuns, permitindo que o Vue vincule `this` ao componente. A opção `deep: true` do observador continua compatível com a forma de objeto.",
  ],
  "script/no-async-in-computed": [
    "Proibir funções assíncronas em propriedades computadas",
    "O getter de `computed` é `async`, então a busca produz uma Promise em vez de um valor computado derivado de forma síncrona.",
    "A busca assíncrona é movida para `watch` e armazena seu resultado em `data.value`. A limpeza cancela a requisição anterior e impede que uma função de retorno inativa grave um resultado desatualizado; não resta nenhum getter computado assíncrono.",
  ],
  "script/no-boolean-default": [
    "Proibir um valor padrão em uma prop Boolean",
    "Tanto `disabled` quanto `checked` declaram um `default` em uma prop cujo único construtor é `Boolean`; a regra rejeita até mesmo um valor padrão `false` explícito.",
    "As props exclusivamente Boolean omitem `default`, usando o valor false implícito do Vue. A união `[Boolean, String]` e a prop Number ilustram que esta verificação se limita ao construtor único `Boolean`.",
  ],
  "script/no-deep-destructure-in-props": [
    "Proibir desestruturação profundamente aninhada em defineProps",
    "O padrão de vinculação percorre `user` para desestruturar `name`, ultrapassando a profundidade rasa padrão da desestruturação de props.",
    "O objeto de props permanece intacto, e um getter computado lê `props.user.name`. O acesso aninhado continua explícito sem um padrão de vinculação profundamente aninhado.",
  ],
  "script/no-deprecated-data-object-declaration": [
    "Proibir um literal de objeto como opção data do componente (o Vue 3 exige uma função)",
    "A opção `data` da Options API é um literal de objeto, uma forma do Vue 2 que o Vue 3 não aceita mais.",
    "`data()` retorna um novo objeto `{ count: 0 }`, fornecendo a declaração de data baseada em função exigida pelo Vue 3.",
  ],
  "script/no-deprecated-destroyed-lifecycle": [
    "Proibir os hooks de ciclo de vida obsoletos destroyed e beforeDestroy",
    "`beforeDestroy` é a opção de ciclo de vida removida do Vue 2 usada para limpar o temporizador.",
    "Renomear o hook para `beforeUnmount` preserva o corpo da limpeza com o nome de ciclo de vida do Vue 3.",
  ],
  "script/no-deprecated-dollar-listeners-api": [
    "Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs)",
    "As leituras de membros e a referência direta no argumento usam `$listeners`, que o Vue 3 removeu após incorporar os ouvintes aos atributos.",
    "As leituras passam a usar `this.$attrs` e `ctx.attrs` do contexto de setup. Essas formas substituem a API de ouvintes removida; os objetos receptores ilustrados precisam existir no contexto do componente ao redor.",
  ],
  "script/no-deprecated-dollar-scopedslots-api": [
    "Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots)",
    "`this.$scopedSlots`, `ctx.$scopedSlots` e a referência direta a `$scopedSlots` usam a API de slots com escopo do Vue 2 removida no Vue 3.",
    "Substituir `$scopedSlots` por `$slots` usa a API unificada de slots. O exemplo remove a grafia obsoleta em vez de estabelecer um contexto de setup para os objetos receptores.",
  ],
  "script/no-deprecated-events-api": [
    "Proibir a API de eventos do Vue 2 removida ($on / $off / $once)",
    "As chamadas de `$on`, `$once` e `$off` usam os métodos de barramento de eventos da instância removidos no Vue 3.",
    "`$emit` continua válido, enquanto a assinatura no barramento de eventos passa para o método `on` do emissor externo. A correção separa a emissão direcionada ao pai de um barramento de eventos externo.",
  ],
  "script/no-deprecated-props-default-this": [
    "Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3)",
    "O valor padrão e o validador da prop leem `this`, mas essas funções não podem depender da instância do componente no Vue 3.",
    "A função de valor padrão lê `props.baseSize` de seu argumento, e o validador testa seu argumento `value`. Ambos deixam de depender de um objeto receptor de instância indisponível.",
  ],
  "script/no-dupe-keys": [
    "Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API",
    "`foo` é declarado tanto em props quanto em data, e `bar` tanto em computed quanto em methods. Essas declarações disputam as mesmas chaves na instância do componente.",
    "As declarações de prop, data e computed usam nomes distintos (`foo`, `bar` e `baz`), eliminando ambas as colisões entre opções.",
  ],
  "script/no-duplicate-attr-inheritance": [
    "Sinalizar um componente que aplica duas vezes seus atributos repassados",
    "Os valores explícitos `inheritAttrs: true` repetem o padrão do Vue. Esta regra sinaliza esse literal redundante mesmo quando não há expansão de `$attrs` na raiz.",
    "`inheritAttrs: false` expressa uma desativação efetiva, enquanto o objeto vazio de opções deixa implícita a herança padrão. Nenhuma das formas repete o valor redundante `true`.",
  ],
  "script/no-export-in-script-setup": [
    "Proibir instruções export dentro de &lt;script setup&gt;",
    "`export const count` tenta expor uma exportação de módulo a partir de `<script setup>`, onde exportações em tempo de execução são proibidas.",
    "Remover `export` mantém `count` como uma variável de setup em vez de uma exportação de módulo.",
  ],
  "script/no-get-current-instance": [
    "Proibir getCurrentInstance() no modo Vapor (retorna null)",
    "O setup marcado como Vapor importa e chama `getCurrentInstance`, dependendo de uma API de instância que esta regra proíbe para componentes orientados a Vapor.",
    '`inject("app-config")` obtém a configuração fornecida explicitamente sem importar nem chamar `getCurrentInstance`.',
  ],
  "script/no-import-compiler-macros": [
    "Proibir a importação de macros do compilador Vue que são importadas automaticamente",
    "A importação de `vue` inclui `defineProps` e `defineEmits`, embora sejam macros do compilador disponíveis diretamente em `<script setup>`.",
    "Remover as importações das macros mantém ambas as chamadas tipadas intactas; nenhuma das declarações precisa de uma importação em tempo de execução.",
  ],
  "script/no-internal-imports": [
    "Proibir importações de módulos internos do Vue",
    "Ambas as importações apontam para arquivos internos de `dist` em vez do ponto de entrada público do pacote Vue, vinculando o componente aos caminhos dos arquivos de compilação.",
    "Importar as funções auxiliares necessárias de `vue` remove a dependência dos locais dos arquivos internos de distribuição.",
  ],
  "script/no-multiple-slot-args": [
    "Proibir passar mais de um argumento a uma chamada de função de slot com escopo",
    "As chamadas de slot passam vários argumentos posicionais ou expandem uma lista desconhecida de argumentos. Os slots do Vue recebem um único objeto de props, não uma lista de parâmetros posicionais.",
    "`{ foo, bar }` combina os dados em um único argumento; `slotProps` e a chamada sem argumentos também respeitam a forma de chamada de slot suportada.",
  ],
  "script/no-next-tick": [
    "Proibir o uso de nextTick() em componentes orientados a Vapor",
    "O componente orientado a Vapor importa e aguarda `nextTick`, introduzindo a dependência do agendamento da atualização do DOM que esta regra de migração rejeita.",
    "O input é obtido por `useTemplateRef` e recebe foco em `onMounted`. O momento explícito de montagem substitui a dependência de `nextTick` do exemplo.",
  ],
  "script/no-options-api": [
    "Proibir padrões da Options API no modo Vapor",
    "O objeto da exportação padrão declara `data()` da Options API, uma forma de opção de componente proibida por esta regra.",
    "O estado do componente passa a ser um `ref` da Composition API em `<script setup>` com Vapor, removendo o objeto da Options API e sua opção `data`.",
  ],
  "script/no-potential-component-option-typo": [
    "Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API",
    "A opção está escrita como `method`, a uma edição de distância da opção reconhecida `methods`; o Vue não a trataria como a declaração de métodos pretendida.",
    "Alterar a chave para `methods` coloca `save()` dentro da opção de componente reconhecida.",
  ],
  "script/no-reactive-destructure": [
    "Proibir a desestruturação de objetos reativos que causa perda de reatividade",
    "`const { count, name } = state` copia propriedades primitivas do objeto `reactive`, perdendo sua conexão com alterações posteriores nas propriedades.",
    "Desestruturar `toRefs(state)` cria refs para `count` e `name`, mantendo cada variável vinculada à propriedade reativa original.",
  ],
  "script/no-ref-as-operand": [
    "Exigir que variáveis vinculadas a refs sejam acessadas por `.value` quando usadas como operandos",
    "`count + 1` usa o próprio objeto ref como operando aritmético em vez do número que ele encapsula.",
    "`count.value + 1` lê o número encapsulado antes de somar um; a aritmética no script exige esse acesso explícito ao ref.",
  ],
  "script/no-required-prop-with-default": [
    "Proibir uma prop que tenha required: true e também um valor padrão",
    '`title` é obrigatório e recebe o valor alternativo `"Untitled"`, combinando um contrato de entrada obrigatória com um valor padrão destinado à ausência de entrada.',
    'Remover `required: true` torna `title` opcional e mantém `"Untitled"` como seu valor alternativo coerente.',
  ],
  "script/no-reserved-identifiers": [
    "Proibir o uso de identificadores reservados pelo compilador Vue",
    "As variáveis `__props`, `__emit` e `__sfc__` usam identificadores reservados para código gerado pelo compilador Vue.",
    "Os nomes comuns `props`, `emit` e `componentData` evitam esses identificadores gerados enquanto mantêm as declarações de props e emits.",
  ],
  "script/no-reserved-keys": [
    "Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API",
    "A chave de data retornada, `$el`, colide com a propriedade nativa da instância do componente Vue e também usa um prefixo `$` reservado.",
    "Renomear o dado da aplicação para `elementLabel` evita a API nativa da instância e o prefixo reservado.",
  ],
  "script/no-reserved-props": [
    "Proibir nomes reservados na declaração de props de um componente",
    "`ref` e `$foo` na forma de objeto, além de `key` na forma de array, são nomes reservados de props. `ref` e `key` são controles do framework, e nomes com prefixo `$` são rejeitados.",
    "Os nomes comuns de props `name` e `refValue` evitam os nomes reservados tanto na grafia quanto no prefixo.",
  ],
  "script/no-restricted-globals": [
    "Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado",
    "O exemplo lê diretamente as variáveis globais restritas por padrão `process`, `localStorage` e `sessionStorage`, ignorando as funções auxiliares explícitas de configuração e armazenamento do projeto.",
    "`useFeatureFlag`, `authStorage.read` e `viewStorage.write` removem essas referências diretas às variáveis globais restritas. O `window.scrollY` restante não é uma restrição padrão desta regra; a segurança em SSR é uma questão separada.",
  ],
  "script/no-restricted-members": [
    "Proibir acessos a membros object.property configurados pelo projeto",
    'Com `{ object: "window", property: "localStorage" }` configurado em `ruleOptions`, `window.localStorage` acessa o par objeto/membro proibido. Esta regra não tem membros proibidos por padrão.',
    '`authStorage.read("token")` delega a leitura à função auxiliar de armazenamento da aplicação e deixa de acessar o membro configurado `window.localStorage`.',
  ],
  "script/no-side-effects-in-computed-properties": [
    "Proibir efeitos colaterais em getters computados da Options API",
    "`doubled` atribui um valor a `this.count`, e `reversed` modifica `this.items` por meio de `reverse()`. Ambos os getters alteram o estado do qual deveriam derivar seus valores.",
    "`doubled` retorna a multiplicação sem atribuição. `reversed` copia o array antes de invertê-lo, então o getter não altera o estado original do componente.",
  ],
  "script/no-top-level-ref-in-script": [
    "Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições",
    "O `<script>` comum inicializa `count` e `user` no escopo do módulo. Durante SSR, esses objetos de estado podem ser compartilhados entre instâncias de componentes e requisições.",
    "O ref de setup é inicializado por instância de componente; o script comum mantém apenas uma constante, uma função que produz estado e um ref criado dentro de `setup()`. Nenhuma dessas formas cria estado reativo no escopo do módulo comum.",
  ],
  "script/no-unstable-nested-components": [
    "Proibir definições de componentes dentro de funções de setup ou renderização",
    "`defineComponent` é executado dentro do `setup()` do pai, criando uma nova definição do componente `Child` sempre que esse setup é executado.",
    "A definição de `Child` é movida para o escopo do módulo, e `setup()` retorna essa definição existente em vez de recriá-la.",
  ],
  "script/no-unused-emit-declarations": [
    "Sinalizar eventos declarados que nunca são emitidos",
    "`defineEmits` declara tanto `change` quanto `unused`, mas a função `emit` capturada só emite o evento literal `change`.",
    "Remover `unused` faz a lista de eventos declarados corresponder à emissão observada. O exemplo usa uma variável emit capturada que não escapa do escopo, permitindo essa conclusão sobre o uso local.",
  ],
  "script/no-use-computed-property-like-method": [
    "Proibir chamar uma propriedade computada da Options API como um método",
    "`this.total()` chama o valor exposto pelo getter computado; o getter retorna `3`, que não pode ser chamado.",
    "`this.total` lê o valor computado sem parênteses de chamada, então `log` imprime o número derivado.",
  ],
  "script/no-with-defaults": [
    "Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+)",
    "`withDefaults` envolve a declaração tipada de props apenas para fornecer valores padrão a `count` e `name`, em vez do estilo de valores padrão na desestruturação do Vue 3.5+ preferido aqui.",
    'O padrão de desestruturação coloca `count = 0` e `name = "Ada"` junto às suas variáveis e remove o encapsulamento de `withDefaults`.',
  ],
};
