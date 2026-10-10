export const portugueseRules0: Record<string, readonly [string, string, string]> = {
  "a11y/alt-text": [
    "Exigir texto alternativo para elementos de mídia",
    "O controle de envio com imagem fornece apenas a URL da imagem; ele não tem texto `alt` que descreva a ação.",
    '`alt="Submit search"` dá ao controle com imagem um nome acessível que descreve o envio da pesquisa.',
  ],
  "a11y/anchor-has-content": [
    "Exigir conteúdo acessível nos elementos de âncora",
    "O link `/settings` não tem texto nem outro conteúdo que lhe dê um nome, então seu destino não tem uma descrição acessível.",
    "O texto visível `Settings` fornece conteúdo para o link com o mesmo destino.",
  ],
  "a11y/anchor-is-valid": [
    "Exigir um href válido nos elementos de âncora",
    "A primeira âncora usa `#` para uma ação; a segunda usa uma URL JavaScript. Nenhuma fornece um destino de navegação comum.",
    "Um botão nativo executa `openPanel`, enquanto a âncora restante tem o destino real `/docs/javascript-urls`.",
  ],
  "a11y/aria-props": [
    "Proibir atributos ARIA inválidos",
    "`aria-lable` está escrito incorretamente e não é um atributo ARIA aceito.",
    "O atributo aceito `aria-label` fornece o nome do botão.",
  ],
  "a11y/aria-role": [
    "Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato",
    "`datepicker` não é um papel ARIA reconhecido para esta seção.",
    "A seção usa o papel reconhecido `dialog` e um rótulo que descreve a seleção da data.",
  ],
  "a11y/aria-unsupported-elements": [
    "Proibir atributos ARIA em elementos que não os aceitam",
    "O elemento de metadados tem `aria-hidden`, embora `meta` não aceite atributos ARIA.",
    "Remover o atributo ARIA mantém intacta a declaração do conjunto de caracteres.",
  ],
  "a11y/click-events-have-key-events": [
    "Exigir manipuladores de eventos de teclado junto aos eventos de clique",
    "A `div` não interativa tem um manipulador de clique, mas não trata eventos de teclado.",
    "Um `button` nativo permite a ativação pelo teclado para o mesmo manipulador `activate`.",
  ],
  "a11y/form-control-has-label": [
    "Exigir rótulos associados aos controles de formulário",
    "O campo de pesquisa não tem um rótulo que identifique o que o usuário deve inserir.",
    "Envolver o campo em um rótulo associa o texto visível `Search` ao controle.",
  ],
  "a11y/heading-has-content": [
    "Exigir conteúdo acessível nos elementos de título",
    "O `h2` acrescenta um nível de título, mas não tem conteúdo de título.",
    "`Billing settings` fornece o conteúdo do título de nível dois existente.",
  ],
  "a11y/heading-levels": [
    "Proibir que níveis de título sejam pulados",
    "A sequência de títulos passa diretamente de `h1` para `h3`, pulando o nível dois.",
    "Alterar o título de cobrança para `h2` preserva uma hierarquia consecutiva de títulos.",
  ],
  "a11y/iframe-has-title": [
    "Exigir um atributo title nos elementos iframe",
    "O quadro de finalização de compra tem uma URL de origem, mas não tem um `title` que descreva o conteúdo incorporado.",
    '`title="Checkout preview"` dá um nome ao conteúdo desse quadro.',
  ],
  "a11y/img-alt": [
    "Exigir o atributo alt nas imagens para garantir acessibilidade",
    "A imagem do avatar não tem o atributo `alt`.",
    '`alt="User avatar"` fornece uma alternativa textual para o avatar.',
  ],
  "a11y/interactive-supports-focus": [
    "Exigir que elementos com papéis interativos possam receber foco",
    "Atribuir a um `span` o papel de botão e um manipulador de clique não permite que o elemento receba foco pelo teclado.",
    "O botão nativo pode receber foco e mantém a mesma ação `open`.",
  ],
  "a11y/label-has-for": [
    "Exigir controles de formulário associados aos rótulos",
    "O rótulo separado não está associado por meio de `for` nem envolve o campo.",
    '`for="email"` corresponde ao ID do campo e associa explicitamente os dois elementos.',
  ],
  "a11y/landmark-roles": [
    "Validar a posição e a unicidade dos papéis de regiões de referência",
    "Dois elementos `main` declaram regiões de referência principais duplicadas no mesmo template.",
    "O painel permanece como a região de referência principal; a área de configurações se torna uma região de referência de navegação com nome.",
  ],
  "a11y/media-has-caption": [
    "Exigir legendas nos elementos de mídia",
    "O vídeo tem controles de reprodução, mas não tem uma faixa de legendas.",
    'Um `track` com `kind="captions"` fornece as legendas em inglês para o mesmo vídeo.',
  ],
  "a11y/mouse-events-have-key-events": [
    "Exigir eventos de foco e perda de foco junto aos eventos de mouse",
    "A visibilidade da prévia muda apenas por meio dos manipuladores de entrada e saída do mouse.",
    "As mesmas ações da prévia são executadas ao receber e perder o foco, e o botão pode receber foco pelo teclado.",
  ],
  "a11y/no-access-key": [
    "Proibir o uso do atributo accesskey",
    'O atalho `accesskey="s"` pode entrar em conflito com atalhos do navegador ou de tecnologias assistivas.',
    "Remover `accesskey` mantém disponível o botão Save comum.",
  ],
  "a11y/no-aria-hidden-on-focusable": [
    'Proibir aria-hidden="true" em elementos que podem receber foco',
    'O botão Close, que pode receber foco, é ocultado da árvore de acessibilidade com `aria-hidden="true"`.',
    "O botão permanece exposto e recebe um rótulo `Close`, em vez de ser ocultado.",
  ],
  "a11y/no-autofocus": [
    "Proibir o uso do atributo autofocus",
    "O campo solicita foco automático quando aparece.",
    "Remover `autofocus` evita essa solicitação de foco automático e mantém o campo de consulta.",
  ],
  "a11y/no-distracting-elements": [
    "Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt;",
    "O elemento `marquee` introduz texto que se move automaticamente.",
    "Um parágrafo exibe a mesma oferta sem o elemento marquee que causa distração.",
  ],
  "a11y/no-i-for-icon": [
    "Proibir o uso do elemento &lt;i&gt; para ícones",
    "O ícone é renderizado por meio de `i`, cuja semântica de texto não descreve uma ação representada apenas por um ícone.",
    "Um span decorativo oculta o glifo do ícone, enquanto o texto separado `Delete item` dá um nome à ação do botão.",
  ],
  "a11y/no-redundant-roles": [
    "Proibir papéis ARIA redundantes",
    'O botão nativo já tem o papel de botão, então `role="button"` repete sua semântica implícita.',
    "Remover o papel repetido mantém a semântica de botão fornecida pelo HTML.",
  ],
  "a11y/no-refer-to-non-existent-id": [
    "Proibir referências a IDs inexistentes",
    "`aria-labelledby` aponta para `save-label`, mas nenhum elemento declara esse ID.",
    "Adicionar o span correspondente resolve a referência e fornece o rótulo do botão.",
  ],
  "a11y/no-role-presentation-on-focusable": [
    'Proibir role="presentation" ou role="none" em elementos que podem receber foco',
    "O link de cobrança, que pode receber foco, solicita role=presentation, o que entra em conflito com seu papel interativo de link; os navegadores devem ignorar essa solicitação de apresentação.",
    "Remova a solicitação de apresentação conflitante e use o papel nativo de link e o destino de cobrança.",
  ],
  "a11y/no-static-element-interactions": [
    "Proibir manipuladores de eventos em elementos estáticos",
    "Uma seção estática recebe uma ação da tecla Enter sem ter um papel interativo.",
    "Um botão nativo executa a mesma ação usando um elemento interativo adequado.",
  ],
  "a11y/placeholder-label-option": [
    "Exigir disabled ou hidden na opção de orientação de um select",
    "A opção de orientação com valor vazio permanece selecionável como se fosse um valor de país.",
    "Adicionar `disabled` distingue a orientação da opção Japão, que pode ser selecionada.",
  ],
  "a11y/role-has-required-aria-props": [
    "Exigir as propriedades obrigatórias dos papéis ARIA",
    "O papel de caixa de seleção omite `aria-checked`, que informa o estado da caixa de seleção.",
    '`aria-checked="false"` fornece o estado exigido pelo papel de caixa de seleção.',
  ],
  "a11y/tabindex-no-positive": [
    "Proibir valores positivos de tabindex",
    "Um tabindex positivo de 3 cria uma ordem de foco personalizada antes dos controles comuns.",
    "O botão usa sua ordem de foco nativa sem um tabindex positivo.",
  ],
  "a11y/use-list": [
    "Sugerir elementos de lista para textos que parecem itens de lista",
    "Os itens das tarefas são parágrafos separados com marcadores de hífen digitados, em vez de elementos de lista.",
    "Uma lista não ordenada e seus itens expressam as mesmas tarefas com semântica de lista.",
  ],
  "css/no-display-none": [
    "Sugerir v-show em vez de display: none",
    "A declaração `.message` oculta o parágrafo local por meio de CSS, em vez de uma condição de visibilidade no template.",
    '`v-show="isSaved"` torna explícita a condição de visibilidade no parágrafo local e remove `display: none`.',
  ],
  "css/no-hardcoded-values": [
    "Sugerir variáveis CSS em vez de valores fixos escritos diretamente",
    "O botão incorpora números de espaçamento e uma cor hexadecimal diretamente nas declarações.",
    "As declarações fazem referência a propriedades personalizadas de espaçamento e cor com nomes, permitindo manter esses valores como tokens.",
  ],
  "css/no-id-selectors": [
    "Desencorajar o uso de seletores de ID no CSS",
    "`#submit` vincula a regra de estilo a um seletor de ID.",
    "A classe `.submit` fornece um ponto de aplicação de estilo reutilizável sem um seletor de ID.",
  ],
  "css/no-important": [
    "Desencorajar o uso de !important no CSS",
    "A declaração de cor substitui a prioridade normal da cascata com `!important`.",
    "A cor vem de uma propriedade personalizada sem uma declaração importante.",
  ],
  "css/no-utility-classes": [
    "Alertar sobre a implementação de classes utilitárias nos estilos de componentes",
    "Os seletores definidos usam nomes típicos de utilitários, como `.flex`, `.mt-4` e `.text-center`.",
    "Um seletor `.my-component` específico do componente agrupa seus estilos sob um único nome semântico.",
  ],
  "css/no-v-bind-performance": [
    "Alertar sobre o custo de desempenho do v-bind() no CSS",
    "A folha de estilos lê o valor variável de `offset` por meio do mecanismo `v-bind()` do CSS de SFC.",
    "O elemento recebe a transformação variável diretamente por meio da vinculação de estilo.",
  ],
  "css/prefer-logical-properties": [
    "Recomendar propriedades lógicas de CSS para melhorar o suporte à internacionalização",
    "`margin-left` fixa a margem em um lado físico, independentemente da direção de escrita.",
    "`margin-inline-start` acompanha o início da direção em linha.",
  ],
};
