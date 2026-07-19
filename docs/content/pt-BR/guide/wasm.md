---
title: Encadernações WASM
---

<!-- Generated translation; source: guide/wasm.md -->

# Encadernações WASM

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. APIs do WASM podem mudar sem aviso prévio.

`@vizejs/wasm` fornece bindings WebAssembly para executar o compilador Vue diretamente no navegador. Isso permite compilação, linting e formatação de SFC em tempo real sem servidor — ideal para playgrounds, documentação e ferramentas educacionais.

As ligações WASM são compiladas a partir do mesmo código Rust que as ligações CLI e NAPI (`vize_vitrine`), garantindo saída de compilação idêntica em todas as plataformas.

## Instalação

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois adicione o pacote:

```bash
vp install @vizejs/wasm
```

## API

### Compatibilidade com opções de compilador

O tipo `CompilerOptions` é o inventário de opções suportado para `compile`, `compileVapor`,
`parseTemplate`e `compileSfc`. Chaves de objeto desconhecidas são ignoradas na fronteira do JavaScript e
não são promessas de compatibilidade. `vueParserQuirks` permanece como um pseudônimo obsoleto para
`templateSyntax: "quirks"`; Uma `templateSyntax` explícita sempre tem prioridade. A `experimentalServerScript` de campo compartilhada de
Rust é reservada e não é exposta até que uma etapa do compilador WASM
a implemente. Cada fachada ignora campos suportados que não se aplicam à sua etapa de compilador:
`bindingMetadata` se aplica apenas à compilação direta de templates. Nomes de runtime se aplicam a módulos VDOM gerados
e à saída do cliente SFC (VDOM ou Vapor); os mapas de fonte aplicam-se à saída VDOM, incluindo o resultado do template
retornado por `compileSfc`. `outputMode` e `scriptExt` se aplicam apenas à compilação SFC.

### Compilar SFC

Compile um componente de arquivo único do Vue em JavaScript:

```javascript
import init, { compileSfc } from "@vizejs/wasm";

await init();

const result = compileSfc(
  `<template>
    <div>{{ msg }}</div>
  </template>

  <script setup lang="ts">
  const msg = ref('Hello Vize!')
  </script>`,
  { filename: "App.vue" },
);

console.log(result.script.code); // compiled <script> / <script setup>
console.log(result.template?.code); // compiled render function, when a template exists
console.log(result.css); // compiled styles, when styles exist
```

### SFC de fiapos

Regras específicas de fiapos para corrida Vue em um SFC:

```javascript
import init, { lintSfc } from "@vizejs/wasm";

await init();

const result = lintSfc(source, {
  filename: "App.vue",
  locale: "en", // 'en' | 'ja' | 'zh'
});

for (const diagnostic of result.diagnostics) {
  console.log(
    `${diagnostic.severity}: ${diagnostic.message} (line ${diagnostic.location.start.line})`,
  );
}
```

### Formato SFC

Formate um Vue SFC:

```javascript
import init, { formatSfc } from "@vizejs/wasm";

await init();

const formatted = formatSfc(source, { printWidth: 80 });

console.log(formatted.code);
```

## Inicialização

A função `init()` deve ser chamada uma vez antes de usar qualquer outra API. Ele carrega e instancia o módulo WebAssembly:

```javascript
import init from "@vizejs/wasm";

// Basic initialization
await init();

// With custom WASM URL (useful for CDN or bundler setups)
await init("https://cdn.example.com/vize_vitrine_bg.wasm");
```

## Casos de Uso

### Playgrounds

Construa playgrounds interativos de compilação do Vue que rodem inteiramente no navegador. O [Vize Playground](https://vizejs.dev/play) oficial usa as ligações WASM para compilação em tempo real:

```javascript
// React to editor changes and compile in real-time
editor.onChange((source) => {
  const result = compileSfc(source, {
    filename: "Playground.vue",
  });

  if (result.errors.length === 0) {
    preview.update({
      script: result.script.code,
      template: result.template?.code,
      css: result.css,
    });
  } else {
    diagnostics.show(result.errors);
  }
});
```

### Documentação

Incorpore exemplos ao vivo e editáveis do Vue na sua documentação:

```javascript
// Compile documentation examples on the fly
const examples = document.querySelectorAll("[data-vue-example]");
for (const el of examples) {
  const result = compileSfc(el.textContent, {
    filename: `example-${el.id}.vue`,
  });
  // Use result.script.code, result.template?.code, and result.css to mount it.
}
```

### Educação

Crie ferramentas interativas de exploração de compiladores que mostrem a saída da compilação em tempo real, ajudando os desenvolvedores a entender como os templates do Vue são transformados.

### CI/CD

Use bindings WASM para compilação leve em ambientes onde binários nativos não estão disponíveis (por exemplo, Cloudflare Workers, Deno Deploy, CI baseado em navegador).

## Construindo a partir da Fonte

```bash
# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Build WASM
cargo build --release -p vize_vitrine \
  --no-default-features \
  --features wasm \
  --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen \
  target/wasm32-unknown-unknown/release/vize_vitrine.wasm \
  --out-dir npm/wasm \
  --target web
```

## Internacionalização

Todas as APIs WASM que produzem diagnósticos (lint, erros de compilação) suportam mensagens localizadas:

| Código | Idioma           |
| ------ | ---------------- |
| `en`   | Inglês (padrão)  |
| `ja`   | Japonês (日本語) |
| `zh`   | Chinês (中文)    |

Passe a opção `locale` para qualquer API que produza diagnósticos:

```javascript
const result = lintSfc(source, {
  filename: "App.vue",
  locale: "ja", // Lint messages in Japanese
});

console.log(result.diagnostics);
```

## Tamanho do feixe

O módulo WASM inclui o pipeline completo do compilador Vue (analisador parser, analisador semântico, gerador de código) compilado para WebAssembly. O tamanho do bundle com gzip é aproximadamente **1,5 MB,** o que é adequado para carregamento não por caminho crítico (por exemplo, carregado após a interatividade da página).

Para uso em produção, considere o carregamento preguiçoso do módulo WASM:

```javascript
// Lazy-load the compiler only when needed
const compiler = await import("@vizejs/wasm");
await compiler.default(); // init()
const result = compiler.compileSfc(source, opts);
console.log(result.script.code, result.template?.code, result.css);
```
