---
title: Complexidade entre arquivos
---

<!-- Generated translation; source: guide/cross-file-complexity.md -->

# Complexidade entre arquivos

O relatório de complexidade entre arquivos da Vize é um resumo do grafo do projeto produzido pela
Croquis. Ele não é uma regra de diagnóstico por si só: é uma pontuação explicável que ferramentas
posteriores podem mostrar em relatórios, no Playground e em futuras verificações baseadas em limites.

O modelo mapeia três sinais de complexidade para o Vue:

- Contagem de caminhos do template: a complexidade ciclomática própria de cada componente, calculada
  pela análise S2 `template-complexity` da Davinci. Ela conta cada condição `v-if` / `v-else-if`, cada
  `v-for` e cada `&&`, `||`, `??` e `?:` nas expressões que o template avalia.
- Fluxo de controle aninhado: a complexidade cognitiva própria de cada componente. Ramos e laços
  custam mais quanto mais fundo estão aninhados em regiões `v-if`, `v-for` e de slots com escopo.
- Fluxo de dados nas fronteiras dos componentes: arestas de props, provide/inject e reativas continuam
  visíveis como sinais entre fronteiras, em vez de serem achatadas em um único arquivo.

A definição das métricas e os limites fixados no corpus estão em
[`complexity-metrics.md`](https://github.com/ubugeeei-prod/vize/blob/main/davinci-road/plan/complexity-metrics.md).

## Pontuações

O relatório expõe tanto os sinais brutos quanto as pontuações derivadas.

| Campo             | Significado                                                                                                                                  |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `cyclomaticScore` | Soma da complexidade ciclomática própria do template de cada componente.                                                                     |
| `cognitiveScore`  | Soma da complexidade cognitiva própria do template de cada componente.                                                                       |
| `totalScore`      | Soma das pontuações por dimensão: fluxo do template, slots, prop drilling, estado global, provide/inject, atributos fallthrough e grafo reativo. |
| `band`            | Faixa legível: `low`, `moderate`, `high` ou `extreme`.                                                                                       |

A entrada bruta também guarda os números por trás da pontuação, incluindo:

| Sinal                                                           | Por que importa                                                                                                  |
| --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `templateCyclomatic` e `templateCognitive`                      | As pontuações próprias dos templates, somadas entre os componentes.                                              |
| `templateMaxNesting`                                            | O aninhamento mais profundo de ramos, laços e slots com escopo dentro de um único template.                      |
| `templateScopedSlotCount`                                       | Slots com escopo acoplam os templates pai e filho, por isso são contados separadamente dos slots comuns.         |
| `templateUnknown`                                               | Expressões sem AST analisada (handlers com várias instruções, por exemplo). Elas não somam em nenhuma pontuação. |
| `propDrillingEdgeCount`                                         | Arestas de props indicam fluxo de dados que cruza fronteiras.                                                    |
| `provideInjectMaxDepth` e `provideInjectReferenceCount`         | Árvores de DI profundas ou amplas dificultam inspecionar a posse localmente.                                     |
| `reactiveNodeCount`, `reactiveEdgeCount` e `reactiveCycleCount` | Grafos reativos capturam estado em nível de declaração, efeitos e ciclos propensos a perdas.                     |

## Fronteiras dos componentes

A complexidade do template tem duas visões, e ambas vêm dos mesmos fatos:

- A complexidade **própria** (own) considera só o template do componente. A regra de lint
  `vue/max-template-complexity` julga essa visão, então extrair um ramo para um componente filho sempre
  reduz a pontuação do pai.
- A complexidade **renderizada** (rendered) é a pontuação própria do componente mais a pontuação
  própria de cada componente distinto que ele renderiza, seguindo o grafo de uso de componentes que a
  Croquis resolve pelos imports. Um filho renderizado em dois lugares conta uma vez. Um componente
  recursivo, e um grupo de componentes que renderizam uns aos outros, também contam uma vez.

`CrossFileResult.templateComplexity` lista todos os componentes com as duas visões, começando pela
árvore de renderização mais complexa. Para cada componente, também traz as construções que adicionam
complexidade, com linha e coluna.

Assim, um componente de aparência rasa ainda pode ter uma pontuação alta quando repassa slots com
escopo, faz prop drilling ou depende de um caminho profundo de provide/inject. O modo Cross-file do
Playground mostra a pontuação ao lado dos diagnósticos, para que esses sinais fiquem visíveis enquanto
você edita as fixtures.

## Regra de lint e achado do Doctor

`vue/max-template-complexity` reporta um `warning` quando o template próprio de um componente tem
complexidade ciclomática acima de 11 ou complexidade cognitiva acima de 16. Esses limites são o p95 do
corpus real da Vize, com 40.724 templates. O aviso aponta para a tag `<template>` e rotula as cinco
construções que mais adicionam complexidade.

Como os limites são um p95, cerca de um componente real em cada vinte os ultrapassa. Nenhum preset
ativa a regra, então ativá-la é uma decisão do projeto. Declare-a em `linter.rules` para ativá-la:

```ts
export default defineConfig({
  linter: {
    rules: {
      "vue/max-template-complexity": "warn",
    },
  },
});
```

`vize doctor` reporta um ponto crítico de complexidade de template, como notice, quando a
complexidade renderizada de um componente fica acima do p95 do corpus: 106 ciclomática ou 139
cognitiva.

A complexidade ciclomática soma 1 para cada decisão: cada condição `v-if` / `v-else-if`, cada
`v-for` e cada operador lógico e `?:`. A complexidade cognitiva conta assim:

- `v-if` e `v-for` somam 1 mais a profundidade de aninhamento.
- `v-else-if` e `v-else` somam 1 cada.
- Cada sequência de `&&`, `||` ou `??` soma 1.
- Um `?:` soma 1 mais a profundidade de aninhamento.
- O corpo de um slot com escopo conta como um nível mais profundo.

## Pontos críticos

O relatório também expõe pontos críticos ordenados, para que as ferramentas apontem os arquivos e
componentes que geram a pontuação em vez de mostrar só um número para o projeto inteiro. Cada ponto
crítico traz a entrada local da pontuação, as pontuações por dimensão, a pontuação total e a dimensão
dominante. Use `dominantDimension` para explicar por que a entrada é alta e depois `input` para
mostrar o sinal bruto que a gerou.

## Superfície atual

O formato JSON público está disponível na ligação WASM de análise entre arquivos como
`CrossFileResult.complexityReport`, `CrossFileResult.complexityHotspots` e
`CrossFileResult.templateComplexity`. A CLI ainda não falha builds por causa dessa pontuação. Use o
relatório como um sinal exploratório e só promova limites estáveis depois que existirem referências
específicas do projeto.
