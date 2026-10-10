export const portugueseShared: Record<string, string> = {
  "Shared project files": "Arquivos compartilhados do projeto",
  "Example qualification": "Qualificação do exemplo",
  "Not emitted": "Não emitido",
  "context-dependent": "dependente do contexto",
  None: "Nenhuma",
  "None; review related files and apply the repair":
    "Nenhuma; revise os arquivos relacionados e aplique a correção",
  "error / warning (with default)": "error / warning (com valor padrão)",
  "Analyzed component graph and the supported facts described below":
    "Grafo de componentes analisado e fatos suportados descritos abaixo",
  "CSS inside SFC style blocks": "CSS dentro dos blocos style de SFCs",
  "HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.":
    "Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.",
  "JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form":
    "Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup",
  "Musea .art.vue art, variant, and style blocks":
    "Blocos art, variant e style de arquivos .art.vue do Musea",
  "Nuxt configuration files (nuxt.config.ts)": "Arquivos de configuração do Nuxt (nuxt.config.ts)",
  "Reachable project declarations and imported components":
    "Declarações alcançáveis do projeto e componentes importados",
  "Type information in Vue SFC scripts and templates, for the constructs shown below":
    "Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo",
  "Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts":
    "Verificações de script voltadas ao Vapor; a ativação explícita também aplica a restrição a scripts comuns",
  "No per-code options; supported CLI findings accept severity overrides":
    "Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade",
  "crossFile; rule severity (off/warn/error)": "crossFile; severidade da regra (off/warn/error)",
  "Public explanation": "Explicação pública",
  Producer: "Produtor",
  "Cross-file index": "Índice de verificações entre arquivos",
  "Experimental Rust analyzer; not an individual CLI code":
    "Analisador Rust experimental; não é um código individual da CLI",
  "Contract; no current producer": "Contrato; sem produtor atual",
  "Project-specific lint ID": "ID de lint específico do projeto",
  "Contract only; no current producer": "Apenas contrato; sem produtor atual",
  "Rust analyzer; CLI uses a different surface or disables this pass":
    "Analisador Rust; a CLI usa outra interface ou desativa esta etapa",
  "The public CLI exposes the same pass with `vize lint --cross-file`. Displayed `vize:croquis/cf/*` codes use `croquis/cf/*` in `lint.vize.rules` (omit `vize:`). Information/hint diagnostics become CLI warnings. Related locations explain the source/consumer relationship.":
    "A CLI pública expõe a mesma etapa por meio de `vize lint --cross-file`. Os códigos exibidos como `vize:croquis/cf/*` usam `croquis/cf/*` em `lint.vize.rules` (omita `vize:`). Diagnósticos de informação ou sugestão tornam-se avisos da CLI. As localizações relacionadas explicam a relação entre a origem e o consumidor.",
  "A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.":
    "Um link que seja a única raiz de um SFC pode herdar seu target dos atributos do pai. Este exemplo usa um link aninhado, cujo target deve ser explícito.",
  "Checks non-interactive elements without an interactive role. Native buttons and elements with an interactive ARIA role are outside this rule's finding.":
    "Verifica elementos não interativos sem uma função interativa. Botões nativos e elementos com uma função ARIA interativa ficam fora dos diagnósticos desta regra.",
  "Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.":
    "Ative typeAware e esta regra explicitamente. Por padrão, números que podem ser nulos são proibidos, enquanto números não nulos são permitidos.",
  "Historical Vue 2.7 only: use matching Vue 2.7 and SFC compiler dependencies for this scenario. Vue 3 proxies track array index assignment, so `items[0] = next` is reactive in Vue 3 and is not a Vue 3 defect. This published code has no current producer.":
    "Apenas para o Vue 2.7 histórico: use dependências compatíveis do Vue 2.7 e do compilador de SFCs neste cenário. Os proxies do Vue 3 rastreiam atribuições a índices de arrays, então `items[0] = next` é reativo no Vue 3 e não é um defeito do Vue 3. Este código publicado não tem produtor atual.",
  "Intentional shared application stores may export reactive state. This scenario requires isolated state and does not claim every reactive export is invalid. No current producer emits this contract.":
    "Stores da aplicação compartilhados intencionalmente podem exportar estado reativo. Este cenário exige estado isolado e não afirma que toda exportação reativa seja inválida. Nenhum produtor atual emite este contrato.",
  "Module-scope reactive state is legal for intentional application stores. This example assumes component/request isolation; the published contract currently has no producer.":
    "Estado reativo no escopo de módulo é válido para stores intencionais da aplicação. Este exemplo pressupõe isolamento por componente ou requisição; o contrato publicado atualmente não tem produtor.",
  "Module-scope watchers are valid when their owner keeps and calls a stop handle or intentionally gives them application lifetime. This example requires component-owned lifetimes; the contract has no current producer.":
    "Watchers no escopo de módulo são válidos quando seu responsável mantém e chama uma função de interrupção ou lhes atribui intencionalmente a duração da aplicação. Este exemplo exige ciclos de vida pertencentes ao componente; o contrato não tem produtor atual.",
  "Pinia must be installed, and main.ts installs its plugin before mounting. Reading `store.doubled` directly inside a tracked computation or template is valid; the defect here is taking a plain snapshot. This contract currently has no producer.":
    "Pinia deve estar instalado, e main.ts instala seu plugin antes da montagem. Ler `store.doubled` diretamente dentro de um cálculo rastreado ou de um template é válido; o defeito aqui é capturar um valor instantâneo comum. Este contrato atualmente não tem produtor.",
  "Refs may legitimately be returned from composables or shared across scopes. This example explicitly requires a snapshot cache; it does not claim that unmount invalidates a ref. No current producer emits this contract.":
    "Refs podem legitimamente ser retornadas por composables ou compartilhadas entre escopos. Este exemplo exige explicitamente um cache de valores instantâneos; não afirma que a desmontagem invalide uma ref. Nenhum produtor atual emite este contrato.",
  "Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.":
    "Exige um arquivo .art.vue e o inventário de tokens mostrado abaixo. Não infere um token a partir de uma cor arbitrária.",
  "Suspense without a fallback is valid Vue syntax. This is a chosen loading-UI convention, not a compiler error; the published contract has no current producer.":
    "Suspense sem conteúdo alternativo é uma sintaxe válida do Vue. Esta é uma convenção escolhida de interface de carregamento, e não um erro do compilador; o contrato publicado não tem produtor atual.",
  "The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.":
    "Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.",
  "The boundary producer reads macros.is_async(), but source parsing currently records top-level await on the script-setup scope instead. The complete Bad/Good source pair below therefore produces no async-no-suspense finding through the current CLI. It explains the Suspense convention; supplying the missing macro fact is implementation follow-up work.":
    "O produtor de diagnósticos de limites lê macros.is_async(), mas a análise do código-fonte atualmente registra o await de nível superior no escopo de script-setup. Portanto, o par completo de código-fonte incorreto e correto abaixo não produz um diagnóstico async-no-suspense pela CLI atual. Ele explica a convenção de Suspense; fornecer o fato ausente da macro é um trabalho posterior de implementação.",
  "The complete Vue project below illustrates update feedback and its repair. It is not a qualified CLI finding witness: the diagnostic producer requires retained reactive-flow reference identities and edges, as shown by the accompanying graph. These sources do not establish that the current source path will emit this exact code. Dedicated tracked-ID graph finding controls remain separate from source grammar checks.":
    "O projeto Vue completo abaixo ilustra a realimentação de atualizações e sua correção. Ele não é uma comprovação qualificada de diagnóstico da CLI: o produtor de diagnósticos exige identidades de referências e arestas de fluxo reativo retidas, como mostra o grafo que acompanha o exemplo. Esses arquivos-fonte não demonstram que o processamento atual do código-fonte emitirá este código exato. Os controles específicos de diagnóstico de grafos com IDs rastreados permanecem separados das verificações de gramática do código-fonte.",
  "The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.":
    "O roteador completo instalado deve ser alcançável a partir de createApp(...).use(router) da aplicação. Tabelas de rotas desconhecidas ou dinâmicas não comprovam diagnósticos de nomes desconhecidos. Parâmetros ausentes geram avisos porque a navegação pode herdar um valor da rota atual.",
  "The concern is this lifecycle-dependent composable, not a blanket ban on ordinary utility functions or all Composition API calls outside setup. This contract has no current producer.":
    "A preocupação é este composable dependente do ciclo de vida, e não uma proibição geral de funções utilitárias comuns ou de todas as chamadas da Composition API fora de setup. Este contrato não tem produtor atual.",
  "The current producer scans template expressions such as JSON.parse(input). It does not report a throw statement that exists only in the script block.":
    "O produtor atual examina expressões de template, como JSON.parse(input). Não reporta uma instrução throw que exista apenas no bloco script.",
  "The example assumes IDs uniquely identify records. Comparing two references to the same reactive proxy remains valid; this contract has no current producer.":
    "O exemplo pressupõe que os IDs identifiquem os registros de forma única. Comparar duas referências ao mesmo proxy reativo continua válido; este contrato não tem produtor atual.",
  "The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.":
    "O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.",
  "The watcher must only derive the destination. Editable copies and callbacks with other side effects are allowed.":
    "O watcher deve apenas derivar o destino. Cópias editáveis e callbacks com outros efeitos colaterais são permitidos.",
  "This check compares explicit provider/consumer type annotations, not inferred literal value types. Keep the provider's `as string` annotation in this example.":
    "Esta verificação compara anotações explícitas de tipos do provedor e do consumidor, e não tipos inferidos de valores literais. Mantenha a anotação `as string` do provedor neste exemplo.",
  "This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.":
    "Este exemplo configura window.localStorage. A regra não tem uma lista padrão de bloqueio; ativá-la sozinha não reporta um membro.",
  "This example uses component provide/inject. `app.provide` and supported `app.runWithContext` injection are different valid ownership surfaces, not prohibited by this scenario. No current producer emits this contract code.":
    "Este exemplo usa provide/inject de componente. A injeção por `app.provide` e pelo `app.runWithContext` suportado são outras formas válidas de associação a um responsável, não proibidas por este cenário. Nenhum produtor atual emite este código de contrato.",
  "This illustrates a concrete eager-initialization cycle. A recursive Vue component or every circular import is not automatically erroneous. No current producer emits this contract code.":
    "Isto ilustra um ciclo concreto de inicialização imediata. Um componente Vue recursivo ou qualquer importação circular não é automaticamente incorreto. Nenhum produtor atual emite este código de contrato.",
  "This illustrates the published preference for purely derived state. Watchers remain appropriate for external effects or independently writable state; no current producer emits this contract.":
    "Isto ilustra a preferência publicada por estado puramente derivado. Watchers continuam adequados para efeitos externos ou estado gravável de forma independente; nenhum produtor atual emite este contrato.",
  "This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.":
    "Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.",
  "This is an explicit immutable-history ownership policy, not a general prohibition on passing or later mutating reactive objects. No current producer emits this contract.":
    "Esta é uma política explícita de propriedade de histórico imutável, e não uma proibição geral de passar objetos reativos ou de modificá-los depois. Nenhum produtor atual emite este contrato.",
  "This is an explicitly chosen project layout policy; it does not invent a supported depth threshold or option. There is no current diagnostic producer for this contract.":
    "Esta é uma política de organização de projeto escolhida explicitamente; não cria um limite de profundidade ou uma opção suportados. Não há um produtor atual de diagnósticos para este contrato.",
  "This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.":
    "Esta regra é um marcador provisório com callback vazio. Adicionar vapor seleciona a compilação Vapor; o linter atual não reporta este ID do catálogo pela ausência desse atributo.",
  "Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.":
    "As verificações com informações de tipos usam o runtime nativo do Corsa e o projeto TypeScript. `typeAware` sozinho não ativa uma regra opcional.",
  "Use a block-body arrow for the currently supported SFC filter. The underlying validator also handles method shorthand, but the current SFC prefilter does not reliably dispatch that shape.":
    "Use uma função de seta com corpo em bloco para o filtro de SFC atualmente suportado. O validador subjacente também trata a sintaxe abreviada de métodos, mas o pré-filtro atual de SFC não encaminha essa forma de modo confiável.",
  "Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.":
    "Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.",
  "Vue permits ref/reactive/computed outside component setup. The risk here is unwanted ownership/sharing under an explicit instance-isolation policy, not API illegality. This contract has no current producer.":
    "O Vue permite ref/reactive/computed fora do setup de componente. O risco aqui é a propriedade ou o compartilhamento indesejados sob uma política explícita de isolamento de instâncias, e não um uso inválido da API. Este contrato não tem produtor atual.",
};
