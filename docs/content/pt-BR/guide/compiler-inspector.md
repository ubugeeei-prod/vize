---
title: Inspetor de Compiladores
---

<!-- Generated translation; source: guide/compiler-inspector.md -->

# Inspetor de Compiladores

O inspetor playground coleta o compilador e as superfícies de análise necessárias para revisar uma reprodução `.vue`
. Ele mostra a saída oficial do compilador SFC do Vue, a saída do compilador Vize, Virtual TS, VIR e um grafo
cross-file para lotes locais.

Abra o inspetor do parquinho:

```bash
https://vizejs.dev/play/?tab=inspector
```

O inspetor executa essas verificações no navegador:

- `@vue/compiler-sfc` para a saída de referência
- Vize WASM para a saída Vize
- Virtual TS suportado pela Canon para o arquivo selecionado
- Croquis VIR para o arquivo selecionado
- Grafo nativo `vize_curator` e metadados de diferença compartilhados com a CLI
- Diagnósticos cruzados para arquivos de payload
- Seleção de alvos DOM ou SSR
- Controles opcionais de renderizador personalizado e modo de sintaxe de template
- Aba de saída completa para ambos os compiladores
- Uma aba de comparação com linhas apenas Vue e apenas Vize
- Um link permalink e pull request pré-preenchido

## Cargas Úteis CLI

Use `vize inspector` quando a reprodução já existir em um projeto local. Um único arquivo produz uma URL
playground por padrão:

```bash
vize inspector src/App.vue
```

Diretórios e globos criam cargas úteis em lote. O playground abre o lote e permite que você troque
entre arquivos.

```bash
vize inspector src/components
vize inspector "src/**/*.vue" --target ssr
```

Para lotes grandes, emita JSON em vez de uma URL longa:

```bash
vize inspector "src/**/*.vue" --format json --output inspector-payload.json
```

Para agentes de IA ou transferência de terminais, emita o relatório do agente. Inclui o payload, URL do playground, métricas resumo
e metadados de grafos entre arquivos.

```bash
vize inspector "src/**/*.vue" --format agent --output inspector-agent.json
```

Em uma verificação local de desenvolvimento, a CLI também pode executar a comparação do compilador diretamente. Isso usa
compilador Rust no binário atual e carrega `@vue/compiler-sfc` do projeto atual ou
o espaço de trabalho Vize `node_modules`.

```bash
vize inspector "src/**/*.vue" --format compare --output inspector-compare.json
```

A carga útil e o relatório do agente são gerados por `vize_curator`, a mesma caixa Rust local usada
pelas ligações WASM do playground para metadados de gráficos e diferenciais de linha. Isso mantém os relatórios de CLI em lote e a inspeção
navegador alinhados, enquanto o compilador oficial do Vue roda dentro do navegador.

Opções úteis:

| Opção               | Descrição                                                    |
| ------------------- | ------------------------------------------------------------ |
| `--target dom`      | Compare a saída do compilador VDOM                           |
| `--target ssr`      | Compare a saída do compilador SSR                            |
| `--format agent`    | Emita JSON legível por agente com metadados de grafo         |
| `--format compare`  | Faça uma comparação de CLI só para desenvolvedores com o Vue |
| `--custom-renderer` | Ative o modo de renderização personalizada no playground     |
| `--template-syntax` | Escolha `standard`, `strict`ou `quirks`                      |
| `--max-files <n>`   | Limite o número de arquivos em uma carga útil batch          |
| `--playground-url`  | Substitua a URL do playground usada para links               |

## Fluxo de Trabalho de Relações Públicas

Ao abrir uma PR de paridade do compilador, inclua o permalink do inspetor no corpo do PR e adicione o fixture mínimo
ou snapshot completo que torne a mudança de saída revisável no CI. O link PR
pré-preenchido é um ponto de partida; depois de empurrar seu branch, substitua a cabeça de comparação se o GitHub pedir.

A evidência útil de PR é:

- O permalink do inspetor
- O alvo selecionado e as opções
- O `.vue` minimizado ou o snapshot completo
- Contexto Virtual TS, VIR ou grafo relevante quando a correção cruza superfícies do compilador
- A razão pela qual a saída do Vize deve corresponder ou diferir intencionalmente do Vue
- O comando de verificação local que cobre a superfície do compilador tocada
