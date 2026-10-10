export const portugueseRules5: Record<string, readonly [string, string, string]> = {
  "vize:croquis/cf/non-unique-id": [
    "Um id de elemento dentro de um laço não é único por item.",
    "Cada iteração de `v-for` renderiza o mesmo ID literal `result-title`; a chave do laço não torna os IDs do DOM únicos.",
    "O ID do título inclui o ID estável do resultado, produzindo um identificador distinto no documento para cada item.",
  ],
  "vize:croquis/cf/object-identity-comparison": [
    "Um objeto reativo é comparado por identidade, que muda ao remover os invólucros.",
    "`proxy === raw` compara a identidade do invólucro, portanto é falso mesmo que ambos representem o mesmo registro de usuário. A aplicação pretendia comparar a identidade do registro, não a do invólucro do objeto.",
    "Comparar o `id` estável do registro responde à pergunta pretendida sem depender de o objeto ser bruto ou estar envolvido por um proxy.",
  ],
  "vize:croquis/cf/pinia-getter": [
    "Uma função de leitura do Pinia é lida sem `storeToRefs`, portanto não permanecerá reativa.",
    "`const doubled = store.doubled` copia o número atual da função de leitura durante setup. O número copiado não acompanha atualizações posteriores de `store.count`.",
    "`storeToRefs(store)` fornece uma ref reativa da função de leitura que pode ser desestruturada e desembrulhada pelo template, mantendo a conexão com o armazenamento.",
  ],
  "vize:croquis/cf/prop-type-mismatch": [
    "O valor de uma prop passada não corresponde ao tipo declarado.",
    "O pai passa a expressão numérica `42` para a prop `title: string` do filho resolvido.",
    'O literal `title="Hello"` fornece uma string que corresponde à declaração do filho.',
  ],
  "vize:croquis/cf/provide-inject-type": [
    "Um valor fornecido e sua injeção não têm o mesmo tipo.",
    "O provedor anota explicitamente `title` como `string`, enquanto o descendente solicita `inject<number>` para a mesma chave.",
    "O `inject<string>` explícito do consumidor concorda com a anotação do provedor. Mantenha `as string`: este produtor compara anotações explícitas, não tipos literais inferidos.",
  ],
  "vize:croquis/cf/provide-without-symbol": [
    "`provide` usa uma chave comum em vez de um símbolo `InjectionKey`.",
    'Os dois componentes usam a string `"theme"`; funcionalidades sem relação podem reutilizar essa chave acidentalmente.',
    "Exporte um único símbolo tipado `ThemeKey` e importe esse mesmo valor nos pontos de provide e inject. Criar símbolos separados com a mesma descrição não os conectaria.",
  ],
  "vize:croquis/cf/reactive-export": [
    "O estado reativo é exportado pelo módulo.",
    "O módulo exporta um único objeto reativo inicializado, de modo que todo importador recebe a mesma contagem. Em um módulo SSR compartilhado entre requisições, isso impede o isolamento de estado por instância ou requisição pretendido no exemplo.",
    "O módulo exporta uma fábrica, e App a chama dentro de setup. Cada instância obtém uma nova contagem reativa em vez da instância única exportada.",
  ],
  "vize:croquis/cf/reactivity-outside-setup": [
    "Uma API reativa é chamada fora de `setup`.",
    "As duas APIs reativas são executadas durante o carregamento do módulo. As duas instâncias de Counter, portanto, compartilham uma ref e um valor computado, apesar da intenção de manter contadores independentes.",
    "`useCounter` cria a ref e o valor computado de forma síncrona dentro de cada chamada de setup de componente, dando a cada componente visual seu próprio estado e sua própria derivação rastreada.",
  ],
  "vize:croquis/cf/reassignment-breaks-reactivity": [
    "Reatribuir uma variável reativa a substitui por um valor simples.",
    "O filho cria uma ref de prop e depois sobrescreve a variável com `props.user`, descartando a conexão dessa ref.",
    "Mantenha o `toRef` em uma variável `const` e remova a reatribuição que o substitui.",
  ],
  "vize:croquis/cf/reference-escapes-scope": [
    "Uma referência reativa escapa do escopo responsável por seu ciclo de vida.",
    "O cache no nível do processo retém a ref ativa de contagem do componente. Ele pode manter o estado dessa instância alcançável após a desmontagem e observar edições posteriores, embora esse cache deva armazenar um instantâneo.",
    "O cache recebe o número simples atual, de modo que mantém um instantâneo sem reter a ref pertencente ao componente.",
  ],
  "vize:croquis/cf/setup-context-violation": [
    "O contexto de setup é usado de uma forma que o Vue não permite.",
    "`ref(0)` é criada no escopo do módulo de um script comum, fora do contexto de setup por instância representado por este cenário do analisador.",
    "Mova a variável para script setup, onde cada instância de componente possui sua contagem e o template pode lê-la.",
  ],
  "vize:croquis/cf/shallow-deep-access": [
    "Uma propriedade profunda de um valor `shallowReactive` ou `shallowRef` é lida como se fosse rastreada.",
    "`shallowReactive` rastreia a propriedade raiz `user`, mas deixa o objeto aninhado bruto. Alterar `profile.user.name` não notifica o template como uma mutação profunda rastreada.",
    "O `reactive` profundo envolve o objeto de usuário aninhado, de modo que a mesma atribuição de nome pode acionar a atualização do nome exibido.",
  ],
  "vize:croquis/cf/spread-breaks-reactivity": [
    "Espalhar um objeto reativo copia seus valores e perde o rastreamento.",
    "`UserSummary.vue` espalha `props.user` em um novo objeto, criando um instantâneo dos dados reativos recebidos.",
    '`toRef(props, "user")` mantém uma referência à prop recebida em vez de copiar seus campos.',
  ],
  "vize:croquis/cf/suspense-no-fallback": [
    "`<Suspense>` não tem conteúdo alternativo.",
    "O limite Suspense tem um filho assíncrono, mas não tem conteúdo alternativo, deixando este exemplo sem conteúdo de carregamento durante o estado pendente.",
    "O slot `#fallback` fornece um parágrafo explícito de carregamento até que o filho assíncrono seja resolvido.",
  ],
  "vize:croquis/cf/template-ref-timing": [
    "Uma ref de template é lida antes da montagem do componente.",
    "Setup lê a ref de template antes da montagem, quando seu valor ainda é nulo. Por isso, a chamada opcional de foco não realiza nenhuma ação de foco.",
    "`onMounted` adia a leitura até que o Vue tenha atribuído o elemento de entrada à ref de template, permitindo que o auxiliar de foco atue sobre ele.",
  ],
  "vize:croquis/cf/toraw-mutation": [
    "`toRaw` é usado e o objeto bruto é alterado em seguida.",
    "`rename` obtém o objeto-alvo bruto e escreve em `raw.name`, contornando a função de escrita do proxy que notificaria o nome reativo exibido.",
    "Escrever em `profile.name` por meio do proxy reativo passado preserva a mesma renomeação e notifica seus dependentes.",
  ],
  "vize:croquis/cf/uncaught-error": [
    "Um componente pode lançar um erro e nenhum limite de erro o captura.",
    "O template do filho chama `JSON.parse` com uma entrada malformada, e o pai alcançável não tem um limite de captura de erros.",
    "O pai registra `onErrorCaptured` ao redor desse filho. Retornar `false` interrompe a propagação; um limite de erro em produção também deve apresentar uma interface útil para recuperação.",
  ],
  "vize:croquis/cf/undeclared-emit": [
    "O componente emite um evento que não foi declarado.",
    'O filho chama `emit("save")`, mas seu contrato `defineEmits` declara apenas `cancel`.',
    "Declare `save` com sua tupla vazia de argumentos para que o evento emitido concorde com o contrato do componente.",
  ],
  "vize:croquis/cf/undeclared-prop": [
    "Um pai passa uma prop que o filho não declara.",
    "O pai passa `typo`, embora o filho resolvido declare apenas `title`. Essa convenção do analisador é distinta do comportamento geral de herança automática de atributos do Vue.",
    "Remova a vinculação não intencional de `typo` e mantenha a prop declarada `title`.",
  ],
  "vize:croquis/cf/undefined-slot": [
    "Um pai preenche um slot que o filho não expõe.",
    "App fornece um slot `footer`, mas Card declara e renderiza apenas `header`. O conteúdo de Notice fornecido não tem um ponto de renderização de slot correspondente nesse filho.",
    "App fornece `header`, correspondendo tanto à declaração tipada de slot do filho quanto ao seu ponto de renderização, de modo que Notice aparece ali.",
  ],
  "vize:croquis/cf/unhandled-event": [
    "Um filho emite um evento que nenhum pai trata.",
    "`Child.vue` emite `save`, mas seu invólucro imediato não escuta esse evento; eventos de componentes não se propagam automaticamente por invólucros.",
    "`Wrapper.vue` associa um ouvinte de `save` ao seu filho direto. A função de retorno vazia demonstra o tratamento para esta regra, não uma implementação completa de salvamento.",
  ],
  "vize:croquis/cf/unmatched-inject": [
    "`inject` nomeia uma chave que nenhum ancestral fornece.",
    "`ThemeLabel.vue` injeta `ThemeKey`, mas seu ancestral alcançável `App.vue` nunca fornece essa chave.",
    "`App.vue` fornece um tema reativo usando o mesmo `ThemeKey` exportado, antes de renderizar o descendente que o injeta.",
  ],
  "vize:croquis/cf/unmatched-listener": [
    "Um pai escuta um evento que o filho não emite.",
    "O pai escuta `save`, enquanto o filho resolvido declara apenas `cancel`.",
    "O filho declara e emite `save`, correspondendo ao nome do ouvinte do pai.",
  ],
  "vize:croquis/cf/unregistered-component": [
    "Um template usa um componente que não foi registrado nem importado.",
    "Existe um arquivo `Child.vue`, mas o pai não importa nem registra `Child` de outra forma para seu template.",
    "Importe `Child` no script setup do pai para que o template resolva a variável do componente.",
  ],
  "vize:croquis/cf/unresolved-import": [
    "Uma importação não resolve para um módulo.",
    "O pai importa `./Missing.vue`, mas o projeto contém `Child.vue` em vez desse caminho.",
    "Aponte a importação para o arquivo existente `./Child.vue`, mantendo a mesma variável no template.",
  ],
  "vize:croquis/cf/unused-attrs": [
    "Atributos de herança automática são passados para um componente com múltiplas raízes que não os usa.",
    "O `tracking-code` do pai não é consumido como prop nem encaminhado pelo filho com múltiplas raízes.",
    "Vincular `$attrs` a `<main>` dá um destino explícito a esse atributo de herança automática.",
  ],
  "vize:croquis/cf/unused-emit": [
    "Um evento declarado nunca é emitido.",
    "O filho declara `save`, mas nunca chama a função de emissão de eventos com esse nome.",
    'O exemplo chama `emit("save")`, fazendo com que o evento declarado seja usado. Interações reais devem emiti-lo quando a ação correspondente ocorrer.',
  ],
  "vize:croquis/cf/unused-provide": [
    "Uma chave fornecida nunca é injetada.",
    "`App.vue` fornece `ThemeKey`, mas sua subárvore renderizada de `Dashboard.vue` não tem nenhum consumidor dessa chave.",
    "O painel agora renderiza `ThemeLabel.vue`, que injeta a identidade exata de `ThemeKey` do ancestral.",
  ],
  "vize:croquis/cf/value-extraction-breaks-reactivity": [
    "Ler um valor reativo para uma variável local perde as atualizações posteriores.",
    "O `item` reativo desestruturado do Vue 3.5 é lido para `itemSnapshot` uma vez; substituições posteriores da prop não atualizam esse instantâneo.",
    "Leia `item` dentro de `computed`, para que a transformação de desestruturação reativa de props do Vue possa rastrear cada avaliação.",
  ],
  "vize:croquis/cf/watch-can-be-computed": [
    "Um observador apenas copia um valor para o estado e pode ser um valor computado.",
    "O observador não produz nenhum efeito externo; ele apenas mantém uma segunda ref gravável sincronizada com o dobro de `count`. Este exemplo não tem escritas independentes nesse valor derivado.",
    "Uma função de leitura computada expressa diretamente a mesma derivação e remove a sincronização manual e o estado gravável adicional.",
  ],
  "vize:croquis/cf/watcheffect-async": [
    "`watchEffect` inicia uma tarefa assíncrona e não consegue limpar a execução anterior.",
    "O `watchEffect` assíncrono mistura a coleta implícita de dependências com uma requisição aguardada e sem uma proteção contra invalidação.",
    "Um `watch(() => props.query, ...)` explícito declara a origem, registra a limpeza da requisição e recusa uma resposta desatualizada após a invalidação.",
  ],
  "vize:croquis/cf/watcher-outside-setup": [
    "`watch` ou `watchEffect` é chamado fora de `setup`.",
    "O observador é criado no carregamento do módulo, fora do setup de qualquer uma das instâncias de Observer, e as duas instâncias compartilham suas refs. Ele não é interrompido automaticamente quando uma instância específica de Observer é desmontada.",
    "Cada chamada síncrona de setup cria suas próprias refs e seu próprio observador dentro de `useObserver`. O Vue associa esse observador ao ciclo de vida do componente que o chama.",
  ],
  "vue/cross-file-attrs-fallthrough": [
    "Um pai passa atributos para um filho resolvido cuja raiz não pode herdá-los e que não usa $attrs explicitamente.",
    'O pai passa `class="notice"` para um filho resolvido com raiz em fragmento, que não tem um destino automático para atributos e nunca lê `$attrs`.',
    "O filho escolhe `<main>` como destino vinculando `$attrs` ali; seu irmão `<aside>` permanece separado.",
  ],
};
