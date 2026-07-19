---
title: Regras de Acessibilidade
---

<!-- Generated translation; source: rules/accessibility.md -->

# Regras de Acessibilidade

Regras de acessibilidade são regras modelo de fila única da Patina. Eles detectam marcações difíceis de
usar com tecnologia assistiva ou navegação por teclado.

## `a11y/img-alt`

Requer um atributo `alt` em `<img>`.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <img src="/avatar.png" />
</template>
```

Bom:

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## `a11y/alt-text`

Requer texto alternativo para elementos de mídia que precisam de uma alternativa de texto.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

Bom:

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit" />
</template>
```

## `a11y/click-events-have-key-events`

Reporta os manipuladores de cliques em elementos interativos não nativos quando não há um manipulador de teclado presente.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <div role="button" @click="save">Save</div>
</template>
```

Bom:

```vue
<template>
  <button type="button" @click="save">Save</button>
</template>
```

## `a11y/interactive-supports-focus`

Requer elementos com papéis interativos para serem focáveis.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

Bom:

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## `a11y/label-has-for`

Exige que os rótulos estejam associados a um controle de formulário.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

Bom:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## `a11y/form-control-has-label`

Exige que os controles tenham um rótulo visível ou programático.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <input type="search" />
</template>
```

Bom:

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## `a11y/no-aria-hidden-on-focusable`

Relata elementos focáveis ocultos da tecnologia assistiva.

Gravidade padrão: `error`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

Bom:

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

## `a11y/no-static-element-interactions`

Relata manipuladores de mouse ou teclado sobre elementos estáticos.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <section @click="select">Select</section>
</template>
```

Bom:

```vue
<template>
  <button type="button" @click="select">Select</button>
</template>
```

## `a11y/tabindex-no-positive`

Reporta valores positivos de `tabindex` porque cria uma ordem de tabulação personalizada que é difícil de prever.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

Bom:

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/anchor-is-valid`

Exige que âncoras tenham alvos de link válidos.
Valores estáticos de `href` são verificados após a normalização do esquema, então `JaVaScRiPt:` e caracteres de controle de
decodificados em HTML dentro `java&#x0A;script:` ainda são reportados, enquanto esquemas semelhantes não correspondentes
permanecem permitidos.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <a href="#" @click="open">Open</a>
  <a href="JaVaScRiPt:void(0)">Open</a>
</template>
```

Bom:

```vue
<template>
  <button type="button" @click="open">Open</button>
  <a href="/docs/javascript:void">Docs</a>
</template>
```

## Regras Adicionais de Acessibilidade

`a11y/anchor-has-content` exige que os elementos âncora tenham conteúdo acessível. Padrão: `warning`.
Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/aria-props` proíbe atributos ARIA inválidos. Padrão: `error`. Presets: `happy-path`,
`nuxt`, `opinionated`.

`a11y/aria-role` exige funções válidas e não abstratas na ARIA. Padrão: `error`. Presets: `happy-path`,
`nuxt`, `opinionated`.

`a11y/aria-unsupported-elements` proíbe atributos ARIA em elementos que não os suportam.
Padrão: `error`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/heading-has-content` exige que elementos de título tenham conteúdo acessível. Padrão: `warning`.
Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/heading-levels` proíbe níveis de cabeçalhos pulados. Padrão: `warning`. Presets: `nuxt`,
`opinionated`.

`a11y/iframe-has-title` exige que `<iframe>` tenha um `title`. Padrão: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/landmark-roles` valida a colocação e a singularidade de papéis marcantes. Padrão: `warning`.
Presets: `nuxt`, `opinionated`.

`a11y/media-has-caption` exige legendas para elementos de mídia. Padrão: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/mouse-events-have-key-events` requer manipuladores de foco e desfoque quando são usados manipuladores de mouse.
Padrão: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-access-key` não permite o atributo `accesskey`. Padrão: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/no-autofocus` proíbe `autofocus`. Padrão: `warning`. Presets: `happy-path`, `nuxt`,
`opinionated`.

`a11y/no-distracting-elements` proíbe elementos distrativos como `<marquee>` e `<blink>`.
Padrão: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-i-for-icon` desencoraja o uso de `<i>` como elemento exclusivo de ícones. Padrão: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/no-redundant-roles` proíbe funções ARIA que dupliquem semântica nativa. Padrão:
`warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-refer-to-non-existent-id` relata referências ARIA a documentos de identidade desaparecidos. Padrão: `warning`.
Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-role-presentation-on-focusable` proíbe `role="presentation"` ou `role="none"` em
elementos focáveis. Padrão: `error`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/placeholder-label-option` exige valores de `<option>` desativados ou ocultos.
padrão: `warning`. Presets: `nuxt`, `opinionated`.

`a11y/role-has-required-aria-props` exige que os papéis incluam seus atributos exigidos pela ARIA.
Padrão: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/use-list` sugere elementos de lista para texto em tópicos. Padrão: `warning`. Presets: `nuxt`,
`opinionated`.
