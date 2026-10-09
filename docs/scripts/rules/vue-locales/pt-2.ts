export const portugueseVue2: Record<string, readonly [purpose: string, bad: string, good: string]> =
  {
    "vue/require-component-is": [
      "Exigir `v-bind:is` em elementos `<component>`",
      "O `<component>` dinâmico não tem um destino `is`, então o Vue não pode escolher um componente para renderizar.",
      '`:is="currentComponent"` fornece a seleção do componente; a vinculação pode mudar durante a execução.',
    ],
    "vue/require-component-registration": [
      "Exigir importação ou registro explícito de componentes",
      "`MissingWidget` não está registrado nem incluído na lista configurada de componentes globais permitidos.",
      "`MyButton` consta na opção `globals` do exemplo. Essa opção isenta um componente global conhecido; ela não o registra nem o importa.",
    ],
    "vue/require-scoped-style": [
      "Exigir o atributo scoped nas tags style",
      "O estilo `.button` não tem escopo e pode afetar elementos correspondentes fora deste componente.",
      "Adicionar `scoped` aplica o escopo de componente do Vue ao mesmo seletor e às mesmas declarações.",
    ],
    "vue/require-toggle-inside-transition": [
      "Exigir uma alternância no elemento envolvido por `<transition>`",
      "O filho estático dentro de `<Transition>` não tem visibilidade condicional nem seleção dinâmica para provocar uma mudança de entrada ou saída.",
      '`v-if="show"` altera a existência do filho, fornecendo um limite de entrada ou saída à transição.',
    ],
    "vue/require-v-for-key": [
      "Exigir `v-bind:key` nas diretivas `v-for`",
      "Cada `<li>` repetido não tem uma chave que identifique seu item correspondente durante atualizações da lista.",
      '`:key="item.id"` dá a cada nó repetido a identidade do item, em vez de sua posição atual.',
    ],
    "vue/scoped-event-names": [
      "Recomendar nomes de eventos com escopo no formato context:event",
      "`playAudio`, `pauseAudio` e `reloadAudio` codificam seu escopo como sufixos camelCase, em vez de seguir a convenção de eventos separados por dois-pontos da regra.",
      "`audio:play`, `audio:pause` e `audio:reload` compartilham um escopo explícito `audio:`. O componente emissor deve usar os mesmos nomes.",
    ],
    "vue/sfc-element-order": [
      "Aplicar uma ordem consistente aos elementos de nível superior dos SFCs",
      "O bloco style precede o bloco script, contrariando a ordem configurada dos blocos do SFC.",
      "Os blocos seguem script → template → style. Projetos podem escolher outra ordem pela opção tipada desta regra.",
    ],
    "vue/single-style-block": [
      "Recomendar um único bloco style",
      "O componente divide seus estilos com escopo de panel e title entre dois blocos style.",
      "Os dois seletores mantêm o escopo em um único bloco style, atendendo à convenção de bloco único sem remover nenhum estilo.",
    ],
    "vue/slot-name-casing": [
      "Exigir kebab-case nos slots nomeados usados por v-slot",
      "O slot nomeado `mySlot` usa camelCase onde a regra exige um nome separado por hífen.",
      "`#my-slot` usa kebab-case. Renomeie o ponto de inserção do slot correspondente para o mesmo nome.",
    ],
    "vue/this-in-template": [
      "Proibir `this.` nas expressões do template",
      "As expressões do template acessam explicitamente `this.message`, `this.className` e `this.handleClick`, embora o Vue exponha essas variáveis diretamente.",
      "Use `message`, `className` e `handleClick` diretamente. A string literal `'this.is.a.string'` permanece igual porque não é um acesso a membro.",
    ],
    "vue/use-unique-element-ids": [
      "Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos",
      "O ID literal `email` é reutilizado por todas as instâncias deste componente, o que pode direcionar seu rótulo ao elemento errado quando várias instâncias são renderizadas.",
      "`useId()` produz o `emailId` da instância; vincule o mesmo valor ao `for` do rótulo e ao `id` do input.",
    ],
    "vue/use-v-on-exact": [
      "Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores",
      "O manipulador de clique comum também pode executar em Ctrl-clique, sobrepondo-se ao manipulador separado `.ctrl`.",
      "`.exact` limita o manipulador de clique comum a cliques sem teclas modificadoras; o manipulador específico de Ctrl permanece separado.",
    ],
    "vue/v-bind-style": [
      "Aplicar um padrão de sintaxe à diretiva `v-bind`",
      "`v-bind:class` usa a forma longa onde o padrão de vinculação configurado exige a forma abreviada com dois-pontos.",
      "`:class` mantém a mesma expressão com a forma abreviada exigida; esta regra trata da grafia, e não do tipo do valor.",
    ],
    "vue/v-on-event-hyphenation": [
      "Exigir hífens nos nomes de eventos personalizados em v-on de componentes",
      "O listener do componente personalizado usa `@myEvent` em vez de um nome de evento separado por hífen.",
      "`@my-event` usa a grafia exigida para eventos personalizados. Os listeners em elementos nativos e os argumentos de evento dinâmicos mostrados abaixo ficam fora desta verificação.",
    ],
    "vue/v-on-handler-style": [
      "Exigir manipuladores de v-on escritos como referência a método ou função inline",
      "Os manipuladores colocam mutações e várias instruções diretamente no atributo de evento.",
      "Use uma referência a manipulador ou uma expressão de função, comum ou de seta, quando for necessária lógica inline. O limite da função torna explícita a forma do manipulador.",
    ],
    "vue/v-on-style": [
      "Aplicar um padrão de sintaxe à diretiva `v-on`",
      "`v-on:click` usa a forma longa de listener de evento onde a regra exige a forma abreviada.",
      "`@click` mantém o mesmo manipulador e usa a forma abreviada configurada.",
    ],
    "vue/v-slot-style": [
      "Aplicar um padrão de sintaxe à diretiva `v-slot`",
      "O componente usa `#default` e o template usa `v-slot:header`, contrariando os padrões da regra para cada contexto.",
      "Use `v-slot` para o slot padrão do componente e `#header` para o slot nomeado do template.",
    ],
    "vue/valid-attribute-name": [
      "Exigir nomes de atributos válidos",
      'A aspa dentro de `my"attr` torna o nome do atributo malformado. Este exemplo produz o diagnóstico `parser/template` do parser, sem prometer um diagnóstico separado da regra.',
      "`my-attr` é um nome de atributo bem formado, então o parser do template pode ler o atributo e seu valor.",
    ],
    "vue/valid-template-root": [
      "Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3",
      "Um `<template>` aninhado comum ocupa a raiz do template sem uma diretiva que lhe dê uma função de renderização.",
      "A `<div>` é um elemento raiz renderizável. Este exemplo não impõe uma restrição universal de raiz única aos fragmentos do Vue 3.",
    ],
    "vue/valid-v-bind": [
      "Exigir diretivas `v-bind` válidas",
      "O `v-bind` sem argumento não tem uma expressão de objeto, e a forma com argumento vazio não tem um nome de atributo.",
      "Forneça um atributo e uma expressão, vincule um objeto ou use a forma abreviada de mesmo nome do Vue 3.4+, como `:loading`.",
    ],
    "vue/valid-v-cloak": [
      "Exigir diretivas `v-cloak` válidas",
      "`v-cloak` recebe um valor, argumento ou modificador, embora não aceite nenhum deles.",
      "Use `v-cloak` sozinho; CSS pode ocultar o elemento até que o Vue remova esse atributo após a montagem.",
    ],
    "vue/valid-v-else": [
      "Exigir diretivas `v-else` válidas",
      "Os exemplos atribuem uma expressão a `v-else`, combinam-no com `v-if` ou omitem o ramo condicional adjacente que deve precedê-lo.",
      "Coloque `v-else` sozinho imediatamente após o ramo `v-if` correspondente.",
    ],
    "vue/valid-v-for": [
      "Exigir diretivas `v-for` válidas",
      "Os laços omitem a expressão de iteração ou adicionam o modificador `.stop`, que não é aceito.",
      "Use `item in items` ou `(item, index) of items` com uma expressão de iteração completa e as chaves mostradas.",
    ],
    "vue/valid-v-html": [
      "Exigir diretivas `v-html` válidas",
      "`v-html` não tem sua expressão ou usa um argumento ou modificador que esta diretiva não aceita.",
      '`v-html="html"` fornece uma expressão válida. A validade sintática não sanitiza HTML nem torna seguro conteúdo não confiável.',
    ],
    "vue/valid-v-if": [
      "Exigir diretivas `v-if` válidas",
      "As condições omitem uma expressão ou combinam `v-if` com uma diretiva else no mesmo nó.",
      "Cada `v-if` tem uma condição não vazia, como `ready` ou `count > 0`, sem uma diretiva else incompatível.",
    ],
    "vue/valid-v-memo": [
      "Exigir diretivas `v-memo` válidas",
      "`v-memo` sozinho não fornece ao Vue uma expressão de dependências para decidir quando reutilizar a subárvore.",
      '`v-memo="[valueA, valueB]"` fornece o array de dependências usado para memorização.',
    ],
    "vue/valid-v-model": [
      "Exigir diretivas `v-model` válidas",
      "Uma `<div>` nativa não pode usar `v-model` como controle de formulário, e uma diretiva de input sem valor não tem uma expressão de destino gravável.",
      "Vincule o input, select, textarea ou componente personalizado às variáveis graváveis mostradas.",
    ],
    "vue/valid-v-on": [
      "Exigir diretivas `v-on` válidas",
      "As formas de listener omitem um argumento de evento ou a expressão obrigatória de manipulador ou objeto.",
      "Use um evento com seu manipulador ou passe um objeto de listeners para `v-on` sem argumento.",
    ],
    "vue/valid-v-once": [
      "Exigir diretivas `v-once` válidas",
      "`v-once` tem um valor, argumento ou modificador, embora esta diretiva seja um marcador sem valor para renderização única.",
      "`v-once` sozinho marca a subárvore para renderização única, sem sintaxe não suportada.",
    ],
    "vue/valid-v-show": [
      "Exigir diretivas `v-show` válidas",
      "`v-show` não tem sua expressão de visibilidade ou é colocado em um `<template>` que não tem um elemento DOM cujo display possa ser alterado.",
      "Aplique a expressão de visibilidade a um elemento renderizado, como `<div>`.",
    ],
    "vue/valid-v-slot": [
      "Exigir diretivas `v-slot` válidas",
      "A diretiva de slot está em uma `<div>` nativa ou entra em conflito com outras declarações de slots padrão ou nomeados.",
      "Declare o slot padrão de um componente nele próprio, ou seu slot nomeado em um filho `<template #header>`.",
    ],
    "vue/valid-v-text": [
      "Exigir diretivas `v-text` válidas",
      "`v-text` não tem sua expressão de texto ou usa um argumento ou modificador que não é aceito.",
      '`v-text="msg"` é sintaticamente válido. A regra de estilo separada `vue/no-v-text` ainda pode preferir interpolação.',
    ],
    "vue/warn-custom-block": [
      "Alertar sobre blocos personalizados em arquivos SFC",
      "O SFC contém um bloco personalizado `<i18n>`, que precisa de uma integração externa além do processamento comum de template, script e style.",
      "O exemplo usa blocos padrão de template e script setup. Este aviso opcional de portabilidade não significa que todo bloco personalizado seja inválido no Vue.",
    ],
    "vue/warn-custom-directive": [
      "Alertar sobre diretivas personalizadas que precisam de registro",
      "`v-focus`, `v-mask` e `v-click-outside` exigem implementações de diretivas específicas do projeto, que esta convenção opcional sinaliza.",
      "O exemplo usa as diretivas nativas `v-if`, `v-model` e `v-on`. Uma diretiva personalizada registrada corretamente ainda pode ser válida no Vue quando esta política estiver desativada.",
    ],
  };

