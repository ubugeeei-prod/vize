export const portugueseRules4: Record<string, readonly [string, string, string]> = {
  "ecosystem/vue-router-extra-param": [
    "A rota não declara tab; o Vue Router o descarta.",
    "O caminho `user-post` declara `userId` e `postId`, mas a navegação também fornece `tab`, que não foi declarado, como parâmetro do caminho.",
    "Remova `tab` de params e mantenha apenas as chaves presentes no caminho da rota. Use query separadamente se a aplicação precisar selecionar uma aba.",
  ],
  "ecosystem/vue-router-missing-param": [
    "O parâmetro obrigatório postId está ausente; depender da rota atual é frágil.",
    "A navegação omite o parâmetro obrigatório `postId` do caminho `user-post`. Isso é um aviso porque o Vue Router pode herdar um valor da rota atual.",
    "Passe `userId` e `postId` explicitamente para que a navegação não dependa do estado dos parâmetros da rota atual.",
  ],
  "ecosystem/vue-router-param-type": [
    "postId não é repetível, portanto um array é inválido.",
    '`postId` é um parâmetro escalar do caminho, mas a navegação fornece a ele o array `["2"]`.',
    'Passe o escalar `"2"` para o segmento `postId`, que não é repetível.',
  ],
  "ecosystem/vue-router-unknown-route": [
    "O nome está ausente do roteador completo instalado.",
    "O roteador instalado alcançável declara `user-post`, mas a navegação usa o nome incorreto `user-posts`.",
    "Use o nome registrado `user-post`, mantendo os dois parâmetros de caminho declarados.",
  ],
  "html/cross-component-nesting": [
    "Verificar o aninhamento HTML real após compor os componentes importados.",
    "O `<p>` do pai contém um filho resolvido cuja raiz é `<div>`, produzindo um aninhamento inválido de bloco em parágrafo após a composição.",
    "Use um contêiner `<section>` que possa conter o elemento de bloco do filho; o filho permanece inalterado.",
  ],
  "vize:croquis/cf/array-mutation": [
    "Um array é alterado por índice, o que um array reativo não rastreia.",
    "Neste projeto histórico com Vue 2.7, `items[0] = next` altera o array sem notificar o observador de arrays do Vue 2, de modo que o primeiro item exibido pode não ser atualizado.",
    "`splice(0, 1, next)` usa o método de mutação de arrays observado pelo Vue 2, permitindo que a mesma substituição atualize a visualização.",
  ],
  "vize:croquis/cf/async-boundary": [
    "O estado reativo atravessa um limite assíncrono e pode ser observado desatualizado.",
    "Uma consulta antiga mais lenta pode terminar após uma consulta mais recente e sobrescrever `result`, porque o observador não tem limpeza na invalidação.",
    "Registre a limpeza antes de aguardar: aborte a requisição antiga e invalide seu sinalizador `active`; depois, atribua apenas uma resposta que ainda esteja ativa.",
  ],
  "vize:croquis/cf/async-no-suspense": [
    "Um componente assíncrono é renderizado sem um limite Suspense.",
    "O filho tem await no nível superior, mas o pai não fornece um limite `<Suspense>`. A análise atual do código-fonte não fornece o fato de macro necessário para emitir este código de diagnóstico.",
    "O pai envolve o mesmo filho assíncrono em `<Suspense>` com conteúdo alternativo de carregamento. Isso demonstra a convenção; a passagem atual continua sem emitir o diagnóstico para nenhuma das duas alternativas de código-fonte.",
  ],
  "vize:croquis/cf/browser-api-ssr": [
    "Uma API exclusiva do navegador é usada onde o componente pode ser renderizado no servidor.",
    "`window.innerWidth` é executado durante setup, quando um ambiente SSR não tem o `window` do navegador.",
    "Inicialize uma ref com um valor seguro para o servidor e leia `window` dentro de `onMounted`, que é executado após a montagem no cliente.",
  ],
  "vize:croquis/cf/circular-dep": [
    "Os componentes importam uns aos outros em um ciclo.",
    "`a.ts` importa `b.ts`, que importa `a.ts` de volta. Ambos inicializam imediatamente uma constante a partir da constante ainda não inicializada do outro módulo, causando uma falha de zona morta temporal.",
    "Os dois módulos leem prefixos inicializados do módulo independente `labels.ts`, removendo o ciclo e a leitura cruzada durante a inicialização imediata.",
  ],
  "vize:croquis/cf/circular-reactive-dependency": [
    "Os cálculos reativos dependem uns dos outros em um ciclo.",
    "App possui e fornece count (A). CycleView deriva nextCount (B) e imediatamente escreve cada valor derivado de volta no mesmo count injetado. Cada escrita altera novamente a entrada do cálculo, criando um ciclo de realimentação de atualizações A → B → A. As identidades no grafo preservado abaixo representam essas duas referências, não variáveis sem relação que tenham nomes iguais.",
    "Remova o observador que escreve B de volta em A. App mantém a propriedade de count e o altera apenas por meio de sua ação explícita Increment; CycleView lê o nextCount derivado sem realimentar o resultado. As mesmas referências preservam apenas a dependência A → B.",
  ],
  "vize:croquis/cf/closure-captures-reactive": [
    "Um fechamento captura um valor reativo e não verá atualizações posteriores.",
    "`makeReader` copia `count.value` antes de criar o fechamento. O leitor computado retorna então esse número inicial sem ler uma dependência reativa.",
    "O fechamento lê `count.value` quando é chamado, de modo que a função de leitura computada pode rastrear a ref e atualizar `shown` após os incrementos.",
  ],
  "vize:croquis/cf/composable-outside-setup": [
    "Uma função de composição é chamada fora de `setup`.",
    "Importar `use-title.ts` registra `onMounted` antes que o setup de um componente esteja ativo. Chamar sua função exportada depois apenas retorna essa ref no escopo do módulo; isso não corrige a falta de vínculo com o ciclo de vida.",
    "Tanto a criação do estado quanto o registro do gancho passam para `useTitle`, que App chama de forma síncrona dentro de setup. O gancho de montagem agora pertence a essa instância de App.",
  ],
  "vize:croquis/cf/computed-side-effects": [
    "Uma função de leitura computada escreve no estado ou produz outro efeito colateral.",
    "Avaliar `doubled` escreve em `lastCalculated`, de modo que ler um valor computado também altera um estado separado. Isso vincula o efeito colateral ao momento em que a função de leitura de avaliação adiada é lida.",
    "A função de leitura apenas retorna o número derivado. Um observador separado fica responsável pela escrita em `lastCalculated` quando `count` muda, incluindo seu valor inicial.",
  ],
  "vize:croquis/cf/deep-import": [
    "Uma cadeia de importações é mais profunda do que o projeto permite.",
    "O ponto de entrada encaminha um valor simples por `level-one`, `level-two` e `level-three`, criando uma cadeia de importações desnecessariamente profunda para um projeto que deseja um limite público raso.",
    "O ponto de entrada usa `public-api.ts`, que reexporta o valor diretamente. O consumidor mantém o mesmo nome importado, enquanto a cadeia fica mais curta.",
  ],
  "vize:croquis/cf/destructuring-breaks-reactivity": [
    "Desestruturar um objeto reativo copia os campos e perde o rastreamento.",
    "A desestruturação comum do objeto `props` copia seu valor atual de `item`; isso é diferente da desestruturação direta de `defineProps()` no Vue 3.5.",
    '`toRef(props, "item")` preserva a conexão com a propriedade em `props`.',
  ],
  "vize:croquis/cf/di-outside-setup": [
    "`provide` ou `inject` é chamado fora de `setup`.",
    "`main.ts` chama o `provide` de componente sem uma instância de componente ativa. Por isso, o `inject` do filho não pode receber o valor pretendido do ancestral e usa `light`.",
    "App chama o provedor em seu setup antes de renderizar o filho. O filho agora herda o valor `dark` de seu componente ancestral.",
  ],
  "vize:croquis/cf/dom-access-without-next-tick": [
    "O DOM é lido antes de o Vue aplicar a atualização.",
    "O manipulador de clique incrementa `count` e imediatamente lê o parágrafo renderizado, antes de o Vue aplicar a atualização agendada do DOM. `sampled` pode conter a contagem anterior.",
    "Aguardar `nextTick()` após a escrita no estado permite que o Vue atualize o parágrafo antes de `readLabel` capturar seu texto.",
  ],
  "vize:croquis/cf/duplicate-id": [
    "O mesmo id de elemento é usado em mais de um componente.",
    'Os componentes alcançáveis de entrega e cobrança renderizam `id="postal-code"`, de modo que seus rótulos compartilham um destino ambíguo no documento.',
    "Cada componente chama `useId()` e vincula seu próprio valor ao rótulo e ao campo de entrada, preservando a associação sem repetir um ID literal.",
  ],
  "vize:croquis/cf/event-listener-leak": [
    "Um ouvinte de eventos é registrado e nunca removido.",
    "A montagem adiciona um ouvinte de redimensionamento de window que captura a ref de largura do componente, mas a desmontagem nunca o remove. Montagens repetidas podem reter ouvintes e estado sem uso.",
    "`onUnmounted` remove exatamente a mesma função `resize` registrada na montagem, encerrando o ciclo de vida do ouvinte externo dessa instância.",
  ],
  "vize:croquis/cf/event-modifier": [
    "Um ouvinte de eventos usa um modificador que o evento emitido não suporta.",
    "`.stop` pressupõe o método de propagação de um evento nativo no evento personalizado `save` do filho, cujo conteúdo não precisa ser um Event do DOM.",
    "Remova `.stop` do ouvinte do evento personalizado; trate a propagação nativa no ouvinte real do DOM quando necessário.",
  ],
  "vize:croquis/cf/hydration-risk": [
    "Este código de diagnóstico agrupa várias ocorrências de reatividade, incluindo uma prop copiada para uma ref. Ele não implica que toda expressão Date.now() seja detectada pela passagem de análise entre arquivos.",
    "O filho inicializa `ref(props.count)` uma vez, de modo que sua contagem local deixa de acompanhar mudanças posteriores na prop do pai. Este é o produtor atual prop-to-ref, não um exemplo geral de SSR não determinístico.",
    '`toRef(props, "count")` aponta para a prop em vez de copiar seu valor inicial para um estado independente.',
  ],
  "vize:croquis/cf/inherit-attrs-unused": [
    "`inheritAttrs: false` está definido e o componente nunca lê os atributos.",
    'O filho define `inheritAttrs: false`, mas nunca encaminha o atributo `class="notice"` do pai.',
    "Mantenha o controle explícito da herança e vincule `$attrs` ao destino `<main>` pretendido.",
  ],
  "vize:croquis/cf/inject-without-symbol": [
    "`inject` usa uma chave comum em vez de um símbolo `InjectionKey`.",
    'O consumidor injeta a chave de string sem tipo `"theme"`, que não oferece uma identidade de símbolo compartilhada com o provedor.',
    "O consumidor e o provedor importam o mesmo `ThemeKey` em vez de duplicar nomes de string.",
  ],
  "vize:croquis/cf/injected-async-mutation-race": [
    "Um valor injetado é alterado por uma tarefa assíncrona sujeita a uma condição de corrida.",
    "`CountLoader.vue` escreve um resultado aguardado diretamente no armazenamento injetado compartilhado com `CountSummary.vue`, permitindo que um trabalho desatualizado afete os dois consumidores.",
    "O carregador cancela trabalhos invalidados e emite apenas um resultado ativo. O provedor fica responsável pela alteração do armazenamento por meio de `applyLoadedCount`.",
  ],
  "vize:croquis/cf/lifecycle-outside-setup": [
    "Um gancho de ciclo de vida é registrado fora de `setup`.",
    "O ponto de entrada chama `installTitle()` antes de montar uma aplicação, de modo que `onMounted` é registrado sem um contexto de setup de componente ativo.",
    "Chamar o mesmo auxiliar de forma síncrona no setup de App vincula a função de retorno do ciclo de vida à montagem dessa instância.",
  ],
  "vize:croquis/cf/lifecycle-without-cleanup": [
    "Um gancho de ciclo de vida inicia um trabalho e nunca faz sua limpeza.",
    "A montagem registra um ouvinte de redimensionamento de window, mas a desmontagem nunca remove a mesma função de retorno.",
    "`onUnmounted` remove o ouvinte com o mesmo nome de evento e a mesma identidade de função usados por `addEventListener`.",
  ],
  "vize:croquis/cf/missing-required-prop": [
    "Uma prop obrigatória não é passada.",
    "O pai renderiza `<Child />` sem a prop obrigatória `title: string` do filho.",
    '`title="Hello"` fornece a prop obrigatória declarada pelo filho resolvido.',
  ],
  "vize:croquis/cf/missing-suspense": [
    "Uma dependência assíncrona é usada fora de um limite Suspense.",
    "`AsyncCard` tem await no nível superior, tornando seu setup assíncrono, mas App o renderiza sem um limite Suspense para coordenar essa dependência.",
    "App envolve o filho assíncrono em `Suspense` e fornece conteúdo alternativo de carregamento até que o setup do filho seja resolvido.",
  ],
  "vize:croquis/cf/module-scope-reactive": [
    "O estado reativo é criado no escopo do módulo e compartilhado por todos os chamadores.",
    "O módulo inicializa `count` uma vez, e as duas instâncias de Counter recebem a mesma ref. Clicar em uma altera os dois contadores, embora este exemplo pretenda manter um estado independente por instância.",
    "Criar a ref dentro de `createCounter` dá a cada chamada síncrona de setup um objeto de estado separado, de modo que cada botão possui seu contador.",
  ],
  "vize:croquis/cf/multi-root-attrs": [
    "Um componente com múltiplas raízes recebe atributos e não tem onde colocá-los.",
    "O filho tem raízes `<main>` e `<aside>`, de modo que o Vue não tem uma raiz única que possa receber automaticamente a classe do pai.",
    "Encaminhe `$attrs` explicitamente para `<main>`, mantendo a segunda raiz.",
  ],
  "vize:croquis/cf/mutated-after-escape": [
    "Um objeto reativo é alterado depois de escapar de seu proprietário.",
    "O arquivo retém o mesmo objeto passado a `publish`. Seu proprietário então altera o nome, mudando retroativamente o registro supostamente histórico para Grace. O parâmetro Readonly do TypeScript não copia o objeto.",
    "Publicar uma cópia simples separa o registro arquivado de Ada das edições posteriores do perfil reativo. A política de instantâneos do arquivo agora é mantida.",
  ],
  "vize:croquis/cf/non-reactive-provide": [
    "Um valor fornecido não é reativo, portanto os descendentes não verão atualizações.",
    "`ThemeProvider.vue` fornece um objeto simples. Alterar os campos desse objeto não dá ao consumidor que o injeta uma dependência reativa do Vue.",
    "O provedor envolve o tema em `ref`; a mesma referência injetada pode rastrear mudanças posteriores.",
  ],
};
