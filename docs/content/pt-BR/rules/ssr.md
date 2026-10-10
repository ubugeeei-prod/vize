---
title: "Regras da SSR"
---

# Regras da SSR

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](https://vizejs.dev/pt-BR/rules/ssr.html#ssr-no-browser-globals-in-ssr) | [Incorreto](https://vizejs.dev/pt-BR/rules/ssr.html#ssr-no-browser-globals-in-ssr-bad) · [Correto](https://vizejs.dev/pt-BR/rules/ssr.html#ssr-no-browser-globals-in-ssr-good) | Proibir variáveis globais exclusivas do navegador no contexto de SSR |
| [`ssr/no-hydration-mismatch`](https://vizejs.dev/pt-BR/rules/ssr.html#ssr-no-hydration-mismatch) | [Incorreto](https://vizejs.dev/pt-BR/rules/ssr.html#ssr-no-hydration-mismatch-bad) · [Correto](https://vizejs.dev/pt-BR/rules/ssr.html#ssr-no-hydration-mismatch-good) | Proibir valores não determinísticos que causam divergências na hidratação |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)
