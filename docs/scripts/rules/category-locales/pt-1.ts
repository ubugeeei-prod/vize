export const portugueseRules1: Record<string, readonly [string, string, string]> = {
  "css/prefer-nested-selectors": [
    "Recomendar o aninhamento de CSS para seletores de descendentes",
    "O seletor de descendentes `.card .title` repete o seletor pai em uma regra sem aninhamento.",
    "A regra `.title` é aninhada dentro de `.card`, mantendo junta a relação de estilo entre pai e filho.",
  ],
  "css/prefer-slotted": [
    "Recomendar ::v-slotted() para estilizar o conteúdo de slots",
    "A folha de estilos com escopo tem como alvo o ponto de inserção `slot`, em vez dos elementos fornecidos pelo slot.",
    "`:slotted(.label)` tem como alvo o elemento de rótulo fornecido, por meio do seletor de slot com escopo.",
  ],
  "css/require-font-display": [
    "Exigir font-display nas regras @font-face",
    "A declaração font-face define a origem da fonte, mas omite sua política font-display.",
    "`font-display: swap` seleciona explicitamente a política de exibir uma fonte substituta e depois a fonte carregada.",
  ],
  "ecosystem/nuxt-prefer-nuxt-link": [
    "Preferir NuxtLink para links internos da aplicação",
    "O destino interno de configurações usa uma âncora comum em uma aplicação Nuxt.",
    "NuxtLink trata o mesmo destino interno por meio do roteador do Nuxt.",
  ],
  "ecosystem/pinia-prefer-store-to-refs": [
    "Preferir storeToRefs() ao desestruturar stores do Pinia",
    "Desestruturar `name` diretamente do store separa o valor de seu acesso reativo ao store.",
    "O store permanece intacto e storeToRefs cria uma referência reativa para name.",
  ],
  "ecosystem/router-link-require-to": [
    "Exigir um destino `to` nos componentes RouterLink e NuxtLink",
    "O RouterLink aninhado não tem um destino `to`; ele não pode depender da propagação de atributos do elemento raiz.",
    '`to="/settings"` fornece explicitamente o destino do link aninhado.',
  ],
  "ecosystem/void-link-require-href": [
    "Exigir `href` nos componentes Link do Void Vue",
    "O Link importado de @void/vue omite seu destino href.",
    "O mesmo Link importado recebe o destino de configurações por meio de href.",
  ],
  "ecosystem/void-link-valid-method": [
    "Validar props method estáticas do Link do Void Vue",
    "A ação DELETE solicita pré-busca, embora a pré-busca seja destinada a solicitações de navegação.",
    "Remover prefetch mantém a ação DELETE sem fazer a pré-busca dessa solicitação que não é GET.",
  ],
  "ecosystem/vue-i18n-no-missing-key": [
    "Relatar chaves estáticas do vue-i18n ausentes nas mensagens locais do SFC",
    "O template solicita auth.missing, mas as mensagens locais em inglês declaram apenas auth.login.",
    "O template solicita a chave auth.login, que existe nas mensagens locais.",
  ],
  "ecosystem/vue-router-prefer-named-link": [
    "Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink",
    "O destino do RouterLink é um caminho literal, em vez de uma rota nomeada.",
    "O objeto de rota vinculado identifica o destino pelo nome de rota settings.",
  ],
  "ecosystem/vue-router-prefer-named-push": [
    "Preferir objetos de rotas nomeadas para navegação programática com Vue Router",
    "router.push recebe uma string de caminho vinculada à grafia atual da URL.",
    "router.push recebe um objeto de rota com o nome estável settings.",
  ],
  "ecosystem/vue-test-utils-no-html-snapshot": [
    "Evitar snapshots de wrapper.html() nos testes com Vue Test Utils",
    "A asserção registra um snapshot de todo o HTML do wrapper, em vez de verificar o comportamento esperado.",
    "A asserção verifica se o texto renderizado contém Saved.",
  ],
  "html/deprecated-attr": [
    "Proibir atributos HTML obsoletos",
    "O parágrafo usa o atributo de apresentação obsoleto `align`.",
    "A classe e a declaração `text-align: center` expressam o alinhamento por meio de CSS.",
  ],
  "html/deprecated-element": [
    "Proibir elementos HTML obsoletos",
    "O elemento `center` usa um elemento de apresentação HTML obsoleto.",
    "Uma seção e uma classe de estilo substituem o elemento obsoleto, preservando o conteúdo.",
  ],
  "html/id-duplication": [
    "Proibir IDs de elementos duplicados",
    'Tanto o campo quanto o parágrafo de ajuda declaram `id="email"`, tornando ambíguo o destino do rótulo.',
    "O campo mantém `email`; o parágrafo de ajuda usa `email-help`, e aria-describedby faz referência a esse ID distinto.",
  ],
  "html/no-consecutive-br": [
    "Proibir elementos &lt;br&gt; consecutivos",
    "Dois elementos de quebra consecutivos criam espaçamento entre blocos dentro de um único parágrafo.",
    "Parágrafos separados expressam os dois blocos de conteúdo sem repetir elementos de quebra.",
  ],
  "html/no-dupe-style-properties": [
    "Proibir propriedades duplicadas em atributos de estilo em linha",
    "Cada estilo estático repete uma propriedade; `margin` e `MARGIN` também contam como a mesma propriedade.",
    "O estilo estático usa propriedades distintas de cor e plano de fundo. Vinculações dinâmicas de estilo ficam fora desta verificação de atributos estáticos.",
  ],
  "html/no-duplicate-class": [
    "Proibir nomes de classe duplicados em um atributo class estático",
    "A lista de classes estática repete o token `btn`.",
    "A lista de classes mantém um token `btn` e o token distinto `primary`.",
  ],
  "html/no-duplicate-dt": [
    "Proibir nomes &lt;dt&gt; duplicados em &lt;dl&gt;",
    "A mesma lista de definições repete o termo `API` para duas descrições.",
    "Um único termo API é seguido pelas duas descrições, evitando a repetição do termo.",
  ],
  "html/no-empty-palpable-content": [
    "Proibir elementos vazios que esperam conteúdo visível",
    "O parágrafo, o item de lista e a célula da tabela têm conteúdo perceptível vazio.",
    "O texto preenche o parágrafo, a interpolação fornece o conteúdo do item de lista e aria-label dá um nome explícito à célula que, de outra forma, estaria vazia.",
  ],
  "html/require-datetime": [
    "Exigir o atributo datetime no elemento &lt;time&gt;",
    "O elemento time contém uma data legível por pessoas, mas não tem um valor datetime legível por máquinas.",
    '`datetime="2026-05-13"` fornece a data correspondente em um formato legível por máquinas.',
  ],
  "musea/no-empty-variant": [
    "Proibir blocos &lt;variant&gt; vazios",
    "A variante chamada primary está vazia, então não fornece conteúdo para a prévia.",
    "A variante renderiza um Button primary com seu conteúdo Save.",
  ],
  "musea/prefer-design-tokens": [
    "Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente",
    "O exemplo art usa a cor azul literal, em vez do token de design primary configurado.",
    "O estilo faz referência a --color-primary, o token configurado para este exemplo.",
  ],
  "musea/require-component": [
    "Exigir o atributo component no bloco &lt;art&gt;",
    "O bloco art fornece um título, mas não identifica o componente exibido na prévia.",
    "defineArt fornece ./Button.vue como o componente do bloco art.",
  ],
  "musea/require-title": [
    "Exigir o atributo title no bloco &lt;art&gt;",
    "O bloco art identifica Button.vue, mas não fornece um título.",
    "As opções de defineArt fornecem o título Button para o bloco art.",
  ],
  "musea/unique-variant-names": [
    "Exigir nomes de variante únicos",
    "Duas variantes no mesmo bloco art usam o nome primary.",
    "As variantes têm os nomes distintos primary e secondary.",
  ],
  "musea/valid-variant": [
    "Exigir o atributo name nos blocos &lt;variant&gt;",
    "A variante omite o nome necessário para identificar a prévia.",
    "O nome primary identifica essa variante.",
  ],
  "nuxt/no-nuxt-config-test-key": [
    "Proibir a definição da chave `test` na configuração do Nuxt",
    "A configuração exportada do Nuxt define a chave identificadora `test` como o booleano `true`, a estrutura de configuração obsoleta que esta regra rejeita.",
    "A configuração vazia remove essa propriedade booleana `test`. Este exemplo não proíbe um objeto de configuração de testes.",
  ],
  "nuxt/no-page-meta-runtime-values": [
    "Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente",
    "`useRoute()` é avaliado imediatamente durante a construção do objeto `definePageMeta`, embora a macro eleve esses metadados para fora do contexto de execução de setup.",
    "`validate` recebe uma função de callback, então o acesso a `useRoute().params.id` é adiado até a execução dessa função. A regra distingue corpos de funções com execução adiada de valores de metadados avaliados imediatamente.",
  ],
  "nuxt/nuxt-config-keys-order": [
    "Preferir a ordem recomendada das propriedades de configuração do Nuxt",
    "A configuração coloca `ssr` antes de `modules`, invertendo sua ordem na sequência de chaves de configuração do Nuxt recomendada pela regra.",
    "Colocar `modules` antes de `ssr` preserva os dois valores e atende à ordem prescrita; a correção muda a disposição, sem alterar o significado de nenhuma das opções.",
  ],
  "nuxt/prefer-import-meta": [
    "Preferir `import.meta.*` a `process.*`",
    "`process.client` usa um indicador de ambiente legado do Nuxt que a regra pede para migrar para `import.meta`.",
    "`import.meta.client` mantém explícito o ramo exclusivo do navegador usando o indicador de ambiente substituto.",
  ],
  "petite-vue/no-unsupported-directive": [
    "Proibir diretivas que o petite-vue não aceita",
    "`v-memo`, `v-slot:header` e a diretiva personalizada `v-my-directive` não constam na lista de diretivas aceitas pelo petite-vue. O script do petite-vue identifica este HTML como o dialeto pertinente.",
    "A substituição usa a sintaxe aceita de `v-scope`, `v-effect`, `v-if`, `v-bind` e `v-on`, em vez de depender de diretivas não aceitas.",
  ],
  "petite-vue/valid-v-effect": [
    "Exigir uma expressão não vazia em v-effect",
    "Cada `v-effect` não tem uma expressão executável: seu valor está ausente, vazio ou contém apenas espaços em branco.",
    "Ambos os valores de `v-effect` contêm uma expressão: um atualiza `el.textContent` e o outro incrementa `count`. Esta regra verifica se há uma expressão não vazia, e não a lógica de negócio do efeito.",
  ],
  "petite-vue/valid-v-scope": [
    "Exigir que v-scope vincule um objeto literal",
    "Os quatro valores não vazios de `v-scope` são um identificador, uma chamada, uma operação aritmética e um número; nenhum é analisado como um objeto literal.",
    "Um `v-scope` sem valor usa o escopo raiz. Os outros valores são objetos literais, incluindo o objeto entre parênteses, que a regra aceita.",
  ],
  "script/component-options-name-casing": [
    "Exigir PascalCase na opção `name` do componente",
    "A opção de componente `name: 'my-component'` está em kebab-case, enquanto esta regra exige um nome de componente literal em PascalCase.",
    "`MyComponent` começa com uma letra maiúscula e contém apenas caracteres alfanuméricos, atendendo à verificação de nome.",
  ],
  "script/custom-event-name-casing": [
    "Exigir camelCase nos nomes de eventos personalizados emitidos",
    "A string emitida `my-event` contém um hífen e viola a política padrão de nomes de eventos em camelCase.",
    "Tanto a declaração quanto a chamada usam `myEvent`, preservando a correspondência entre o nome do evento e sua emissão e atendendo à política padrão de maiúsculas e minúsculas. Uma política configurada de kebab-case tem uma expectativa diferente.",
  ],
  "script/define-emits-declaration": [
    "Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array",
    '`defineEmits(["change"])` usa uma declaração de array em tempo de execução; esta regra de estilo prefere uma declaração baseada em tipos.',
    '`defineEmits<{ change: [id: number] }>()` move a declaração de evento para um argumento de tipo e descreve explicitamente o dado numérico usado por `emit("change", 1)`.',
  ],
};
