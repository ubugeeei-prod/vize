---
title: Regras HTML
---

<!-- Generated translation; source: rules/html.md -->

# Regras HTML

Essas regras cobrem a validade do HTML e a marcação semântica dentro dos templates do Vue. Elas são separadas de
regras diretivas específicas do Vue e das regras de acessibilidade, então as verificações de conformidade HTML podem ser ativadas
ou explicadas sozinhas.

## `html/id-duplication`

Relatórios duplicam IDs estáticos dentro de um único modelo.

Gravidade padrão: `error`
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

Bom:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## `html/deprecated-element`

Relatórios obsoletos de elementos HTML.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <center>Profile</center>
</template>
```

Bom:

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

## `html/deprecated-attr`

Relatórios de atributos HTML obsoletos.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <table border="1">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

Bom:

```vue
<template>
  <table class="summary">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

## `html/no-consecutive-br`

Reporta elementos consecutivos de `<br>` usados para layout.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

Bom:

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## `html/require-datetime`

Requer valores de `datetime` legíveis por máquina em `<time>`.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

Bom:

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

## `html/no-duplicate-dt`

Relatórios duplicam `<dt>` termos dentro do mesmo `<dl>`.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

Bom:

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

## `html/no-empty-palpable-content`

Reporta elementos vazios que se espera que exponham conteúdo visível ou de outra forma perceptível.
Elementos com texto, conteúdo infantil, `aria-label`, `aria-labelledby`, `v-html`ou `v-text`
são aceitos.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

Bom:

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```
