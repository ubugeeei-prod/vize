export const portugueseRules3: Record<string, readonly [string, string, string]> = {
  "script/prefer-computed": [
    "Preferir computed() para estado reativo derivado",
    "O observador apenas copia uma derivação de `count` para um segundo ref, `doubled`, então o estado derivado é mantido por sincronização manual.",
    "`computed(() => count.value * 2)` expressa a derivação diretamente e remove tanto o ref gravável adicional quanto seu observador de sincronização.",
  ],
  "script/prefer-define-options": [
    "Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs",
    "A única instrução significativa do script comum exporta um objeto contendo apenas `name` e `inheritAttrs`; essas opções podem ser expressas por `defineOptions`.",
    "O método `data()` mostrado faz o script conter lógica efetiva da Options API, portanto ele fica fora da sugestão conservadora desta regra para scripts que contêm apenas opções. Este exemplo correto demonstra uma exceção permitida; a migração direta colocaria `defineOptions({ name: 'MyComponent', inheritAttrs: false })` em `<script setup>`.",
  ],
  "script/prefer-import-from-vue": [
    "Preferir importar de 'vue' em vez de pacotes internos",
    "`ref` e `h` são importados dos pacotes internos `@vue/runtime-core` e `@vue/runtime-dom` em vez do pacote público `vue`.",
    "Ambas as funções auxiliares são importadas juntas de `vue`, usando o ponto de entrada público do pacote em vez de qualquer um dos pacotes internos.",
  ],
  "script/prefer-ref-over-reactive": [
    "Recomendar o uso de ref() em vez de reactive() para gerenciar estado",
    "O estado é criado com `reactive`, contrariando a preferência desta regra de convenção por refs. O exemplo ilustra uma preferência de estilo, não um objeto reativo inerentemente inválido.",
    "Os exemplos criam tanto estado escalar quanto de objeto com `ref`; campos relacionados também podem ser separados em refs distintos. Isso atende à forma preferida de criação de estado.",
  ],
  "script/prefer-use-attrs": [
    "Recomendar o uso de useAttrs() em vez de context.attrs",
    "`setup` obtém `attrs` desestruturando seu parâmetro de contexto, forma que esta regra pede para substituir pela função auxiliar da Composition API.",
    "`useAttrs()` fornece `attrs` dentro de setup, mantendo a leitura de `attrs.class` sem depender do segundo parâmetro de setup.",
  ],
  "script/prefer-use-id": [
    "Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+)",
    "`id` contém `Math.random()`, então o identificador gerado para input/label pode diferir entre a renderização no servidor e no cliente. A variável com nome de ID é o contexto de geração reconhecido pela regra.",
    "`useId()` do Vue 3.5+ gera o identificador, e tanto `:for` quanto `:id` continuam lendo a mesma variável em vez de gerar valores aleatórios de forma independente.",
  ],
  "script/prefer-use-slots": [
    "Recomendar o uso de useSlots() em vez de context.slots",
    "`setup` desestrutura `slots` de seu argumento de contexto, a forma de acesso que esta regra prefere substituir.",
    "`useSlots()` obtém os slots dentro de setup, preservando a função de renderização e sua chamada opcional ao slot padrão sem um parâmetro de contexto.",
  ],
  "script/prefer-use-template-ref": [
    "Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+)",
    'O ref `input`, que aceita null, está associado ao literal `ref="input"` do template, identificando-o como uma referência de elemento em vez de um dado comum que aceita null.',
    "`useTemplateRef<HTMLInputElement>('input')` do Vue 3.5+ torna essa referência de template explícita. O `error = ref(null)` sem associação permanece um dado comum e fica intencionalmente fora desta regra.",
  ],
  "script/require-default-prop": [
    "Exigir um valor padrão para toda prop opcional que não seja Boolean",
    "`name` e `age` são props opcionais de tempo de execução que não são Boolean e não têm valores padrão, deixando indefinidos seus valores quando a entrada é omitida.",
    "`name` recebe `default: ''`. `enabled` usa o valor padrão false implícito de Boolean, e o `id` obrigatório não precisa de um valor alternativo, ilustrando ambas as exceções.",
  ],
  "script/require-explicit-emits": [
    "Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits",
    "A função de emissão capturada emite `save`, mas `defineEmits([])` não declara esse evento.",
    'Adicionar `"save"` à declaração torna o evento literal emitido parte do contrato explícito de eventos do componente.',
  ],
  "script/require-explicit-slots": [
    "Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;()",
    "O `defineProps<{ id: number }>()` tipado estabelece a sintaxe TypeScript, mas setup usa `useSlots()` sem uma declaração de `defineSlots`. Assim, a regra encontra slots consumidos sem um contrato explícito de slots.",
    "`defineSlots` declara um slot `default` cujas props incluem `msg: string`; `useSlots()` agora aparece junto de um contrato de slots explicitamente tipado.",
  ],
  "script/require-function-return-type": [
    "Exigir anotações de tipo de retorno nas funções",
    "Tanto `add` quanto `greet` anotam seus parâmetros, mas omitem uma anotação de tipo de retorno; retornos inferidos não atendem a esta política de anotação explícita.",
    "`add` declara `: number`, e `greet` declara `: string`, tornando os contratos de retorno explícitos sem alterar nenhum dos corpos.",
  ],
  "script/require-prop-type-constructor": [
    "Exigir que os valores de `type` de props sejam construtores em vez de literais de string",
    'As declarações de props usam as strings `"String"` e `"Number"` como tipos de tempo de execução, inclusive dentro do array de construtores. Essas strings não são funções construtoras.',
    "As declarações usam os identificadores reais `String` e `Number`, inclusive no array de união `[String, Number]`.",
  ],
  "script/require-prop-types": [
    "Exigir que toda prop declare um tipo",
    "A entrada do array declara apenas o nome `status`; o valor `null` e o descritor vazio também não declaram um tipo de prop em tempo de execução.",
    "`status: String` fornece um construtor na forma abreviada, e `other` fornece `type: Number` dentro de seu descritor. Ambas as props agora têm declarações de tipo.",
  ],
  "script/require-symbol-provide": [
    "Recomendar o uso de Symbol como chave de injeção para provide/inject",
    "`provide` e `inject` usam chaves de string literal como `'user'` e `'theme'`, que podem colidir com outro provedor que use a mesma grafia.",
    "O `UserKey` compartilhado é criado com `Symbol` e anotado como `InjectionKey<User>`; ambas as chamadas passam essa chave em vez de uma string literal.",
  ],
  "script/require-typed-object-prop": [
    "Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array`",
    "Os construtores simples `Object` e `Array` descrevem apenas categorias amplas de tempo de execução, então nem `user` nem a estrutura dos elementos de `items` têm um tipo estático explícito.",
    "`PropType<User>` e `PropType<User[]>` adicionam os tipos de objeto e de elemento enquanto mantêm os mesmos construtores de tempo de execução.",
  ],
  "script/require-typed-ref": [
    "Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined",
    "As chamadas do `ref` importado não têm um argumento de tipo nem um valor inicial útil: a ausência de argumento, `null` e `undefined` não permitem inferir o tipo pretendido para o valor futuro.",
    "Os argumentos de tipo explícitos descrevem os refs de string e de User que aceita null. `ref(0)` já tem um inicializador numérico concreto e pode depender da inferência.",
  ],
  "script/require-valid-default-prop": [
    "Exigir que o valor padrão de uma prop seja válido para seu tipo declarado",
    "As props Number e Boolean recebem valores padrão escalares incompatíveis, e as props Array e Object usam valores literais compartilhados em vez de funções de criação.",
    "Os valores padrão escalares passam a ser `0` e `false`; os valores padrão de array e objeto passam a ser funções que retornam novos valores. O exemplo `[String, Number]` aceita seu valor padrão de string porque ele corresponde a um dos tipos declarados.",
  ],
  "script/return-in-computed-property": [
    "Exigir um valor de retorno em todo getter computado",
    "O getter computado com corpo de bloco avalia `1 + 2`, mas nunca retorna o resultado, deixando o valor computado undefined.",
    "`return 1 + 2` transforma a expressão no valor retornado pelo getter. A regra procura um return que retorne um valor no próprio getter, não apenas uma instrução de expressão.",
  ],
  "script/return-in-emits-validator": [
    "Exigir um valor de retorno em todo validador de emits da Options API",
    "O validador `submit` registra a carga útil, mas não retorna um resultado de validação, então seu corpo de bloco produz undefined.",
    "`return payload != null` fornece um resultado booleano de validação para a carga útil enviada em vez de encerrar sem um valor retornado.",
  ],
  "script/valid-define-emits": [
    "Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada)",
    'A mesma chamada de `defineEmits` fornece tanto um argumento de tipo quanto o array de tempo de execução `["save"]`, misturando duas declarações mutuamente exclusivas.',
    "Remover o argumento de tempo de execução deixa uma única declaração de evento baseada em tipos para `save`.",
  ],
  "script/valid-define-options": [
    "Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots)",
    "A primeira chamada coloca a declaração específica de `props` dentro de `defineOptions`; as chamadas posteriores também repetem a macro e incluem um argumento que não é objeto. Elas ilustram as restrições de formato proibido e de chamadas repetidas.",
    "Uma única chamada de `defineOptions` recebe um objeto contendo apenas as opções comuns suportadas `name` e `inheritAttrs`.",
  ],
  "script/valid-define-props": [
    "Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais)",
    "A mesma chamada de `defineProps` fornece tanto `{ title: string }` como argumento de tipo quanto `{ title: String }` como argumento de tempo de execução, combinação que o compilador não permite.",
    "Remover o objeto de tempo de execução deixa uma única declaração baseada em tipos para `title` em vez de combinar ambas as formas de declaração.",
  ],
  "script/valid-next-tick": [
    "Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno",
    "O `nextTick()` importado é uma expressão isolada sem função de retorno, então a Promise retornada é ignorada e nenhum trabalho aguarda a atualização do DOM.",
    "`await nextTick()` consome a Promise e aguarda explicitamente a próxima atualização do DOM antes que o código subsequente de setup continue.",
  ],
  "ssr/no-browser-globals-in-ssr": [
    "Proibir variáveis globais exclusivas do navegador no contexto de SSR",
    "Setup lê `window.innerWidth` imediatamente, embora `window` não exista quando o componente é executado no servidor.",
    "A largura inicial é um valor de ref seguro no servidor, e o acesso ao navegador é movido para `onMounted`, que é executado no cliente em vez de durante o setup de SSR.",
  ],
  "ssr/no-hydration-mismatch": [
    "Proibir valores não determinísticos que causam divergências na hidratação",
    "O template avalia `Math.random()` durante a renderização, então o servidor e o cliente podem produzir textos diferentes para o mesmo parágrafo.",
    'O parágrafo renderiza o estado estável `seed` em vez de um novo resultado aleatório. Neste exemplo no estilo do Nuxt, `useState` fornece o estado compartilhado e o inicializador é a constante `"stable"`.',
  ],
  "type/no-floating-promises": [
    "Proibir Promises soltas (não tratadas)",
    "A função assíncrona `save` retorna uma Promise, mas a chamada isolada de `save()` não a aguarda nem a retorna, e não sinaliza explicitamente um descarte intencional.",
    "`void save()` sinaliza explicitamente a intenção de executar sem aguardar, aceita por esta regra. É um marcador explícito de descarte, não um manipulador de rejeição.",
  ],
  "type/no-reactivity-loss": [
    "Proibir cópias estáticas simples de valores reativos em atribuições e chamadas",
    "`const count = state.count` obtém uma cópia numérica simples da propriedade reativa, então atualizações posteriores de `state.count` não se refletem nessa variável.",
    '`toRef(state, "count")` mantém `count` vinculado à propriedade reativa original em vez de copiar seu valor primitivo atual.',
  ],
  "type/no-unsafe-template-binding": [
    "Proibir vinculações de template que resultam em tipos inseguros",
    "O `value` interpolado é explicitamente tipado como `any`, então o verificador não consegue atribuir um tipo concreto seguro à vinculação do template.",
    "Alterar a anotação para `string` fornece à mesma interpolação um tipo concreto que pode ser verificado, sem alterar o valor renderizado.",
  ],
  "type/require-typed-emits": [
    "Exigir uma definição de tipo para defineEmits",
    'O `defineEmits(["save"])` que usa apenas um array declara o nome do evento sem um contrato tipado de carga útil.',
    "`defineEmits<{ save: [] }>()` declara o evento tipado `save` com uma tupla vazia de carga útil, indicando explicitamente que ele não recebe argumentos de carga útil.",
  ],
  "type/require-typed-props": [
    "Exigir uma definição de tipo para defineProps",
    'O `defineProps(["title"])` que usa apenas um array declara `title` pelo nome sem lhe atribuir um tipo.',
    "`defineProps<{ title: string }>()` atribui a `title` um tipo string explícito em vez de uma declaração de tempo de execução que contém apenas o nome.",
  ],
  "type/strict-boolean-expressions": [
    "Exigir expressões booleanas seguras nas condições de script e template",
    "`if (count)` depende da conversão implícita para booleano de uma variável numérica que pode estar ausente em vez de um teste booleano explícito; também confunde zero com ausência.",
    "`count !== undefined && count > 0` testa separadamente a presença e a positividade, produzindo uma condição booleana explícita após restringir o tipo do valor opcional.",
  ],
  "vapor/no-inline-template": [
    "Proibir o atributo obsoleto inline-template",
    "LegacyCard usa o atributo inline-template para a marcação de seu conteúdo filho.",
    "A marcação é passada pelo slot padrão em vez de um template inline.",
  ],
  "vapor/no-vue-lifecycle-events": [
    "Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor)",
    "O input usa o evento de ciclo de vida de template @vue:mounted.",
    "onMounted acessa a referência nomeada de template e coloca o foco no input por meio do hook de ciclo de vida de script suportado.",
  ],
  "vapor/prefer-static-class": [
    "Preferir class estática a uma vinculação dinâmica de class para literais de string",
    "A vinculação de class avalia uma string constante, embora a classe não mude.",
    "Um atributo class estático expressa as mesmas classes do painel sem uma vinculação.",
  ],
  "vapor/require-vapor-attribute": [
    "Sugerir a adição do atributo vapor a script setup",
    "O bloco script setup não tem o atributo de compilação Vapor. Esta é uma convenção pretendida: a função de retorno atualmente vazia da regra não diagnostica sua ausência.",
    "Adicionar vapor seleciona a compilação Vapor. Isso demonstra a correção pretendida e não implica que o linter atual emita esta regra do catálogo.",
  ],
};
