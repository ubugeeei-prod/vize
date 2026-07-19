---
title: JSX & TSX
---

<!-- Generated translation; source: guide/jsx.md -->

# JSX & TSX

> **Status:** O JSX/TSX é coberto pelo compilador, linter, verificador de tipos, LSP e formatador.
> Verificações conscientes de tipo permanecem com opt-in, então arquivos React `.tsx` nunca são tratados como Vue JSX por acidente.
> O HMR para módulos `.jsx`/`.tsx` independentes ainda é a principal lacuna de integração restante.

Vize compila componentes `.jsx` e `.tsx` Vue através das **mesmas caixas de compilador** que `.vue`
componentes de arquivo único — os backends VDOM e Vapor, análise semântica Croquis, verificação de tipo
Canon, Patina lint e o servidor de linguagem Maestro. Não há pipeline Babel separado nem
shim de fábrica JSX em tempo de execução: um componente JSX é rebaixado diretamente para uma função de renderização Vue (ou um template de Vapor
) pelo compilador nativo.

Isso significa que um componente `.tsx` Vue recebe a mesma compilação nativa de Rust, a mesma verificação de tipos e
a mesma experiência de editor que um SFC — só que criado como uma função tipada em vez de um `<template>`.

## Ativando JSX/TSX

`.jsx` e `.tsx` arquivos são roteados automaticamente pelos plugins bundler do Vize — não há flag de
de opt-in para compilar. Qualquer projeto que já use integração com bundler Vize recebe suporte a JSX/TSX
:

- `@vizejs/vite-plugin`
- `@vizejs/unplugin` (rollup / webpack / esbuild)
- `@vizejs/rspack-plugin`
- `@vizejs/nuxt`

```ts
// vite.config.ts — nothing JSX-specific is required
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

Sob o capô, os plugins chamam o ponto nativo/WASM `compileJsx` entrada (exposto a partir de
`@vizejs/native` e `@vizejs/wasm`), que reduz a fonte e retorna código de renderização além de qualquer CSS com escopo extraído
.

## API de autoria

Um componente Vize JSX/TSX é uma **função simples com parâmetros tipados**. Não há macros nem
`defineComponent` wrapper no caso comum — os tipos são lidos diretamente da função
assinatura e apagados da saída em tempo de execução (custo zero).

- **Os props** são o **primeiro parâmetro digitado**.
- **Emits e slots** são o **segundo parâmetro tipado**, um `Ctx<Emits, Slots>` fornecido pelo Vize
  contexto (com `emit`, `slots`e `attrs`, espelhando o contexto de configuração do Vue).
- **Os valores padrão de prop** vêm da **desestruturação dos padrões** no padrão de parâmetros — o
  compilador extrai esses elementos da desestruturação.

```tsx
import { computed, ref } from "vue";

type CounterProps = {
  label: string;
  start?: number;
};

type CounterEmits = {
  change: [value: number];
};

const Counter = ({ label, start = 0 }: CounterProps, { emit }: Ctx<CounterEmits>) => {
  const count = ref(start);
  const doubled = computed(() => count.value * 2);

  const increment = () => {
    count.value += 1;
    emit("change", count.value);
  };

  return (
    <section class="counter">
      <p>
        {label}: {count.value}
      </p>
      <p>Double: {doubled.value}</p>
      <button type="button" onClick={increment}>
        Increment
      </button>
    </section>
  );
};
```

Componentes apenas de props podem omitir completamente o segundo parâmetro:

```tsx
const Hello = ({ name }: { name: string }) => <h1>Hello, {name}!</h1>;
```

Os valores padrão são escritos como padrões de desestruturação; Não é necessária opção de `props` separada:

```tsx
const Badge = ({ count = 0 }: { count?: number }) => <span class="badge">{count}</span>;
```

O nome do componente é retirado da vinculação (`const Counter = …`) ou da declaração de função
(`function Card() { … }`), exatamente como você esperaria. Todo o resto é JSX semelhante ao React — aninhamento de elementos
, fragmentos (`<>…</>`), filhos de expressão e props de eventos como `onClick`. A única adição específica
de Vue é o elemento `<style scoped>` descrito [below](#scoped-styles).

> O formulário de autoria apenas por tipo acima é o caso comum suportado. Sintetizando `props` de execução
> Metadados e o formulário de configuração `defineComponent(() => () => vnode)` são acompanhamentos planejados.

## Superfície JSX suportada

O compilador reduz o JSX para o mesmo IR de alívio usado pelos templates SFC, depois envia esse IR para o
VDOM ou backend Vapor. Esses formulários são cobertos pela matriz de teste JSX/TSX:

- Fragmentos e elementos aninhados
- tags componente, tags de expressão de membro e tags intrínsecas HTML/SVG
- atributos estáticos, ligações dinâmicas `prop={expr}` , props booleanos abreviados e props spread
- gerenciadores de eventos, incluindo modificadores de opção no estilo Vue codificados no nome do prop
- `v-if`, `v-else-if`, `v-else`, `v-show`, diretivas personalizadas `v-*` e `v-model`
- filhos de expressão, ramos JSX lógicos, ramos JSX ternários e renderização de listas `.map(...)`
- slots escritos como filhos de objetos ou filhos de render-prop
- Sintaxe TSX: parâmetros tipados, anotações de retorno, chamadas JSX genéricas, casts e assertos não nulos
- `<style scoped>` extração; Interpolação `${expr}` literal de modelo é suportada para avançado
  casos, mas classes estáticas e variáveis CSS geralmente são mais claras

A forma canônica de lista é o JSX idiomático:

```tsx
import { computed, ref } from "vue";

