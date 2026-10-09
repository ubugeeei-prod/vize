export const portugueseVue0: Record<string, readonly [purpose: string, bad: string, good: string]> =
  {
    "vue/a11y-img-alt": [
      "Exigir o atributo alt nas imagens para garantir acessibilidade",
      "Nem a imagem estática nem a imagem de origem dinâmica fornecem um atributo alt.",
      "Imagens informativas recebem um texto alt descritivo; imagens decorativas recebem um alt vazio; a imagem dinâmica vincula sua descrição.",
    ],
    "vue/attribute-hyphenation": [
      "Aplicar um padrão de nomes de atributos em componentes personalizados",
      "O atributo do componente usa a grafia camelCase firstName.",
      "A grafia first-name segue a convenção configurada de atributos de componente separados por hífen.",
    ],
    "vue/attribute-order": [
      "Aplicar uma ordem consistente aos atributos",
      "O manipulador de evento aparece antes da diretiva estrutural v-if e do atributo comum id.",
      "v-if vem primeiro, seguido de id e do manipulador de evento, conforme a ordem da regra.",
    ],
    "vue/component-definition-name-casing": [
      "Exigir PascalCase ou kebab-case nos nomes de definição de componentes",
      "O nome de arquivo myComponent.vue mistura uma inicial minúscula com uma letra maiúscula interna, em vez de usar PascalCase ou kebab-case.",
      "Renomear o arquivo para MyComponent.vue aplica PascalCase; o conteúdo do template permanece igual.",
    ],
    "vue/component-name-in-template-casing": [
      "Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates",
      "O componente é escrito em kebab-case e camelCase, embora a convenção seja PascalCase.",
      "MyComponent usa PascalCase; a sintaxe nativa de slot permanece em minúsculas.",
    ],
    "vue/html-button-has-type": [
      "Exigir um type explícito e válido nos elementos button",
      "Um botão omite type e outro fornece o tipo foo, que não é aceito.",
      "Os botões especificam button, submit ou reset; um type vinculado é tratado como dinâmico.",
    ],
    "vue/html-quotes": [
      "Aplicar um padrão de aspas aos atributos HTML",
      "Os atributos usam aspas simples ou nenhuma aspa, em vez da convenção de aspas duplas.",
      "Tanto os atributos comuns quanto as expressões de diretivas usam aspas duplas.",
    ],
    "vue/html-self-closing": [
      "Aplicar um padrão de tags com fechamento automático",
      "O componente vazio usa uma tag de fechamento separada, enquanto os elementos vazios img e br omitem a grafia de fechamento automático configurada.",
      "O componente e os elementos vazios usam a sintaxe de fechamento automático; uma div com conteúdo mantém sua tag de fechamento.",
    ],
    "vue/max-template-complexity": [
      "Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva",
      "As ramificações, o laço, o conteúdo de slot e as decisões em expressões escritos pelo componente pai produzem pontuações de 13 e 25, acima dos limites padrão de 11 e 16.",
      "O template pai delega a renderização a RowList e mantém um v-if; suas próprias pontuações são 2 e 1.",
    ],
    "vue/multi-word-component-names": [
      "Exigir nomes de componentes com mais de uma palavra",
      "Item.vue dá ao componente um nome de uma única palavra.",
      "TodoItem.vue dá ao mesmo template um nome de componente com mais de uma palavra.",
    ],
    "vue/mustache-interpolation-spacing": [
      "Aplicar espaçamento consistente dentro das interpolações com chaves duplas",
      "A interpolação de texto não tem um espaço junto a um ou a ambos os delimitadores.",
      "Espaços separam a expressão dos delimitadores de abertura e fechamento das chaves duplas.",
    ],
    "vue/no-array-index-key": [
      "Proibir o uso direto da variável de índice de v-for como :key",
      "A chave da lista é seu índice atual, então a identidade do item muda quando a lista é reordenada.",
      "A chave vem de item.id, preservando a identidade de cada item quando sua posição muda.",
    ],
    "vue/no-bare-strings-in-template": [
      "Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado",
      "O texto visível e os atributos de identificação incorporam strings sem tradução diretamente no template.",
      "O conteúdo traduzível chama $t; os exemplos com pontuação e apenas números são exceções permitidas.",
    ],
    "vue/no-boolean-attr-value": [
      "Proibir valores explícitos em atributos HTML booleanos",
      "Os atributos booleanos disabled e checked contêm valores de string redundantes.",
      "A presença de cada atributo booleano expressa o mesmo estado ativado, sem um valor.",
    ],
    "vue/no-child-content": [
      "Proibir conteúdo filho ao usar v-html ou v-text",
      "v-text substitui o conteúdo do parágrafo, então o texto alternativo escrito no template não pode ser preservado por essa diretiva.",
      "Remover o texto filho deixa v-text como a única fonte de conteúdo do parágrafo.",
    ],
    "vue/no-deprecated-filter": [
      "Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical",
      "A barra vertical usa a sintaxe de filtros removida do Vue para aplicar capitalize.",
      "Chamar capitalize(message) aplica a transformação como uma expressão comum.",
    ],
    "vue/no-deprecated-functional-template": [
      "Proibir o atributo `functional` no `<template>` de um SFC",
      "O template do SFC tem o atributo functional removido e lê o antigo contexto props.",
      "O template comum omite functional e lê diretamente a variável msg exposta pelo componente.",
    ],
    "vue/no-deprecated-html-element-is": [
      "Proibir o atributo `is` em elementos HTML nativos",
      "Uma div nativa usa o antigo atributo is sem prefixo para solicitar um componente Vue.",
      "Um componente dinâmico usa :is; a forma no elemento nativo usa explicitamente o prefixo vue:.",
    ],
    "vue/no-deprecated-inline-template": [
      "Proibir o atributo obsoleto `inline-template`",
      "Card usa o atributo obsoleto inline-template para o conteúdo fornecido.",
      "O mesmo conteúdo é passado normalmente, sem o atributo inline-template.",
    ],
    "vue/no-deprecated-router-link-tag-prop": [
      "Proibir a prop `tag` em &lt;router-link&gt;",
      "RouterLink usa a prop tag removida para solicitar um elemento button.",
      "O slot fornece navigate a um botão escrito explicitamente no template.",
    ],
    "vue/no-deprecated-scope-attribute": [
      "Proibir o atributo obsoleto `scope` em &lt;template&gt;",
      "O template do slot declara props pelo atributo obsoleto scope.",
      "A diretiva do slot padrão declara a mesma variável props pela sintaxe atual de slots.",
    ],
    "vue/no-deprecated-slot-attribute": [
      "Proibir o atributo obsoleto `slot`",
      "O slot header é selecionado pelo antigo atributo slot.",
      "v-slot:header seleciona explicitamente o slot header com a diretiva atual.",
    ],
    "vue/no-deprecated-slot-scope-attribute": [
      "Proibir o atributo obsoleto `slot-scope`",
      "O template recebe as props do slot pelo atributo obsoleto slot-scope.",
      "A diretiva #default recebe essas props sem slot-scope.",
    ],
    "vue/no-deprecated-v-bind-sync": [
      "Proibir o modificador obsoleto `.sync` em `v-bind`",
      "As vinculações usam o modificador .sync removido, inclusive em combinação com .camel.",
      "Use uma vinculação comum unidirecional de title ou v-model:title quando for necessário um canal de atualização.",
    ],
    "vue/no-deprecated-v-on-native-modifier": [
      "Proibir o modificador obsoleto `.native` em `v-on`",
      "Os manipuladores do componente usam o modificador de evento .native removido.",
      "Os manipuladores omitem .native e preservam outros modificadores de evento, como .stop.",
    ],
    "vue/no-deprecated-v-on-number-modifiers": [
      "Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`",
      "Os manipuladores de teclado identificam as teclas pelos códigos numéricos removidos 13 e 27.",
      "Os manipuladores usam os modificadores de tecla nomeados enter e esc.",
    ],
    "vue/no-dupe-v-else-if": [
      "Proibir condições duplicadas em cadeias de `v-if` / `v-else-if`",
      "O else-if repete a condição ready já testada pelo primeiro ramo, tornando esse ramo posterior inacessível.",
      "O segundo ramo testa loading, um estado distinto que pode alcançar o else-if.",
    ],
    "vue/no-duplicate-attributes": [
      "Proibir atributos duplicados no mesmo elemento",
      "O mesmo botão declara class duas vezes, em vez de usar um único valor combinado de class.",
      "Os dois nomes de classe aparecem em um único atributo class.",
    ],
    "vue/no-empty-component-block": [
      "Proibir blocos vazios em SFCs",
      "Os blocos template, script e style não contêm conteúdo significativo.",
      "Cada bloco mantido contém marcação, declarações de script ou declarações de estilo de fato.",
    ],
    "vue/no-inline-style": [
      "Desencorajar o uso de atributos de estilo inline",
      "O atributo style estático incorpora a declaração de cor no elemento.",
      "Classes expressam a cor fixa; a largura dependente de ratio permanece como uma vinculação dinâmica de estilo, fora da verificação de atributos estáticos.",
    ],
    "vue/no-invalid-html-attribute": [
      "Proibir valores estáticos inválidos para atributos HTML",
      "A âncora usa stylesheet como valor de rel, embora esse valor pertença a elementos link de folhas de estilo.",
      "A âncora usa help, um valor de rel apropriado para um recurso de ajuda vinculado.",
    ],
    "vue/no-lone-template": [
      "Proibir elementos `<template>` desnecessários",
      "O template interno não tem uma diretiva nem uma função de slot que lhe dê uma finalidade estrutural.",
      "Remover o invólucro desnecessário deixa o parágrafo diretamente dentro da div.",
    ],
    "vue/no-multi-spaces": [
      "Proibir vários espaços consecutivos",
      "Dois espaços separam os atributos ou o nome do elemento e o primeiro atributo.",
      "Espaços únicos separam os mesmos atributos.",
    ],
    "vue/no-multiple-objects-in-class": [
      "Proibir vários objetos literais dentro de uma vinculação de array em :class",
      "Um array de classes contém dois objetos literais no nível superior que podem ser combinados.",
      "Um único objeto contém as condições das classes; arrays com um objeto e uma string ou com entradas não literais continuam permitidos.",
    ],
    "vue/no-multiple-template-root": [
      "Proibir vários nós raiz em um template",
      "A convenção opcional de raiz única encontra dois parágrafos irmãos na raiz do template.",
      "Uma section envolve os parágrafos em uma única raiz; ative essa convenção apenas quando houver um contrato de raiz única.",
    ],
  };
