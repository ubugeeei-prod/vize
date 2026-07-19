---
title: Diagnóstico de Análise
---

<!-- Generated translation; source: guide/analysis-diagnostics.md -->

# Diagnóstico de Análise

Esta página explica como os diagnósticos do Vize são organizados. A referência detalhada da regra agora está na seção de Regras
, para que cada regra possa manter seu comportamento, severidade padrão, cobertura pré-definida e exemplos de
Ruim/Bom.

## Referência de Regra

- [Rules overview](../rules/index.md)
- [Vue rules](../rules/vue.md)
- [Accessibility rules](../rules/accessibility.md)
- [Type and script rules](../rules/type-and-script.md)
- [HTML rules](../rules/html.md)
- [SSR rules](../rules/ssr.md)
- [Vapor rules](../rules/vapor.md)
- [Cross-file rules](../rules/cross-file.md)
- [Musea and CSS rules](../rules/musea-and-css.md)

## Famílias Diagnosticadas

As regras de pátina são regras de fiapos em fila única. Eles usam nomes como `vue/require-v-for-key` e podem ser configurados
a partir de `vize.config.*`, da linha de cli, da API JavaScript e da ponte Oxlint.

Diagnósticos entre arquivos usam códigos `vize:croquis/cf/*`. Eles são emitidos por
`vize lint --cross-file` após o Vize construir um grafo de projeto, para que possam comparar provedores com injetores de
, rastrear IDs duplicados e detectar riscos de reatividade através dos limites dos componentes.

Diagnósticos conscientes de tipos utilizam o verificador TypeScript. Eles precisam da mesma configuração de projeto que
TypeScript vê através de `tsconfig.json`, incluindo `compilerOptions.types`, `paths`e referências de projeto
. Vize não exige uma lista de `globals` separada para esses nomes.

Diagnósticos de Musea e CSS são regras suportadas por bibliotecas. Eles são executados quando blocos de arte ou
de estilo Musea são analisados e documentados separadamente porque não fazem parte da regra padrão do template Vue
superfície.