export const portugueseVueNotes: Record<string, string> = {
  "The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.":
    "O nome de arquivo do componente é verificado. PascalCase e kebab-case são aceitos; o uso misturado de maiúsculas e minúsculas é reportado.",
  "Bad has cyclomatic complexity 13 and cognitive complexity 25 (limits: 11 and 16). Each component is measured separately; only inline HTML templates are supported.":
    "Bad tem complexidade ciclomática 13 e complexidade cognitiva 25 (limites: 11 e 16). Cada componente é medido separadamente; apenas templates HTML inline são suportados.",
  "See [complexity scoring and component boundaries](../../guide/cross-file-complexity.md) for the contributions behind the example's two scores.":
    "Veja [o cálculo da complexidade e os limites dos componentes](../../guide/cross-file-complexity.md) para conhecer as contribuições às duas pontuações do exemplo.",
  "The filename is the finding. Rename the same component; changing a child tag does not fix it.":
    "A ocorrência diagnosticada é o nome de arquivo. Renomeie o mesmo componente; alterar uma tag filha não corrige o problema.",
  "Enable only for a single-root contract. Vue 3 normally supports fragments.":
    "Ative apenas quando houver um contrato de raiz única. O Vue 3 normalmente suporta fragmentos.",
  "This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.":
    "Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.",
  "The current check compares nested v-for bindings. It does not report a single v-for binding merely because it shares a script binding's name.":
    "A verificação atual compara variáveis de v-for aninhados. Ela não reporta uma variável de um único v-for apenas porque compartilha o nome de uma variável do script.",
  "Checks declared prop names, not the casing of attributes passed to a child.":
    "Verifica os nomes de props declaradas, e não o padrão de maiúsculas e minúsculas dos atributos passados a um filho.",
  "List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.":
    "Liste nomes explícitos de componentes fornecidos por plugins da aplicação ou pelo previewSetup do Musea. Grafias PascalCase e kebab-case são aceitas; expressões regulares não são interpretadas. As opções não ativam a regra. Camadas posteriores substituem a lista; uma lista vazia remove os nomes herdados.",
  "Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.":
    "A grafia malformada de um atributo é diagnosticada por parser/template antes que esta regra defensiva veja um atributo. Portanto, Bad reporta parser/template; não promete uma ocorrência separada de vue/valid-attribute-name.",
};
