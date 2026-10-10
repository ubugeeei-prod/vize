---
title: "Regras de petite-vue"
---

# Regras de petite-vue

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-no-unsupported-directive) | [Incorreto](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-no-unsupported-directive-bad) · [Correto](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-no-unsupported-directive-good) | Proibir diretivas que o petite-vue não aceita |
| [`petite-vue/valid-v-effect`](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-valid-v-effect) | [Incorreto](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-valid-v-effect-bad) · [Correto](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-valid-v-effect-good) | Exigir uma expressão não vazia em v-effect |
| [`petite-vue/valid-v-scope`](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-valid-v-scope) | [Incorreto](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-valid-v-scope-bad) · [Correto](https://vizejs.dev/pt-BR/rules/petite-vue.html#petite-vue-valid-v-scope-good) | Exigir que v-scope vincule um objeto literal |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)
