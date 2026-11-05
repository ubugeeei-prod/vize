---
title: "Regras de SSR"
description: "Código que quebra ou se comporta de outro jeito durante a renderização no servidor e a hidratação."
---

# Regras de SSR

Código que quebra ou se comporta de outro jeito durante a renderização no servidor e a hidratação.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Padrão (2)

Adicionadas por `happy-path`, a predefinição padrão.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | Proibir variáveis globais exclusivas do navegador no contexto de SSR. |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | Proibir valores não determinísticos que causam divergências na hidratação. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
