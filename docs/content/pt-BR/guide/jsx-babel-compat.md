---
title: Compatibilidade Babel JSX
---

<!-- Generated translation; source: guide/jsx-babel-compat.md -->

# Compatibilidade Babel JSX

> **Status:** opt-in e desligado por padrão. `compiler.jsxCompat` é lido pelo carregador de configuração e
> homenageado pelas `compileJsx` encadernações; Os plugins bundler ainda não encaminham para o compilador.
> A seção "Habilitando" abaixo cobre o que funciona hoje.

O Vize compila `.jsx` e `.tsx` através de suas próprias caixas de compilador, então a saída é
formato de compilador de template: uma árvore de blocos, `v-if` / `v-for` reduzida do JavaScript e patch
flags em cada nó. [`@vue/babel-plugin-jsx`](https://github.com/vuejs/babel-plugin-jsx) não faz nada
disso — emite chamadas `createVNode` nuas, nunca abre um bloco, deixa `&&`, `?:` e `.map()` como
JavaScript simples e, por padrão, não emite nenhuma bandeira de patch.

A maior parte dessa diferença é invisível em tempo de execução. O resto é para isso que esse switch existe: um projeto
migrando do plugin Babel precisa de uma forma de pedir a semântica do plugin em vez da do Vize.
`compiler.jsxCompat: "babel"` é esse interruptor.

Esta página é sobre **semântica de compatibilidade**. Para a API de autoria, a superfície de tipos e o seletor de saída
Vapor/VDOM, veja o [JSX & TSX guide](./jsx.md).

## Viabilizando

```json
{
  "compiler": {
    "jsxCompat": "babel"
  }
}
```

A chave aceita `"native"` (o padrão) e `"babel"`. Qualquer outro valor retorna ao `"native"`
em vez de falhar na compilação, correspondendo à forma como um `jsxMode` não reconhecido é tratado: uma configuração
valor isolada nunca deve bloquear a compilação.

O mesmo valor é aceito diretamente pelas ligações `compileJsx` , que é onde o modo entra
efeito hoje:

```js
import { compileJsx } from "@vizejs/native";

const result = compileJsx(source, {
  filename: "App.tsx",
  lang: "tsx",
  jsxCompat: "babel",
});
```

`@vizejs/wasm` expõe a mesma opção `jsxCompat`. Os plugins do bundler
(`@vizejs/vite-plugin`, `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`) atualmente passam
`jsxMode` e `vapor` para `compileJsx`, mas não `jsxCompat`, então configurar a chave de configuração sozin
ha ainda não muda o que o bundler emite. Essa fiação é rastreada em
[#3391](https://github.com/ubugeeei-prod/vize/issues/3391).

## Por que é opt-in e em nível de projeto

**Desligado por padrão.** `"native"` é o padrão e tem que permanecer o padrão. Invertê-lo
mudava silenciosamente a saída emitida de todos os projetos existentes do Vize, nenhum dos quais pedia babel
semântica.

**nível de projeto, sem forma por componente.** `jsxMode` podem ser selecionados por componente com um prólogo
`"use vue:vapor"` / `"use vue:vdom"`, porque os componentes VDOM e Vapor coexistem felizmente em
único módulo — cada um é uma função de renderização independente. O modo de compatibilidade não é assim. Ele
muda a forma de saída **em nível de módulo**: o plugin babel reescreve a expressão JSX no lugar, então
`const A = () => <div />` permanece como um `const A = …`, enquanto o Vize emite uma exportação `render` independente. Um módulo
compilado metade em modo compat e metade fora dele emitiria dois módulos
formas mutuamente incompatíveis a partir de um único arquivo. Portanto, o Compat é configurado uma vez para o projeto e deliberadamente não
prólogo diretivo.

## Mapeamento de opções de plugin

As opções do próprio plugin babel não têm grafia de arquivo de configuração no Vize. Cada um é um parâmetro de um ponto de entrada
`compile_jsx_with_babel_*` na caixa
[`vize_atelier_jsx`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_jsx),
e todos eles são inertes, a menos que `jsxCompat` seja `"babel"`.

| `@vue/babel-plugin-jsx` | Ponto de entrada do Vize                    |
| ----------------------- | ------------------------------------------- |
| `transformOn`           | `BabelJsxOptions::transform_on`             |
| `pragma`                | `compile_jsx_with_babel_pragma`             |
| `mergeProps`            | `compile_jsx_with_babel_merge_props`        |
| `isCustomElement`       | `BabelJsxCustomizations::is_custom_element` |
| `enableObjectSlots`     | `compile_jsx_with_babel_object_slots`       |
| qualquer combinação     | `compile_jsx_with_babel_customizations`     |

Duas opções de plugins não estão nessa tabela:

- **`optimize`** não tem equivalente ao Vize, porque a saída do Vize é sempre otimizada — que é o que
  o `optimize: true` do plugin produz. O padrão do plugin é `optimize: false`, e seu próprio
  README alerta que ativá-lo "pode pular certas rerenderizações", então o modo gap compat precisa
  fechar é a direção _não otimizada_ : emitindo saída sem patch-flag.
- **`resolveType`** não é implementado; veja "O que é adiado" abaixo.

`enableObjectSlots` padrão é `true` no plugin e na faixa de compat do Vize: um identificador solitário ou
expressão de chamada passada como único filho de um componente pode já ser um objeto slot, então é verificado
em tempo de execução. Passar `false` sempre trata esse valor como filho bruto do slot padrão.

## Onde o modo não se aplica

**Saída Vapor.** `@vue/babel-plugin-jsx` é um plugin da era VDOM: toda forma de saída que define é uma árvore
`createVNode`, e não tem equivalente ao Vapor. `jsxCompat: "babel"` combinado com
`jsxMode: "vapor"`, portanto, não tem um significado definido, e é rejeitado com um diagnóstico em vez de
silenciosamente ignorado:

```text
compiler.jsxCompat: "babel" is not supported with Vapor output: @vue/babel-plugin-jsx has no
Vapor equivalent. Use jsxMode "vdom" for babel compatibility, or drop jsxCompat to use Vize's own
Vapor semantics.
```

**saída SSR.** As opções do plugin descrevem árvores vnode do cliente. Portanto, a compilação SSR
não aplica a rota de Babel — nem os auxiliares `transformOn` e `enableObjectSlots`, nem o predicado
`isCustomElement`, nem `mergeProps: false`, nem qualquer redução exclusiva de Babel — e usa a própria
semântica SSR do Vize em vez de emitir uma mistura meio aplicada.

Ambas são respostas deliberadas, registradas na caixa para não serem discutidas novamente.

## O que é diferido

Duas linhas de corpus são registradas como `deferred` em vez de divergentes, porque cada uma está aguardando
trabalho de compilador não relacionado, e não no modo compat em si:

| Row                       | O que Babel faz                               | O que está esperando                                                                                                                                                                                                  |
| ------------------------- | --------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `options/resolve_type_on` | acrescenta `{ props: { … }, name: "A" }`      | Inferência guiada por tipo props/emits, que exige que a resolução do tipo seja acompanhada em [#1497](https://github.com/ubugeeei-prod/vize/issues/1497) / [#1502](https://github.com/ubugeeei-prod/vize/issues/1502) |
| `slots/dynamic_slot_name` | emite uma chave computada, `{ [n]: () => … }` | rebaixamento dinâmico de slots; Vize atualmente avisa e deixa o slot cair                                                                                                                                             |

## Como a compatibilidade é medida

A compatibilidade é medida em relação ao **plugin real**, não pela memória. O corpus é compilado por um
fixado `@vue/babel-plugin-jsx`, sua saída é registrada como verdade terrestre comprometida, e o conjunto Rust
snapshots dessa gravação ao lado da saída do Vize com um veredito explícito por linha.

| Artefato                                                          | Função                                                    |
| ----------------------------------------------------------------- | --------------------------------------------------------- |
| `crates/vize_atelier_jsx/tests/babel_compat/fixtures/corpus.json` | as entradas e as opções de plugins são compiladas com     |
| `crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs`           | Executa o corpus pelo plugin real                         |
| `crates/vize_atelier_jsx/tests/babel_compat_oracle.rs`            | snapshots da saída da babel ao lado da da Vize, por linha |
| `crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md`         | a forma em prosa da tabela de veredictos, e os totais     |

Os veredictos linha por linha, as divergências globais que valem para quase todas as linhas (formato do módulo, árvore
bloco, flags de patch, fluxo de controle não reduzido) e os totais de corrente estão todos em
[`BABEL_COMPAT_INVENTORY.md`](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md).
Esses totais são fixados pelo teste de `babel_compat_verdict_totals`, então não podem se desviar do
corpus — por isso esta página não cita nenhum deles. Leia-as na fonte.

Para regenerar ou verificar a gravação localmente:

```bash
node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check
cargo test -p vize_atelier_jsx --test babel_compat_oracle
node --test tests/tooling/babel-jsx-oracle.test.ts
```

## Veja também

- [JSX & TSX](./jsx.md) — a API de autoria, props e emits tipados, estilos de escopo e `jsxMode`.
- [Configuration](./configuration.md) — toda `compiler.*` chave e a ordem de busca no arquivo de configuração.
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) — um projeto JSX/TSX rodável.
