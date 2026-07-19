---
title: oxlint-plugin-vize Alpha
description: Uma nova ponte de plugins Oxlint JS traz diagnósticos Vize Patina para uma única execução Oxlint para SFCs Vue.
---

<!-- Generated translation; source: blog/releases/2026-03-26-oxlint-plugin-vize-alpha.md -->

# `oxlint-plugin-vize` Alpha

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publicado em</span>
<span class="blog-meta-value">26-03-2026</span>
</span>
</span>
<a class="blog-author-card" href="https://github.com/ubugeeei">
<img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
<span class="blog-author-text">
<span class="blog-meta-label">Autor</span>
<span class="blog-meta-value">ubugeeei</span>
</span>
</a>
</div>

Hoje estou abrindo a primeira alfa de `oxlint-plugin-vize`, uma nova ponte plugin Oxlint JS para Vize Patina.

O objetivo é simples: manter [Oxlint](https://oxc.rs/docs/guide/usage/linter) como principal runner para as regras JavaScript e TypeScript, enquanto permite que o Vize contribua com diagnósticos específicos do Vue na mesma execução. Em vez de escolher entre Oxlint e Patina, esse alfa é sobre fazê-los trabalhar juntos.

## O que é

`oxlint-plugin-vize` permite que o Oxlint execute o Patina através do binding nativo do Vize, ainda usando o modelo de plugin JS e a configuração de regras do Oxlint.

Isso significa que um único `.oxlintrc.json` pode misturar regras como:

- Regras básicas de Oxlint, como `no-console`
- Plugin de `vue` embutido do Oxlint
- Regras vize como `vize/vue/require-v-for-key`
- Diagnósticos de Vue com patina atrás como `vize/vue/no-v-html` e `vize/vue/no-duplicate-attributes`

O plugin usa o namespace `vize` e lê as configurações de `settings.vize`.

## Por que esse alfa importa

A Patina já entende bem os templates do Vue, mas muitas equipes querem que o Oxlint fique no centro do fluxo de trabalho de lint.

Este alfa é o primeiro passo para essa forma:

- um comando de fiapos
- um arquivo de configuração
- um fluxo de saída
- Regras JavaScript e TypeScript nativas de Rust junto com diagnósticos Vue com modelos

Para projetos do Vue, essa combinação importa. Regras modelos, como falta de chaves de `v-for` ou uso inseguro de `v-html` , deveriam poder ficar ao lado das regras gerais do Oxlint, em vez de exigir um lint pass separado e um formato de relatório separado.

## Exemplo de Configuração

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "locale": "en",
      "helpLevel": "none"
    }
  },
  "rules": {
    "no-console": "warn",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "vize/vue/no-duplicate-attributes": "error"
  }
}
```

O alfa atualmente suporta:

- `settings.vize.locale` para linguagem diagnóstica
- `settings.vize.helpLevel` com `"full"`, `"short"`ou `"none"`
- `showHelp` para retrocompatibilidade
- `settings.patina` como um alias de compatibilidade enquanto `settings.vize` se torna a chave canônica

## Como Funciona

A ponte foi projetada com base no modelo de execução por regra da Oxlint, em vez de lutar contra ele.

- A primeira regra habilitada do Vize em um arquivo executa um passe nativo de Patina apenas para essa regra.
- Se uma segunda regra do Vize for ativada para o mesmo arquivo, o plugin atualiza para uma passagem Patina de arquivo completo compartilhada e reutiliza o resultado para as regras Vize restantes.
- O conteúdo do arquivo e os resultados das regras são armazenados em cache por arquivo e configurações durante toda a vida útil do processo Oxlint.

Esse design mantém a primeira regra barata, mas ainda evita trabalhos nativos redundantes quando várias regras do Vize estão ativas.

## Diagnóstico e Saída

Uma das partes difíceis dessa integração é o relatório de localização.

O sistema de plugins JS da Oxlint atualmente funciona a partir do programa de scripts extraído do Vue, enquanto muitos diagnósticos do Patina se originam em blocos `<template>` ou outros SFC. Nessa alfa, `oxlint-plugin-vize` mantém o bloco real do Vue e `line:column` em linha na mensagem de diagnóstico, assim a saída ainda te direciona para o lugar certo no SFC.

O repositório também inclui um pequeno exemplo `examples/oxlint-vize` para mostrar resultados mistos de:

- Diagnóstico de núcleo Oxlint
- Suporte embutido para Vue da Oxlint
- Diagnóstico Vize com patina traseira

## Limitações Atuais

Ainda é um alfa, e algumas limitações são importantes para destacar claramente:

- Plugins JS Oxlint atualmente dependem do script extraído do programa Vue, então arquivos sem `<script>` ou `<script setup>` ainda não invocam o plugin.
- Âncoras de diagnóstico ainda apontam para o programa script quando o Oxlint não consegue aceitar diretamente o intervalo original de templates.
- O pacote alfa inicial tinha como alvo o Nó 24+; as versões atuais suportam Node 22 e Node 24+.
- O suporte a plugins JS do Oxlint ainda está evoluindo, então algumas arestas aqui são restrições a montante, e não comportamentos apenas do Vize.

## Por que Alpha Agora

Queria colocar essa integração nas mãos das pessoas cedo, mesmo antes de todos os casos excepcionais serem polidos.

A forma central já parece útil:

- Vize traz inteligência específica de fiapos para a Vue
- Oxlint continua sendo o corredor de alto nível
- a superfície de configuração permanece pequena
- O modelo de desempenho permanece prioritário no nativo

Isso já é suficiente para começar a receber feedback real de usuários do Vue que querem uma pilha de fiapos mais rápida sem abrir mão de verificações que conseguem usar templates.

## O que vem a seguir

Os próximos passos são simples:

- melhorar o mapeamento de localização de templates à medida que o Oxlint expõe mais hooks de plugins conscientes do Vue
- Endureça o fluxo de instalação e publicação em torno de bindings nativos de plataforma
- expanda a documentação e exemplos para configurações reais de projetos
- continue refinando como o texto de ajuda da pátina é apresentado dentro dos formadores de Oxlint

Esse alfa não é o estado final. É a primeira ponte utilizável entre Oxlint e o linting Vue da Vize, e estou animado para ver para onde ela vai a seguir.
