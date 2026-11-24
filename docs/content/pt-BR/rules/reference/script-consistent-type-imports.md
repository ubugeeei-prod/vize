---
title: "script/consistent-type-imports"
description: "Uniformizar o uso de imports de tipo, contando os usos no template como usos de valor."
---

# `script/consistent-type-imports`

Uniformizar o uso de imports de tipo, contando os usos no template como usos de valor.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Sim | [Opções das regras](/rules/options.md) |

Aplicável a: Scripts em SFCs Vue e módulos TypeScript independentes (`.ts`).

## O que conta como uso de valor

Todo uso que o componente faz de um import conta, pela mesma análise de bindings que o compilador lê: uma expressão do template, uma tag de componente em qualquer grafia que o Vue resolve, uma diretiva personalizada, uma ref de template, um `v-bind()` em `<style>` e o outro bloco de script.

Um alias de `v-for` ou de slot com o mesmo nome oculta o import. Uma posição de tipo, como `(item as Foo)` no template ou o atributo `generic`, é um uso de tipo.

Quando um bloco não pode ser lido (um bloco externo com `src`, uma expressão que não é analisável, `eval` direto, marcação `<variant>` do Musea), a regra não relata nada para esse componente.

Um nome importado com `import type` que uma expressão do template ou um `v-bind()` em `<style>` avalia também é relatado, porque esse binding não existe em tempo de execução. A leitura no template fica a cargo de `vue/no-undefined-refs` quando essa regra está ativa. Tags de componente e diretivas não são relatadas: um componente registrado ou importado automaticamente pode renderizá-las.

## Opções

`prefer`, `fixStyle` e `disallowTypeAnnotations` são as opções do typescript-eslint. `emitDecoratorMetadata` (padrão `true`) indica se o projeto emite metadados de decorators, o que o lint não lê do `tsconfig.json`. Com `true`, um tipo que uma assinatura decorada serializa conta como uso de valor.

## A correção

Quando a divisão mantém uma importação de valor, sua ordem de avaliação é preservada. Converter toda a declaração em `import type` pode eliminar a inicialização do módulo com `verbatimModuleSyntax`. Uma declaração que precisa ser dividida e contém um comentário ou atributos de import é relatada sem correção.

## Incorreto

`Foo` é importado como valor, mas seu único uso é o argumento de tipo de `defineProps`. `Bar` não é relatado: o template lê `Bar.label`, que é um uso de valor.

```vue annotate="remove:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## Correto

`Foo` é importado com `import type`, portanto o import é removido na compilação. `Bar` continua sendo um import de valor porque o template o avalia.

```vue annotate="add:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import type { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-type-imports": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Diferenças em relação ao typescript-eslint

O typescript-eslint precisa do `vue-eslint-parser` para enxergar um uso no template, e o oxlint ignora arquivos `.vue` nesta regra. Uma assinatura decorada conta como uso de valor, a menos que `emitDecoratorMetadata` seja `false`; o typescript-eslint lê o `tsconfig.json` e, nesse caso, ignora todo arquivo que contenha um decorator. A correção nunca mescla com outra declaração. Um import somente de tipo que uma expressão do template ou um `v-bind()` em `<style>` avalia é relatado.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/consistent_type_imports.rs#L55)
