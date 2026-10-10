---
title: "Regras de Vapor"
---

# Regras de Vapor

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`script/no-get-current-instance`](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-get-current-instance) | [Incorreto](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-get-current-instance-bad) · [Correto](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-get-current-instance-good) | Proibir getCurrentInstance() no modo Vapor (retorna null) |
| [`script/no-next-tick`](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-next-tick) | [Incorreto](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-next-tick-bad) · [Correto](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-next-tick-good) | Proibir o uso de nextTick() em componentes orientados a Vapor |
| [`script/no-options-api`](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-options-api) | [Incorreto](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-options-api-bad) · [Correto](https://vizejs.dev/pt-BR/rules/vapor.html#script-no-options-api-good) | Proibir padrões da Options API no modo Vapor |
| [`vapor/no-inline-template`](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-no-inline-template) | [Incorreto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-no-inline-template-bad) · [Correto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-no-inline-template-good) | Proibir o atributo obsoleto inline-template |
| [`vapor/no-vue-lifecycle-events`](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-no-vue-lifecycle-events) | [Incorreto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-no-vue-lifecycle-events-bad) · [Correto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-no-vue-lifecycle-events-good) | Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor) |
| [`vapor/prefer-static-class`](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-prefer-static-class) | [Incorreto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-prefer-static-class-bad) · [Correto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-prefer-static-class-good) | Preferir class estática a uma vinculação dinâmica de class para literais de string |
| [`vapor/require-vapor-attribute`](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-require-vapor-attribute) | [Incorreto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-require-vapor-attribute-bad) · [Correto](https://vizejs.dev/pt-BR/rules/vapor.html#vapor-require-vapor-attribute-good) | Sugerir a adição do atributo vapor a script setup |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)
