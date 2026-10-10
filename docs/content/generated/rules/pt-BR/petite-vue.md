---
title: "Regras de petite-vue"
---

# Regras de petite-vue

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [Incorreto](#petite-vue-no-unsupported-directive-bad) · [Correto](#petite-vue-no-unsupported-directive-good) | Proibir diretivas que o petite-vue não aceita |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [Incorreto](#petite-vue-valid-v-effect-bad) · [Correto](#petite-vue-valid-v-effect-good) | Exigir uma expressão não vazia em v-effect |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [Incorreto](#petite-vue-valid-v-scope-bad) · [Correto](#petite-vue-valid-v-scope-good) | Exigir que v-scope vincule um objeto literal |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `petite-vue/no-unsupported-directive`

Proibir diretivas que o petite-vue não aceita

[Incorreto](#petite-vue-no-unsupported-directive-bad) · [Correto](#petite-vue-no-unsupported-directive-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**Incorreto**

`v-memo`, `v-slot:header` e a diretiva personalizada `v-my-directive` não constam na lista de diretivas aceitas pelo petite-vue. O script do petite-vue identifica este HTML como o dialeto pertinente.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**Correto**

A substituição usa a sintaxe aceita de `v-scope`, `v-effect`, `v-if`, `v-bind` e `v-on`, em vez de depender de diretivas não aceitas.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [Todas as regras](all.md)

### `petite-vue/valid-v-effect`

Exigir uma expressão não vazia em v-effect

[Incorreto](#petite-vue-valid-v-effect-bad) · [Correto](#petite-vue-valid-v-effect-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**Incorreto**

Cada `v-effect` não tem uma expressão executável: seu valor está ausente, vazio ou contém apenas espaços em branco.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**Correto**

Ambos os valores de `v-effect` contêm uma expressão: um atualiza `el.textContent` e o outro incrementa `count`. Esta regra verifica se há uma expressão não vazia, e não a lógica de negócio do efeito.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [Todas as regras](all.md)

### `petite-vue/valid-v-scope`

Exigir que v-scope vincule um objeto literal

[Incorreto](#petite-vue-valid-v-scope-bad) · [Correto](#petite-vue-valid-v-scope-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**Incorreto**

Os quatro valores não vazios de `v-scope` são um identificador, uma chamada, uma operação aritmética e um número; nenhum é analisado como um objeto literal.

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**Correto**

Um `v-scope` sem valor usa o escopo raiz. Os outros valores são objetos literais, incluindo o objeto entre parênteses, que a regra aceita.

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [Todas as regras](all.md)
