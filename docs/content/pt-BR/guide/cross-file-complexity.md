---
title: Complexidade entre arquivos
---

<!-- Generated translation; source: guide/cross-file-complexity.md -->

# Complexidade entre arquivos

O relatório de complexidade entre arquivos da Vize é um resumo de grafo de projeto produzido pela Croquis. Não é uma regra diagnóstica
por si só; é uma pontuação explicável que ferramentas posteriores podem mostrar em relatórios,
Playground e futuras verificações baseadas em limiares.

O modelo mapeia três sinais de complexidade para o Vue:

- Contagem de caminhos do modelo: um ponto base por componente, mais `v-if`, `v-for`, e
  operadores booleanos em `v-if` expressões.
- Fluxo de controle aninhado: fluxo de templates mais profundo custa mais, incluindo o aninhamento que
  continua através dos componentes filhos.
- Fluxo de dados componente-fronteira: props, fornecer/injetar e arestas reativas permanecem
  visíveis como sinais transfronteiriços em vez de serem achatados em um único arquivo.

## Pontuações

O relatório expõe tanto sinais brutos quanto pontuações derivadas.

| Campo             | Significado                                                                                                                                       |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cyclomaticScore` | Contagem de base de componentes + `v-if` + `v-for` + operadores booleanos em `v-if`.                                                              |
| `cognitiveScore`  | Pontuação de aninhamento de templates de árvore de componentes entre `v-if`, `v-for`, e slots com escopo.                                         |
| `totalScore`      | Soma dos escorações dimensionais: fluxo do modelo, slots, perfuração de prop, estado global, fornecer/injeção, atrações de falha e grafo reativo. |
| `band`            | Balde voltado para humanos: `low`, `moderate`, `high`ou `extreme`.                                                                                |

A entrada bruta também mantém os números atrás da pontuação, incluindo:

| Sinal                                                          | Por que isso importa                                                                                                                 |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `componentTreeVIfMaxDepth`                                     | Caminhos condicionais longos entre componentes pai e filho precisam de mais estados para serem testados.                             |
| `componentTreeVForMaxDepth`                                    | Loops aninhados entre os limites dos componentes amplificam a complexidade de renderização e forma dos dados.                        |
| `componentTreeScopedSlotMaxDepth`                              | Slots com escopo combinam modelos de pai e filho, então a profundidade deles é acompanhada separadamente da contagem comum de slots. |
| `propDrillingEdgeCount`                                        | As arestas de prop indicam fluxo de dados transfronteiriço.                                                                          |
| `provideInjectMaxDepth` e `provideInjectReferenceCount`        | Árvores DI profundas ou amplas dificultam a inspeção local da propriedade.                                                           |
| `reactiveNodeCount`, `reactiveEdgeCount`e `reactiveCycleCount` | Grafos reativos capturam estados em nível de declaração, efeitos e ciclos propensos a perdas.                                        |

## Limites Componentes

A complexidade do template não se limita a um único SFC. O Croquis constrói primeiro um registro de módulos e um grafo de
de uso de componentes, depois percorre as arestas dos componentes com proteção de ciclo. Um pai `v-if` ao redor de uma criança, um pai
`v-for` ao redor de uma criança, e um slot com escopo filho contribuem todos para a mesma árvore de componentes
caminho de aninhamento.

Isso significa que um componente com aparência rasa ainda pode produzir uma pontuação alta quando avança em slots com escopo,
exerce props ou depende de um caminho profundo de fornecimento ou injeção. O modo Cross-file do Playground mostra a pontuação
ao lado dos diagnósticos, para que esses sinais fiquem visíveis durante a edição dos dispositivos.

## Pontos de interesse

O relatório também expõe hotspots ranqueados para que as ferramentas possam apontar para os arquivos/componentes que criam a pontuação
, em vez de mostrar apenas um número em nível de projeto. Cada hotspot carrega a entrada local de pontuação,
pontuações de dimensão, pontuação total e dimensão dominante. Use `dominantDimension` para explicar por que a entrada
está alta, depois use `input` para mostrar o sinal bruto que a impulsionou.

## Superfície Atual

A forma JSON pública está disponível na vinculação cross-file WASM como
`CrossFileResult.complexityReport` e `CrossFileResult.complexityHotspots`. O CLI não falha
ainda se baseia nesse ponto. Use o relatório como um sinal exploratório e depois promova limiares estáveis
somente após existirem referências específicas de cada projeto.
