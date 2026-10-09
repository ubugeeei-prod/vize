export const portugueseVue1: Record<string, readonly [purpose: string, bad: string, good: string]> =
  {
    "vue/no-mutating-props": [
      "Proibir a mutação de props de componentes",
      "Incrementar props.count escreve diretamente em um valor fornecido pelo componente pai.",
      "O componente emite update:count com o próximo valor, deixando o pai responsável por atualizar a prop.",
    ],
    "vue/no-negated-v-if-condition": [
      "Proibir uma condição negada em v-if quando a cadeia tiver v-else",
      "Os ramos emparelhados v-if e v-else começam com uma condição negada.",
      "Uma condição positiva ok vem primeiro; ao inverter uma condição, coloque primeiro o ramo originalmente oposto. Um v-if negado isolado e comparações !== continuam permitidos.",
    ],
    "vue/no-non-component-keep-alive-child": [
      "Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`",
      "KeepAlive envolve condicionalmente uma div nativa, em vez de armazenar UserCard diretamente em cache.",
      "O primeiro exemplo torna UserCard o filho condicional. O invólucro com v-show ilustra uma estrutura fora desta verificação de filhos condicionais, sem prometer que o invólucro nativo seja armazenado em cache.",
    ],
    "vue/no-preprocessor-lang": [
      "Desencorajar o uso de preprocessadores CSS em favor de CSS moderno",
      "O bloco style seleciona SCSS com lang. Isso descreve a convenção pretendida de não usar preprocessadores; o processamento atual de SFCs não emite esta regra.",
      "As mesmas declarações CSS omitem o lang do preprocessador. Essa é a correção da convenção, e não uma diferença executável de diagnósticos entre Bad/Good hoje.",
    ],
    "vue/no-reserved-component-names": [
      "Proibir o uso de nomes reservados como nomes de componentes",
      "O nome de componente button entra em conflito com o nome de um elemento HTML nativo.",
      "AppButton é um nome de componente da aplicação e não reutiliza o nome nativo button.",
    ],
    "vue/no-root-v-if": [
      "Proibir v-if no único elemento raiz de um template",
      "A própria raiz do componente aparece e desaparece sob v-if.",
      "Uma div externa estável permanece como raiz, enquanto o parágrafo aninhado recebe a condição de visibilidade.",
    ],
    "vue/no-script-non-standard-lang": [
      "Desencorajar valores não padronizados de lang em scripts",
      "O script usa sintaxe CoffeeScript com lang=coffee. O processamento atual de SFCs não emite esta regra do catálogo para essa linguagem.",
      "O script usa uma declaração TypeScript comum com lang=ts, ilustrando a convenção de linguagem pretendida.",
    ],
    "vue/no-src-attribute": [
      "Desencorajar o atributo src em blocos de SFCs",
      "Os blocos do SFC delegam o conteúdo de template, script e style a arquivos src.",
      "Cada bloco do SFC contém seu próprio conteúdo, sem um atributo src externo.",
    ],
    "vue/no-static-inline-styles": [
      "Proibir atributos estáticos de estilo inline",
      "O parágrafo contém a declaração de cor constante no atributo style.",
      "Uma classe notice e uma folha de estilo com escopo mantêm a cor constante fora do atributo no template.",
    ],
    "vue/no-template-key": [
      "Proibir o atributo `key` em `<template>`",
      "Um invólucro template sem laço tem uma key, embora não seja o limite da iteração com chave.",
      "A key pertence a uma iteração template v-for, na qual identifica cada fragmento repetido.",
    ],
    "vue/no-template-lang": [
      "Desencorajar o atributo lang no bloco template",
      "O template seleciona Pug por meio de lang. Essa é uma convenção pretendida de usar apenas HTML; o processamento atual de SFCs não gera diagnósticos para este ID do catálogo.",
      "Um template HTML comum omite lang e usa o parágrafo diretamente. Isso ilustra a convenção sem afirmar que há uma ocorrência diagnosticada atualmente em SFCs.",
    ],
    "vue/no-template-shadow": [
      "Proibir nomes de variáveis que ocultam variáveis de um escopo externo",
      "O v-for interno declara item novamente e oculta a variável item externa dentro do laço aninhado.",
      "O laço interno declara child, deixando item disponível para a linha externa e child para a linha aninhada.",
    ],
    "vue/no-template-target-blank": [
      'Proibir target="_blank" sem rel="noopener noreferrer"',
      "O link externo abre um novo contexto de navegação sem a proteção rel esperada.",
      "O mesmo link inclui noopener noreferrer junto de target=_blank.",
    ],
    "vue/no-textarea-mustache": [
      "Proibir interpolação com chaves duplas em `<textarea>`",
      "O textarea coloca message em uma interpolação filha, em vez de vincular seu valor.",
      "v-model vincula o valor editável do textarea a message.",
    ],
    "vue/no-undefined-refs": [
      "Proibir referências a variáveis não definidas nos templates",
      "O template lê missing, embora o script declare apenas message.",
      "A interpolação lê a variável message existente.",
    ],
    "vue/no-unsafe-url": [
      "Alertar sobre vinculações de URL potencialmente inseguras",
      "O destino da âncora começa com o esquema executável javascript:.",
      "A âncora usa o destino local comum de navegação /next.",
    ],
    "vue/no-unsandboxed-iframe": [
      "Exigir um atributo sandbox nos elementos iframe",
      "O frame incorporado não tem um atributo sandbox que limite suas capacidades.",
      "sandbox aplica restrições; allow-scripts habilita explicitamente essa única capacidade quando necessário.",
    ],
    "vue/no-unused-components": [
      "Proibir o registro de componentes não usados nos templates",
      "UserAvatar é importado como componente, mas o template nunca o renderiza.",
      "O template renderiza o UserAvatar importado e passa a variável user.",
    ],
    "vue/no-unused-properties": [
      "Proibir propriedades não usadas definidas em defineProps",
      "O componente declara description como prop, mas renderiza apenas title.",
      "As duas props declaradas são referenciadas pelo template.",
    ],
    "vue/no-unused-refs": [
      'Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt;',
      "O template declara o nome de ref unused sem uma variável de referência correspondente no script.",
      "A ref de template inputEl tem uma variável ref de mesmo nome em script setup.",
    ],
    "vue/no-unused-setup-bindings": [
      "Proibir variáveis de script setup que nunca são lidas",
      "A variável message de script setup nunca é lida pelo template.",
      "O parágrafo interpola message, usando a variável declarada.",
    ],
    "vue/no-unused-vars": [
      "Proibir definições de variáveis não usadas nas diretivas v-for e v-slot",
      "O laço declara um index não usado e o slot declara foo sem referenciá-lo.",
      "Os exemplos usam index ou o marcam como intencionalmente não usado por meio de _index, e o slot renderiza data. Chaves de índice são apenas um exemplo de uso aqui, e não uma recomendação para manter a identidade estável dos itens.",
    ],
    "vue/no-use-v-else-with-v-for": [
      "Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for`",
      "O ramo else e a iteração v-for estão associados ao mesmo parágrafo.",
      "Um template separado contém v-else, e seu parágrafo filho contém v-for.",
    ],
    "vue/no-use-v-if-with-v-for": [
      "Proibir `v-if` no mesmo elemento que `v-for`",
      "O mesmo elemento de lista combina v-if e v-for e testa a visibilidade por meio da variável do laço.",
      "Uma coleção computada filtra os itens visíveis antes que o template itere sobre eles.",
    ],
    "vue/no-useless-mustaches": [
      "Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante",
      "A interpolação contém apenas uma string constante e não precisa avaliar uma expressão.",
      "O texto literal é escrito diretamente; expressões com variáveis, strings de template interpoladas e espaços separadores intencionais continuam sendo casos de interpolação.",
    ],
    "vue/no-useless-template-attributes": [
      "Proibir atributos sem efeito em elementos `<template>`",
      "O template condicional tem uma class, mas esse invólucro estrutural não renderiza um elemento DOM para recebê-la.",
      "A class passa para o parágrafo que é realmente renderizado, enquanto v-if permanece no template estrutural.",
    ],
    "vue/no-useless-v-bind": [
      "Proibir um v-bind cujo valor seja uma string literal simples",
      "A vinculação foo avalia uma string constante entre aspas ou uma string de template sem interpolação.",
      "O valor constante vira um atributo estático; valores com variáveis e interpolações mantêm sua vinculação.",
    ],
    "vue/no-v-for-template-key-on-child": [
      "Proibir `key` no filho de um `<template v-for>`",
      "O parágrafo filho tem a key, enquanto a própria iteração template não tem chave.",
      "A key passa para template v-for, identificando o fragmento repetido completo.",
    ],
    "vue/no-v-html": [
      "Alertar sobre v-html para prevenir vulnerabilidades XSS",
      "v-html interpreta content como HTML, em vez de texto comum.",
      "A interpolação com chaves duplas exibe content como texto escapado, em vez de injetar HTML.",
    ],
    "vue/no-v-text-v-html-on-component": [
      "Proibir v-text / v-html em elementos de componente",
      "A tag do componente recebe v-html ou v-text, que substitui o conteúdo do elemento em vez de fornecer slots ao componente.",
      "Elementos HTML nativos podem receber as diretivas; MyComponent recebe seu conteúdo pelo slot padrão.",
    ],
    "vue/no-v-text": [
      "Proibir a diretiva v-text; preferir interpolação com chaves duplas",
      "O conteúdo da div é fornecido pela diretiva v-text.",
      "A interpolação com chaves duplas expressa a mesma vinculação de texto diretamente no conteúdo do elemento.",
    ],
    "vue/permitted-contents": [
      "Aplicar as regras do modelo de conteúdo HTML",
      "Os exemplos colocam conteúdo em bloco dentro de p, omitem o corpo da tabela, aninham controles interativos ou colocam uma div diretamente dentro de ul.",
      "Os exemplos usam conteúdo inline no parágrafo, um tbody explícito e filhos li. O componente personalizado MyItem não é tratado como um filho nativo conhecido de ul.",
    ],
    "vue/prefer-props-shorthand": [
      "Recomendar sintaxe abreviada para props (Vue 3.4+)",
      "Cada vinculação repete o nome da variável correspondente, inclusive o equivalente camelCase de um argumento separado por hífen.",
      "A forma abreviada de vinculação de mesmo nome do Vue 3.4+ remove as expressões repetidas; uma variável de origem diferente, como bar, permanece explícita.",
    ],
    "vue/prefer-true-attribute-shorthand": [
      "Preferir a forma abreviada para um atributo booleano vinculado a `true`",
      "Um atributo booleano nativo disabled vincula o valor constante true.",
      "O atributo nativo usa sua forma booleana abreviada. Vinculações false e props de componentes mantêm seus valores explícitos.",
    ],
    "vue/prop-name-casing": [
      "Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas",
      "O nome de prop declarado user_name usa uma grafia separada por sublinhado.",
      "A declaração e sua referência no template usam o nome camelCase userName.",
    ],
  };
