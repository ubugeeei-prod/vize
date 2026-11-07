---
title: "vue/no-template-shadow"
description: "Proibir nomes de variáveis que ocultam variáveis de um escopo externo."
---

# `vue/no-template-shadow`

Proibir nomes de variáveis que ocultam variáveis de um escopo externo.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Não | Nenhuma |

Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra.

A verificação compara as vinculações de `v-for`, `v-slot` (incluindo `#name`) e slots com escopo da sintaxe legada dentro do template. Ela reporta o identificador que oculta uma vinculação anterior, incluindo declarações no mesmo elemento, na ordem dos atributos. Elementos irmãos têm escopos independentes. Chaves de propriedades e referências em valores padrão não são declarações, e `v-pre` torna sua subárvore inerte.

Nomes do script e de propriedades do componente estão fora da verificação atual: uma única vinculação no template não produz um aviso apenas porque uma vinculação no script tem o mesmo nome. Os intervalos de diagnóstico abrangem o identificador como foi escrito no código-fonte, incluindo grafias com escapes e entidades HTML.

## Incorreto

O v-for interno declara item novamente e oculta a variável item externa dentro do laço aninhado.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

## Correto

O laço interno declara child, deixando item disponível para a linha externa e child para a linha aninhada.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-shadow": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L35)
