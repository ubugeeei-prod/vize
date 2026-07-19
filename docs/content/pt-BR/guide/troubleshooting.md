---
title: Solução de problemas
---

<!-- Generated translation; source: guide/troubleshooting.md -->

# Solução de problemas

## Modos de Sintaxe de Template

Vize `compiler.templateSyntax` padrão para `"standard"`. O modo padrão aceita problemas de sintaxe de templates recuperáveis
, reporta avisos e os reescreve para resultados válidos.

Um caso comum de migração é a sintaxe auto-fechante em elementos HTML não nulos:

```vue
<template>
  <div />
  <span />
</template>
```

`<div />` e `<span />` não são elementos HTML válidos e auto-fechados. O modo padrão os reescreve como
elementos vazios, equivalentes a `<div></div>` e `<span></span>`, e emite um aviso. O modo estrito
os reporta como erros. O modo Quirks os mantém como sai que se fecha sozinho sem aviso.

Prefira escrever etiquetas finais explícitas:

```vue
<template>
  <div></div>
  <span></span>
</template>
```

Escolha um modo explicitamente ao migrar:

```ts
import vize from "@vizejs/vite-plugin";

export default {
  plugins: [
    vize({
      templateSyntax: "standard",
    }),
  ],
};
```

Use `"strict"` para falhar em sintaxe inválida, ou `"quirks"` quando um projeto depende do Vue aceitar essas tags
como folhas que se fecham sozinhas. Elementos válidos do vazio como `<input />`, `<img />`, `<br />`e
`<meta />` não precisam de individualidades.

## Resolução nativa de pacotes de tipos

`vize check` resolve pacotes do tipo Vue e Vite do projeto verificado antes de usar backups
agrupados, então as próprias versões `vue`, `@vue/runtime-dom`, `@vue`e `vite` do projeto impulsionam o projeto virtual gerado
. Para layouts incomuns de gerenciador de pacotes, defina `VIZE_VUE_PACKAGE`,
`VIZE_VUE_NAMESPACE_PACKAGE`, `VIZE_VUE_RUNTIME_DOM_PACKAGE`ou `VIZE_VITE_PACKAGE` para raízes explícitas de
de pacotes. `VIZE_RUNTIME_NODE_MODULES` também pode apontar para uma ou mais raízes `node_modules` como um caminho de busca
de recurso.
