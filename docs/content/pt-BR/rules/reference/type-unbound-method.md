---
title: "type/unbound-method"
description: "Proibir referências a métodos sem o seu recetor (`this`)."
---

# `type/unbound-method`

Proibir referências a métodos sem o seu recetor (`this`).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

counter.increment extrai o método sem o recetor, então this perde-se quando ele é chamado.

```vue annotate="remove:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
const unbound = counter.increment;
</script>
```

## Correto

counter.increment() chama o método em counter, então this permanece ligado.

```vue annotate="add:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
counter.increment();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/unbound-method": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Diferenças em relação ao typescript-eslint

Métodos da biblioteca lidos do objeto global não são reportados. Isso vale para um receptor tipado `Window & typeof globalThis` ou `typeof globalThis`, incluindo `window.setTimeout`, `const { matchMedia } = window`, `self.requestAnimationFrame` e `document.defaultView?.setTimeout`. Uma operação da plataforma chamada sem receptor usa o objeto global, então essas referências funcionam como escritas. Um método que o seu código adiciona a `Window` continua sendo reportado. O mesmo vale para o `Window` de outro frame (`parent.setTimeout`) e para o método de um objeto alcançado pelo objeto global (`window.localStorage.getItem`). Com `checkGlobalObjectMethods: true`, essas referências são reportadas como no typescript-eslint.

Um objeto nativo que recebe uma declaração mesclada (`declare global { interface Math {} }`) mantém a isenção, e `RegExp.escape` fica isento por qualquer caminho até o construtor `RegExp`. A isenção segue as declarações da biblioteca do TypeScript, e não a versão do Node.js que executa o linter.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/unbound_method.rs#L5)
