---
title: TypeScript Content Mapper
---

<!-- Generated translation; source: guide/content-mapper.md -->

# TypeScript Content Mapper

Content Mappers são a superfície de plugins do TypeScript para verificar tipos de arquivo que o
compilador não consegue analisar sozinho — o
[roadmap da API do TypeScript 7.1](https://github.com/microsoft/typescript-go/issues/4830) os
identifica como o substituto dos plugins do TS Server necessário para o Vue. A API foi mesclada na
branch main do `typescript-go` em
[microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712).

O Vize inclui um content mapper conforme dentro do pacote npm `vize`: um build do `tsgo` com
suporte a content mappers inicia `vize content-mapper` e verifica arquivos `.vue` diretamente —
hover, ir-para-definição, renomear, completions e diagnósticos são todos mapeados de volta para o
SFC original, sem materializar um projeto `.vue.ts` paralelo.

> **⚠️ Preview:** Os Content Mappers foram mesclados upstream, mas ainda não estão nos pacotes
> TypeScript 7 platform lançados. Até que um release inclua o protocolo, compile um binário
> TypeScript nativo com Content Mapper a partir da main do `typescript-go` e mantenha o
> [`vize check`](./cli.md#check) como o caminho de verificação de tipos suportado.

## Configuração

Instale o `vize` e declare o mapper no seu `tsconfig.json`:

```bash
vp install -D vize
```

```json
{
  "compilerOptions": {
    "module": "preserve",
    "strict": true
  },
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"]
    }
  ],
  "include": ["src"]
}
```

Executar processos de mapper externos exige opt-in explícito:

```bash
tsgo --runExternalCode --noEmit -p tsconfig.json
```

No VS Code, a extensão do Vize registra o suporte a `.vue` no host content-mapper do TypeScript 7
automaticamente em workspaces confiáveis — o mesmo mapper passa a alimentar o editor.

## Opções

Uma entrada de mapper aceita um objeto `options`:

```json
{
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"],
      "options": { "optionsApi": false }
    }
  ]
}
```

| Opção        | Padrão | Propósito                                                        |
| ------------ | ------ | ----------------------------------------------------------------- |
| `optionsApi` | `true` | Resolver bindings de instância da Options API do Vue em templates |

Opções inválidas nunca quebram o build: o Vize as reporta como diagnósticos de opção posicionados
dentro do seu tsconfig (`vize1`–`vize3`) e continua com os padrões. O Vize também declara
dependência da opção de compilador `noUnusedLocals` do projeto, então o relatório de locais não
usados dentro de `<script setup>` segue a configuração de cada projeto.

## Diretivas de Template

`@ts-expect-error` funciona normalmente dentro de blocos `<script>`, que passam adiante sem
alteração. Expressões de template não podem carregar comentários TS, então o Vize mapeia as
diretivas de comentário HTML padrão do Vue através do protocolo:

```vue
<template>
  <!-- @vue-expect-error -->
  {{ count.toFixed(true) }}

  <!-- @vue-ignore -->
  {{ untypedThirdPartyValue.field }}
</template>
```

- `<!-- @vue-expect-error -->` suprime diagnósticos do TypeScript na próxima linha do template e
  reporta `vize4: Unused '@vue-expect-error' directive` quando nada foi suprimido.
- `<!-- @vue-ignore -->` suprime silenciosamente.

Uma diretiva se aplica ao restante da própria linha quando há conteúdo após o comentário; caso
contrário, à próxima linha não vazia.

## Protocolo

O Vize fala o protocolo v1 de content mapper conforme mesclado upstream: codificação de posição
UTF-8, ciclo de vida `openProject`/`closeProject` por projeto e saída virtual `.tsx` para que
tanto o TypeScript quanto o JSX embutido sejam analisados corretamente. A conformidade é garantida
no CI contra uma revisão fixada do `typescript-go`, que compila o compilador upstream exato e
executa as suítes completas de CLI, build e LSP através dos artefatos npm empacotados.

Códigos de diagnóstico reportados sob a fonte `vize`:

| Código  | Significado                                        |
| ------- | --------------------------------------------------- |
| `vize1` | O valor das opções do mapper não é um objeto        |
| `vize2` | Opção de mapper desconhecida                        |
| `vize3` | Opção de mapper com tipo errado                     |
| `vize4` | Diretiva `@vue-expect-error` não usada              |

## Limitações

- Requer um `tsgo` compilado da main do `typescript-go` até que um release do TypeScript 7 inclua a
  API.
- Declaration maps para entradas mapeadas aguardam
  [microsoft/typescript-go#4860](https://github.com/microsoft/typescript-go/issues/4860).
- `vize check` continua sendo o caminho de verificação de tipos suportado em produção enquanto a
  API upstream está em preview.
