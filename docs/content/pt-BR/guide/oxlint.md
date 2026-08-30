---
title: Oxlint Plugin
---

<!-- Generated translation; source: guide/oxlint.md -->

# Oxlint Plugin

`oxlint-plugin-vize` permite que o Oxlint execute diagnósticos do Vize Patina através do sistema de plugins JS da Oxlint.
Use quando quiser as regras JS e TS nativas de Rust da Oxlint junto com os diagnósticos de
conscientes do Vue da Vize em uma única execução.

Para o pipeline nativo de verificação de lint e tipos fora do Oxlint, veja
[Static Analysis](./static-analysis.md).

> [! IMPORTANTE]
> O pacote está disponível no npm, mas a integração ainda é inicial. Para terminal legível por humanos
> a saída, prefere `oxlint-vize -f stylish` enquanto a fidelidade do alcance original do SFC continua a melhorar.

## Instalação

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois adicione os pacotes:

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` resolve a vinculação nativa correspondente do Vize por meio de dependências opcionais, então
a maioria dos usuários não precisa instalar `@vizejs/native` separadamente.

## Uso Básico

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "no-console": "warn"
  }
}
```

Se você usar uma configuração JS ou TS Oxlint, o pacote também exporta mapas de regras predefinidos:

```js
import { configs } from "oxlint-plugin-vize";

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      helpLevel: "short",
      preset: "opinionated",
      typeAware: true,
    },
  },
  rules: configs.opinionatedWithTypeAware,
};
```

Exportações pré-definidas disponíveis incluem:

- `configs.recommended`
- `configs.essential`
- `configs.opinionated`
- `configs.nuxt`
- `configs.all`
- `configs.recommendedWithTypeAware`
- `configs.ecosystemWithTypeAware`
- `configs.opinionatedWithTypeAware`

## Comando Recomendado

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` é uma capa fina em torno de `oxlint` que suaviza `.vue` casos limite, sem scripts
enquanto a cobertura de plugins JS upstream continua melhorando.

## Configurações

As configurações são passadas por `settings.vize`:

```json
{
  "settings": {
    "vize": {
      "locale": "ja",
      "preset": "general-recommended",
      "helpLevel": "short",
      "typeAware": true
    }
  }
}
```

- `locale` controla a linguagem de diagnóstico.
- `preset` aceita `"general-recommended"`, `"essential"`, `"ecosystem"`, `"incremental"`, `"opinionated"`ou `"nuxt"`.
- `preset` padrão é `"general-recommended"`.
- `incremental` executa apenas as regras que você configura explicitamente.
- `helpLevel` aceita `"full"`, `"short"`ou `"none"`.
- `typeAware: true` permite regras de `vize/type/*` apoiadas pela Corsa durante passes compartilhadas de Patina.
- `corsaPath` seleciona o executável Corsa ou `tsgo` para linting consciente de tipos.
- `showHelp` e `settings.patina` ainda são aceitos por compatibilidade retroativa.

## Limitações Atuais

- `oxlint` brutas ainda podem perder alguns arquivos `.vue` sem `<script>` ou `<script setup>`. Uso
  `oxlint-vize` se seu projeto incluir SFCs apenas com templates.
  - Plugins JS Oxlint ainda ancoram os intervalos ao programa de script extraído, então template e style
    diagnósticos ainda não preservam os intervalos originais de SFC em todos os formatadores.
- `stylish` atualmente é o melhor formatador legível para humanos para saída mista Oxlint + Vize. JSON e
  outros formatos legíveis por máquina devem ser tratados como o melhor esforço para o modelo/estilo original
  posições.
- Exportações de regras conscientes de tipos são experimentais. Use uma configuração `*WithTypeAware` e defina
  `settings.vize.typeAware: true` quando você quer o passe compartilhado de arquivo inteiro para executar essas regras com vontade.

## Desenvolvimento Local

```bash
nix develop
vp install --frozen-lockfile
vp run --filter './npm/native' build
vp run --filter './npm/oxlint' build
```