type Todo = {
  id: string;
  title: string;
  done: boolean;
};

type TodoListProps = {
  todos: Todo[];
  initialActiveId?: string;
};

const TodoList = ({ todos, initialActiveId }: TodoListProps) => {
  const activeId = ref(initialActiveId ?? todos[0]?.id);
  const activeTodo = computed(() => todos.find((todo) => todo.id === activeId.value));

  return (
    <section class="todo-panel">
      <header>
        <h2>{activeTodo.value?.title ?? "Select a todo"}</h2>
      </header>

      <ul class="todo-list">
        {todos.map((todo, index) => (
          <li
            key={todo.id}
            class={{ done: todo.done, active: todo.id === activeId.value }}
            data-index={index}
          >
            <button type="button" onClick={() => (activeId.value = todo.id)}>
              <span>{todo.title}</span>
              {todo.id === activeId.value ? <strong>Active</strong> : <em>{index + 1}</em>}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
};
```

Os `.map(...)` aliases de callback (`todo`, `index`) são mantidos no escopo para verificador de tipos gerado e
TypeScript virtual LSP, então passar o curso, completar, diagnosticar e renomear operam nas mesmas ligações
você criou.

## Modo de saída: VDOM vs Vapor

Cada componente compila para a saída **Virtual DOM** (renderizador padrão do Vue) ou para a saída
[**Vapor**](https://blog.vuejs.org/posts/vue-vapor). O padrão é escolhido por configuração;
componentes individuais podem sobrepor-se.

### Configuração padrão

`compiler.jsxMode` define o backend global padrão para `.jsx`/`.tsx` componentes. Aceita `"vdom"`
ou `"vapor"` e o padrão é `"vdom"`.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode` é independente do `compiler.vapor`: `vapor` alterna o Vapor para `.vue` SFCs, enquanto `jsxMode`
controla o backend padrão para JSX/TSX. Um projeto pode manter SFCs no VDOM enquanto o JSX é usado por padrão para
Vapor, ou vice-versa. O plugin Vite também aceita `jsxMode` diretamente como opção de plugin, o que
sobrepõe a configuração compartilhada.

### Diretivas por componente

Um componente individual sobrescreve o padrão com um prólogo diretivo, espelhando `"use strict"`:

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

Como cada componente é roteado independentemente, um **único arquivo pode misturar ambos os backends**:

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### Precedência

O modo de saída de um componente resolve nesta ordem:

1. Uma diretiva `"use vue:vapor"` / `"use vue:vdom"` por componente.
2. O `compiler.jsxMode` padrão da configuração (ou da opção `jsxMode` do plugin).
3. O plano B embutido, `"vdom"`.

### Diagnósticos

Diretrizes mal formadas ou conflitantes são reportadas, em vez de ignoradas silenciosamente:

- Uma diretiva que começa com `"use vue:"` , mas não nomeia um modo conhecido (um erro de digitação como
  `"use vue:vdomx"`) é um erro de compilação.
  - Duas diretivas de modo conflitantes em um componente (`"use vue:vapor"` seguidas por `"use vue:vdom"`)
    são diagnosticados; A primeira diretriz ainda vence no modo resolvido.
- Prólogos não relacionados, como `"use strict"` ficam intocados.

## Estilos com mira

Um elemento `<style scoped>` **dentro do componente** é o equivalente em JSX ao bloco
`<style scoped>` de um SFC. Ele é extraído em tempo de compilação — nunca renderizado como um runtime `<style>`
vnode — seu CSS é reescrito em escopo com um id de escopo gerado `data-v-<hash>`, esse atributo de escopo
é injetado nos outros elementos do componente, e o CSS reescrito é emitido através do pipeline CSS do plugin
bundler. Isso funciona tanto nos backends VDOM quanto no Vapor, e ambos derivam o mesmo id de escopo
para um determinado componente.

Idiomaticamente, o elemento `<style scoped>` vai **por último**, após a marcação — correspondendo à ordem
`<template>` → `<style>` de um SFC — mas o compilador o extrai onde quer que apareça.

```tsx
type CardProps = {
  title: string;
};

const Card = ({ title }: CardProps) => (
  <article class="card">
    <h2>{title}</h2>

    <style scoped>{`
      .card {
        border: 1px solid currentColor;
        padding: 12px;
      }
    `}</style>
  </article>
);
```

### Valores dinâmicos de estilo

Prefira bindings de classes normais, objetos no estilo inline ou propriedades personalizadas CSS para estilo dinâmico em
JSX/TSX. Interpolações literais de modelo `${expr}` dentro de `<style scoped>` são suportadas e
verificadas por tipo, mas elas são uma saída de escape em vez do estilo principal de autoria:

```tsx
type BoxProps = {
  color: string;
  gap: number;
};

const Box = ({ color, gap }: BoxProps) => (
  <section
    class="box"
    style={{
      "--box-color": color,
      "--box-gap": `${gap}px`,
    }}
  >
    <p>content</p>

    <style scoped>{`
      .box {
        color: var(--box-color);
        gap: var(--box-gap);
      }
    `}</style>
  </section>
);
```

Um elemento `<style>` **sem** `scoped` é tratado como um elemento normal e renderizado como está —
não é extraído.

`<style scoped>{`.box { color: ${color}; }`}</style>` também funciona e é coberto pelo verificador de tipos,
mas mantenha-o para casos em que uma folha de estilos com escopo realmente precise referenciar uma expressão componente.
A sintaxe literal CSS `v-bind(...)` função usada dentro de um bloco SFC `<style>` não é uma forma suportada
de autoria dentro de um bloco estilo JSX.

## Formatação

Glyph formata o conteúdo do script JSX/TSX com o parser e formatador OXC. Em arquivos `.vue`,
`<script lang="jsx">`, `<script lang="tsx">``<script setup lang="tsx">` e são analisados como JSX/TSX
em vez de recorrer ao TypeScript simples, então filhos JSX e anotações TSX são formatados como
sintaxe real:

```vue
<script setup lang="tsx">
type CardProps = {
  title: string;
  items: string[];
};

const Card = ({ title, items }: CardProps) => (
  <section class="card">
    <h2>{title}</h2>
    {items.map((item) => (
      <span key={item}>{item}</span>
    ))}
  </section>
);
</script>
```

Módulos independentes de `.jsx`/`.tsx` são descobertos por `vize fmt` junto com arquivos `.vue` e
formatados com o mesmo tipo de tratamento de fonte JSX/TSX:

```bash
# Formats .vue, .jsx, and .tsx files by default
vize fmt src --write
```

## Verificação de tipos

A verificação de tipos JSX/TSX é **opt-in** por meio de `typeChecker.jsxTypecheck`, que por padrão ** é`false`**.
Está desligado por padrão de propósito: um repositório pode conter arquivos React `.tsx` que não devem ser verificados
tipo como Vue JSX.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  typeChecker: {
    enabled: true,
    jsxTypecheck: true,
  },
});
```

Quando ativado, `vize check` verifica o tipo `.jsx`/`.tsx` componentes do Vue via Canon. O arquivo virtual gerado
é TypeScript simples, não TSX, e preserva o contrato de componente criado:

- o primeiro parâmetro tipado permanece o tipo props;
- `Ctx<Emits, Slots>` permanece visível para o corpo de configuração e para as expressões JSX;
- Gerenciadores de eventos, props vinculados, `v-if`/`v-show`, diretivas personalizadas e interpolação no estilo Scoped
  expressões, quando usadas, são reemitidas como leituras normais do TypeScript;
- `v-model` alvos são reemitidos como autoatribuições graváveis, ou seja, ligações somente leitura ou sem valor
  são diagnosticados na ligação;
- `.map(...)` corpos de lista são reemitidos dentro do callback gerado, então os aliases valor/índice permanecem
  seus tipos de elementos inferidos.

Os diagnósticos são reportados nos **locais originais de origem** (tanto como JSON para a CLI quanto através
LSP), porque todo intervalo virtual-TS significativo remete para o intervalo de origem que você escreveu.

```tsx
type FieldProps = {
  model: {
    readonly value: string;
  };
};

const Field = ({ model }: FieldProps) => <input v-model={model.value} />;
```

No exemplo acima, `model.value` é marcado como alvo de atribuição. Se for somente leitura, o diagnóstico
cai em `model.value` na fonte TSX, não no código gerado.

```bash
# Type-check a project including its .jsx/.tsx Vue components.
# .jsx/.tsx files are collected only when typeChecker.jsxTypecheck is enabled.
vize check src
```

Componentes JSX/TSX independentes são inferiores ao simples TypeScript virtual para verificação. SFCs que contêm blocos
`<script lang="jsx">`, `<script lang="tsx">`ou `script setup` correspondentes são materializados como arquivos virtuais
`.vue.tsx` então o TypeScript analisa a sintaxe JSX no bloco de script. O LSP e a CLI compartilham
mesma diminuição, então um diagnóstico Corsa chega no mesmo intervalo de origem no editor e na linha de comando
.

## Editor / LSP

Abrir um componente Vue `.jsx`/`.tsx` em um editor respaldado por `vize lsp` fornece a mesma linguagem
recursos que um SFC — **sem necessidade de wrapper SFC**:

- Diagnósticos
- Paire
- Conclusão
- Go-to-definition
- Referências
- Renomeação
- Símbolos de documentos
- Tokens semânticos
- Ações do código
- Diagnósticos CSS embarcados para blocos `<style scoped>`

Características estruturais (símbolos de documentos, tokens semânticos, diagnósticos em estilo de escopo, ações de código) funcionam
a partir do documento analisado e estão sempre disponíveis. Recursos conscientes de tipos (diagnóstico, passagem do cursor,
completão, acesso à definição, referências, renomeação) são acessados somente quando `typeChecker.jsxTypecheck` está ativado
, então arquivos React `.tsx` nunca são tratados como Vue JSX no editor também.

## Linting

As regras de lint Patina da Vize rodam no JSX/TSX por meio de uma regra IR de custo zero **projetada diretamente do OXC
AST**. Regras orientadas a marcação não reconstroem um modelo sintético de SFC; eles leem elementos JSX e
atributos diretamente. Regras que precisam do formato do modelo Vue, como cheques de `.map(...)` lista de chaves, passam
sobre a árvore de relevo rebaixada. Regras semânticas são respaldadas por Croquis, a mesma camada de análise usada para
SFCs.

Isso significa que o linting JSX/TSX captura as mesmas classes de problemas sem depender de
parciais de correspondência de strings:

```tsx
const BrokenMedia = () => (
  <article>
    <img src="/avatar.png" />
    <button accessKey="s" autoFocus>
      Save
    </button>
  </article>
);
```

O exemplo acima é citado como fonte JSX:

- `a11y/img-alt` relata o desaparecimento `alt`;
- `a11y/no-access-key` relata `accessKey`;
- `a11y/no-autofocus` relata `autoFocus`.

As regras chave da lista entendem a forma idiomática JSX `.map(...)` :

```tsx
const KeyedList = ({ rows }: { rows: Array<{ id: string; label: string }> }) => (
  <ul>
    {rows.map((row) => (
      <li key={row.id}>{row.label}</li>
    ))}
  </ul>
);
```

Diagnósticos e correções mapeiam para as faixas de origem JSX, então a saída da CLI e as decorações do editor apontam para o elemento
ou prop que deve mudar.

```bash
# Lint .vue, .html, .jsx, and .tsx files
vize lint src
```

Veja [Static Analysis](./static-analysis.md) para o modelo de fiapos e verificação de tipo, e
[Rules](../rules/index.md) para a saída de regras concretas.

## Limitações

Fique atento às bordas atuais:

- **A verificação de tipos é opcional.** `typeChecker.jsxTypecheck` é `false` por padrão, então mistura Vue/React
  repositórios não roteiam acidentalmente o React TSX através do verificador JSX do Vue.
- **O HMR ainda não está cabeado para módulos `.jsx`/`.tsx` .** O compilador JSX atualmente emite um
  módulo de função de renderização em vez de um módulo completo de componente e objeto, então não há
  de fronteira do HMR do Vue para se conectar. Saída completa do módulo componente mais HMR que preserva o estado é uma continuação planejada; Até
  então, as edições em um componente `.jsx`/`.tsx` voltam a uma recarga normal.
- **O `v-bind(...)` CSS literal dentro de um bloco JSX `<style scoped>` não é suportado.** Use `${expr}`
  interpolação literal de modelo, que é o formulário suportado e verificado por tipo.

## Veja também

- [Configuration](./configuration.md) — a `compiler.jsxMode` e `typeChecker.jsxTypecheck` chaves,
  mais a configuração compartilhada completa.
- [Vite Plugin](./vite-plugin.md) — a integração recomendada para bundlers.
- [Static Analysis](./static-analysis.md) — como o lint e a verificação de tipos compartilham o pipeline do compilador.
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) —
  focou em exemplos de fonte JSX/TSX para cobertura de compilador, linter, verificador de tipos, LSP e formatador.
