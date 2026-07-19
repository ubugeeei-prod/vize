---
title: Musea
---

<!-- Generated translation; source: guide/musea.md -->

# Musea

> **⚠️ Trabalho em andamento:** Musea ainda está evoluindo. Formatos de arquivo, APIs e comportamento da interface podem mudar.

O Musea é a cadeia de ferramentas de arquivos de arte e galeria de componentes da Vize.

- `vize_musea` é o núcleo Rust para análise de `*.art.vue`, geração de documentos, construção de paletas de props,
  autogeração de variantes e preparação de dados VRT.
- `@vizejs/vite-plugin-musea` é a galeria recomendada e o fluxo de trabalho de dev-server atualmente.
- `musea-vrt` é o CLI para snapshots de regressão visual, auditorias a11y, aprovações, limpeza e
  gerava arquivos de arte.

## Visão geral

![Musea Component Gallery — Home](/musea-home.png)

O Musea usa arquivos `*.art.vue` para descrever variantes de componentes com sintaxe nativa do Vue.

## Instalação

Instale `vp` uma vez a partir do [Vite+ install guide](https://viteplus.dev/guide/install), depois adicione o pacote:

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## Uso recomendado: Plugin Vite

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

Execute seu servidor de desenvolvimento Vite normal e abra a rota Musea configurada:

```bash
vp dev
```

```txt
http://localhost:5173/__musea__
```

Se você instalar o pacote `vize` npm, `vp exec vize musea` é um wrapper de conveniência no Vite:

```bash
vp exec vize musea
vp exec vize musea --build
```

## Configuração Compartilhada

`musea()` opções sobrepõem a configuração compartilhada. Coloque os padrões estáveis do projeto no `vize.config.ts` e mantenha
configurações apenas de pré-visualização no `vite.config.ts`.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

A configuração compartilhada atualmente cobre `include`, `exclude`, `basePath`, `storybookCompat`e
`inlineArt`. Passe `previewCss`, `previewSetup`, `tokensPath`, `theme`e `storybookOutDir`
diretamente para `musea()`.

## Arquivos de Arte

```art-vue
<script setup lang="ts">
import { ref } from "vue";

defineArt("./MyButton.vue", {
  title: "MyButton",
  category: "Components",
  status: "ready",
  tags: ["button", "ui", "input"],
});

const pressed = ref(false);
</script>

<art>
  <variant name="Default" default>
    <MyButton type="button" :pressed="pressed">Click me</MyButton>
  </variant>

  <variant name="Outlined">
    <MyButton type="button" outlined :pressed="pressed">Click me</MyButton>
  </variant>
</art>
```

`defineArt(source, options)` é uma macro de compilador. Ele declara o componente que o Musea deve carregar,
além de metadados que antes permaneciam em `<art>`. Prefere uma cadeia de caminho de componente relativa, como
`defineArt("./MyButton.vue", { title: "MyButton" })`; O Musea importa esse componente em código gerado
em tempo de execução e o servidor de linguagem usa a mesma fonte para inferência de prop e slot.
A cadeia de origem participa da conclusão de caminhos, diagnósticos de arquivos não resolvidos, links de documentos e
go-to-definition.

`<art title="..." component="...">` ainda funciona para compatibilidade, e atributos explícitos `<art>`
sobrepor `defineArt` metadados quando ambos estão presentes.

### Estado variante-local

O estado da `<script setup>` raiz é isolado por padrão por variante. Cada variante recebe sua própria configuração
instância, então referências e valores computados em uma variante não vazam para outra:

```art-vue
<script setup lang="ts">
import { computed, ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>

<art>
  <variant name="Base" default>
    <Counter :count="count" />
  </variant>
  <variant name="Doubled">
    <Counter :count="doubled" />
  </variant>
</art>
```

Use `<script setup isolate="false">` apenas quando o arquivo de arte precisar intencionalmente de uma configuração
instância compartilhada em cada variante:

```art-vue
<script setup lang="ts" isolate="false">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const sharedCount = ref(0);
</script>
```

### Anatomia

| Elemento / Macro                 | Propósito                                                  |
| -------------------------------- | ---------------------------------------------------------- |
| `defineArt(source, options)`     | Metadados de componente alvo e arte                        |
| `defineArt(...).title`           | Nome de exibição                                           |
| `defineArt(...).category`        | Agrupamento na barra lateral                               |
| `defineArt(...).status`          | Distintivo de status opcional                              |
| `defineArt(...).tags`            | Tags de busca e filtragem                                  |
| `<script setup>`                 | Estado de configuração local variante por padrão           |
| `<script setup isolate="false">` | Estado compartilhado de configuração em todas as variantes |
| `<art>`                          | Bloco de variantes raiz                                    |
| `<art title component ...>`      | Atributos de metadados de compatibilidade                  |
| `<variant>`                      | Variação de componentes nomeados                           |
| `default`                        | Marca a variante padrão                                    |
| `args`, `viewport`, `skip-vrt`   | Configuração variante opcional                             |

Mantenha os arquivos de arte próximos ao componente quando variantes fizerem parte do contrato do componente:

```txt
src/components/Button.vue
src/components/Button.art.vue
```

Use um diretório `stories` ou `art` separado quando um sistema de projeto possui muitos exemplos transversais,
ou quando a descoberta automática de componentes Nuxt escanear o diretório de componentes:

```txt
src/components/Button.vue
stories/forms/Button.art.vue
stories/navigation/Menu.art.vue
```

## Arte Inline

Quando `inlineArt` está ativado, arquivos de `.vue` normais que contenham um bloco `<art>` podem aparecer na galeria de
. Isso é útil para componentes pequenos onde exemplos devem estar no mesmo arquivo.

```ts
musea({
  inlineArt: true,
});
```

Dentro da arte em linha, use `<Self>` para renderizar o componente host.

## Recursos da Galeria

![Musea Component Detail — Variants](/musea-component.png)

A musea pode emergir:

- Metadados de componentes e variantes
- Geração de paletas de adereços
- Visualizações de tokens de design
- Verificações de acessibilidade
- Ajudantes no teste de regressão visual
- Saída compatível com storybook quando solicitado

## Paleta de Adereços

![Musea Props Panel](/musea-props.png)

O pipeline de paletas pode inferir controles interativos a partir de metadados de componentes e definições de arte.

## Design Tokens

![Musea Design Tokens](/musea-tokens.png)

`@vizejs/vite-plugin-musea` pode ingerir um arquivo de token compatível com o Dicionário de Estilos e expô-lo em
a interface da galeria.

```ts
musea({
  tokensPath: "src/tokens.json",
});
```

## Configuração de Pré-visualização

Você pode injetar CSS do projeto e pré-visualizar o código de configuração:

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

Isso é útil para instalar plugins como `vue-i18n` ou `vue-router` no iframe de pré-visualização.

```ts
// musea.preview.ts
import type { App } from "vue";
import { createI18n } from "vue-i18n";

export default function setup(app: App) {
  app.use(
    createI18n({
      legacy: false,
      locale: "en",
      messages: {
        en: {},
      },
    }),
  );
}
```

## Teste de Regressão Visual

O pacote expõe o `musea-vrt` binário:

```bash
vp exec musea-vrt --base-url http://localhost:5173
vp exec musea-vrt --update
vp exec musea-vrt --ci --json
vp exec musea-vrt --a11y
vp exec musea-vrt approve
vp exec musea-vrt approve "Button/*"
vp exec musea-vrt clean
```

O fluxo típico de CI inicia o servidor Vite em um processo, então executa o comando snapshot contra ele:

```bash
vp dev --host 0.0.0.0
vp exec musea-vrt --base-url http://localhost:5173 --ci --json
```

O fluxo de trabalho: commit linhas de base sob o diretório snapshot, rodar `musea-vrt --ci --json` contra um servidor de desenvolvimento
rodando, depois inspecionar `vrt-report.json`/`vrt-report.html` mais `snapshots/current` e
`snapshots/diff` em caso de falha. Execute novamente com `--update` (ou `approve` para variantes selecionadas) para
mudanças intencionais, e execute `clean` após remover arquivos de arte para que as linhas de base obsoletas não escondam lacunas.
`--ci` sai diferente de zero para diferenças visuais e erros de prévia/captura (rota ausente, falha do
do navegador, tempo de extinção do seletor); Novas linhas de base são reportadas como `new`, então execute `--update` localmente primeiro.

O aplicativo de exemplo também conecta o caminho VRT nativo do Playwright (`examples/vite-musea`, executado via
`vp run test:vrt` / `vp run test:vrt:update`). Snapshots vivem em `e2e/vrt/__snapshots__`, falhas
artefatos em `e2e/vrt/test-results`, e o relatório HTML em `playwright-report`; O GitHub Actions
as envia em caso de falha para que os revisores possam inspecionar as imagens base, atuais e diferenciais.

## Gerar arquivos de arte

Use o gerador para criar um primeiro `.art.vue` a partir de um componente existente:

```bash
vp exec musea-vrt generate src/components/Button.vue
```

O arquivo gerado é um ponto de partida. Revise as variantes, títulos, tags e a cobertura dos adereços antes de
comprometer o projeto.

## Produção de Livros de Fadas

Ative a geração de CSF compatível com Storybook quando quiser arquivos de arte Musea para alimentar uma configuração de Storybook:

```ts
musea({
  storybookCompat: true,
  storybookOutDir: ".storybook/stories",
});
```

## CLI Status

`vize musea` existe na CLI do Rust, mas o fluxo de trabalho recomendado para o Musea hoje ainda é o caminho do plugin Vite
. Trate o subcomando Rust como experimental enquanto o fluxo de trabalho dedicado da galeria se estabiliza.

O subcomando Rust pode estruturar um projeto artístico inicial:

```bash
vize musea new
```

## Pacotes Relacionados

- `@vizejs/vite-plugin-musea`
- `@vizejs/musea-mcp-server`
- `vize_musea`
