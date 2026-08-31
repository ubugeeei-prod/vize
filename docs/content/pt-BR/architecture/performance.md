---
title: Desempenho
---

<!-- Generated translation; source: architecture/performance.md -->

# Desempenho

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. Os números de benchmark vêm de builds de desenvolvimento e podem mudar.

O Vize alcança melhorias significativas de desempenho em relação ao compilador padrão baseado em JavaScript do Vue ao aproveitar as abstrações de custo zero e o multithreading nativo da Rust. Velocidade não é algo agradável — é um pré-requisito para a experiência do desenvolvedor.

## Ambiente de Benchmark

Dois ambientes de medição aparecem nesta página, e cada número abaixo diz de qual deles veio.

**Runner de referência.** As comparações entre ferramentas são medidas pelo workflow Tool Benchmark
e versionadas em `tools/benchmarks/results/tool-benchmark-latest.json`. Esse artefato é a fonte citável, e o
[snapshot de benchmark do Blacksmith](./performance-blacksmith) o publica na íntegra.

|              |                                                      |
| ------------ | ---------------------------------------------------- |
| **Máquina**  | `blacksmith-32vcpu-ubuntu-2404` (32 vCPU, AMD EPYC)  |
| **Snapshot** | commit `1511788d96ea`, 2026-07-30                    |
| **Método**   | mediana de 5 execuções medidas após 1 de aquecimento |
| **Versões**  | vize 0.303.0 · vue 3.6.0-beta.10 · Node v24.14.0     |

**Estação de trabalho local.** As tabelas do linter, do formatador e do verificador de tipos mais
adiante ainda são mantidas à mão a partir de benches locais (`tools/benchmarks/scripts/lint.ts`, `tools/benchmarks/scripts/fmt.ts`,
`tools/benchmarks/scripts/check.ts`) e foram medidas aqui. Elas ainda não são reproduzíveis no runner de referência,
portanto leia-as como indicativas.

|             |                                             |
| ----------- | ------------------------------------------- |
| **Máquina** | MacBook Pro (M2 Max, 12 núcleos, 96 GB RAM) |
| **SO**      | macOS 15.3.2 (Darwin 24.3.0)                |
| **Node.js** | v24.14.0                                    |
| **Vite**    | v8.0.0 (Rolldown)                           |
| **Vue**     | v3.6.0-beta.10                              |

## Benchmark: 15.000 arquivos SFC

Compilando **15.000 arquivos Vue SFC gerados** (58,7 MB no total) no runner de referência:

|                                | @vue/compiler-sfc | Vize    | Aceleração |
| ------------------------------ | ----------------- | ------- | ---------- |
| **Thread única**               | 17,15s            | 3,95s   | **4,3x**   |
| **Todos os núcleos (32 vCPU)** | 6,08s             | 329,2ms | **18,5x**  |
| **compiler-sfc 1T vs max**     | 17,15s            | 329,2ms | **52,1x**  |

Fonte: a superfície `compile` do snapshot versionado `tools/benchmarks/results/tool-benchmark-latest.json`
([run 30557718030](https://github.com/ubugeeei-prod/vize/actions/runs/30557718030)) — o mesmo
artefato que `README.md` e o [snapshot de benchmark do Blacksmith](./performance-blacksmith)
publicam.

A melhoria em thread única vem das abstrações de custo zero do Rust (sem GC, sem aquecimento JIT, layout de memória amigável ao cache). A melhoria multithread vem do pool de threads com roubo de trabalho do Rayon, que escala com a contagem de núcleos da CPU.

> **Nota:** este snapshot foi tirado na vize 0.303.0, antes do trabalho de arena e de expressões descrito em "Escolhas de Arquitetura para Desempenho". Ele é datado e reproduzível, mas não é uma medição da árvore atual. A regravação das superfícies entre ferramentas no runner de referência está pendente.

## Por que Rust?

### Abstrações de custo zero

O modelo de propriedade do Rust elimina pausas de coleta de lixo. Os nós da AST do template vivem em uma arena por compilação (`vize_carton`) e tomam emprestado seu texto do código-fonte do template, de modo que um nó é dado puro, sem alocações de heap próprias (`crates/vize_relief/src/relief/elements.rs`). Isso significa:

- **Sem pausas de GC** — Em compiladores baseados em V8, a coleta de lixo pode causar picos imprevisíveis de latência. O Vize não tem overhead de GC.
- **Sem aquecimento JIT** — O compilador JIT do V8 precisa de tempo para otimizar os caminhos quentes. O Vize roda em velocidade máxima desde a primeira instrução.
- **Desempenho previsível** — A compilação antecipada do Rust significa que o desempenho é consistente entre execuções, sem depender das heurísticas de otimização do V8.

### Multithreading nativo

O Vize usa [Rayon](https://docs.rs/rayon) para compilação paralela de dados. Cada arquivo SFC é compilado de forma independente, tornando a carga de trabalho trivialmente paralela, e o driver de lote em `crates/vize/src/commands/build/runner.rs` distribui as entradas planejadas pelo pool:

```rust
// crates/vize/src/commands/build/runner.rs — o driver de lote
planned_inputs
    .par_iter()
    .map(|input| compile_file_with_profile(&input.source, compile_settings, &stats))
    .collect()
```

A arena não é criada aqui. Ela é adquirida onde nasce — nos pontos de entrada de template, script e estilo dentro de `vize_atelier_sfc` — a partir de um pool por worker:

```rust
// por exemplo, crates/vize_atelier_sfc/src/compile.rs
let allocator = vize_carton::pool::acquire();
```

A abordagem de roubo de trabalho significa que, se um arquivo for significativamente maior que os outros, threads ociosas vão roubar trabalho da fila da thread ocupada, mantendo um balanceamento de carga quase perfeito.

### Layout de memória eficiente

O layout de structs e os discriminantes de enum do Rust são compactos. A representação da AST em `vize_relief` é amigável ao cache, reduzindo gargalos de largura de banda de memória:

- **Discriminantes de um byte** — `NodeType` é um `#[repr(u8)]` com 27 variantes (`crates/vize_relief/src/relief/core.rs`), então o tipo de um nó custa um byte, não uma string alocada no heap.
- **Tamanhos de nó fixados** — cada nó de template carrega uma asserção `const` de tamanho, de modo que um campo que faz o nó crescer quebra a build em vez do orçamento. `ElementNode` tem 104 bytes, `SimpleExpressionNode` 88, `AttributeNode` 56, `TextNode` 24 e `SourceLocation` 8 (`crates/vize_relief/src/relief/{elements,expressions,control_flow,nodes}.rs`).
- **Sem cabeçalhos de objeto** — Ao contrário dos objetos JavaScript (que carregam cadeias de protótipos, mapas de propriedades e ponteiros de classe oculta), structs Rust são dados puros com overhead zero.

### Sem overhead de tempo de execução

Diferente dos compiladores baseados em JavaScript que rodam na V8, o Vize compila diretamente para código nativo. Não há aquecimento JIT, nem coletor de lixo, nem contenção de loop de eventos. A CLI é distribuída como um executável nativo autocontido por plataforma — totalmente estático nos alvos Linux musl, o que a CI verifica (`tools/commands/ci/github/verify-musl-cli-binary.rs`), e ligado dinamicamente à biblioteca C do sistema nos alvos glibc, macOS e Windows. O plugin do Vite carrega o mesmo compilador como um addon nativo do Node (`@vizejs/native`), e não como um processo separado.

## Escolhas de Arquitetura para Desempenho

### Alocação em Arena

`vize_carton::Allocator` é um alocador de bump para nós da AST que encapsula o [`oxc_allocator`](https://docs.rs/oxc_allocator), de modo que os nós de template e as expressões JavaScript retidas compartilham uma arena e um tempo de vida (`crates/vize_carton/src/allocator.rs`). Isso significa:

- **A alocação é O(1)** — Basta avançar um ponteiro. Sem percorrer listas livres, sem gerenciamento de fragmentação.
- **A recuperação é O(1) e reutilizada** — Ao fim de uma compilação a arena sofre `reset()`, não é descartada: o ponteiro de bump volta ao início do bloco e a arena retorna a uma lista livre por worker (`crates/vize_carton/src/pool.rs`, limitada a 4 arenas ociosas por worker). O arquivo seguinte reutiliza a mesma memória em vez de pedir mais ao sistema operacional.
- **A localidade de memória é excelente** — Os nós são empacotados de forma contígua na memória, maximizando os acertos de cache L1/L2 durante a travessia da árvore.

Valores apoiados na arena não podem sobreviver à sua compilação. Esse contrato é imposto pelo compilador (`reset` recebe `&mut self`, e o guard do pool é dono da sua arena) e, em builds de depuração, por um carimbo de geração que causa panic se um valor for lido depois que sua arena foi reciclada (`crates/vize_carton/src/allocator/generation.rs`).

Nada na AST implementa `Drop` — os tipos de contêiner da arena rejeitam payloads que precisem ser destruídos, então isso é um erro de compilação, e não uma convenção.

### Tokenizador de passagem única

O tokenizador do `vize_armature` é uma máquina de estados orientada a bytes sobre `&[u8]` (`crates/vize_armature/src/tokenizer.rs`). Ele nunca materializa um token: não existe tipo `Token` nem vetor de tokens em lugar algum do compilador. Em vez disso, `tokenize()` executa uma passagem até o fim da entrada e envia eventos a um receptor `Callbacks`, que o parser implementa — assim cada evento é tratado de forma síncrona conforme é produzido, e o array intermediário que um projeto de duas fases exigiria simplesmente nunca existe.

Note que isso é baseado em push, não em uma leitura preguiçosa: o parser não pede tokens e não pode interromper o laço no meio.

### Internação de strings

Nomes que se repetem dentro de uma compilação — nomes de diretiva normalizados, nomes de assets, nomes de argumento em camelCase — são internados em átomos apoiados na arena por `vize_carton::interner`, com um conjunto [`phf`](https://docs.rs/phf) de tempo de compilação com 181 nomes bem conhecidos (tags HTML/SVG/MathML, componentes embutidos do Vue, nomes de diretiva e os atributos que as transformações tratam de forma especial) que resolvem para literais `'static` sem tocar a arena. Isso significa:

- Nomes computados repetidos compartilham uma única alocação na arena
- As buscas por nomes bem conhecidos são um hash perfeito de tempo de compilação, sem alocação

A internação é o caminho de fallback, não o caso comum. A maioria dos nomes nunca é copiada: um nome de tag, um nome de atributo e a maior parte do conteúdo de expressões são fatias `&'a str` emprestadas diretamente do código-fonte do template, então o caminho comum não aloca nada (`crates/vize_carton/src/interner.rs` documenta a política campo a campo).

Átomos são `&'a str` comuns, então comparações de nome são comparações de conteúdo, não de identidade de ponteiro. A internação compra economia de alocação e localidade de cache — ela não é um atalho rápido para `==`.

### Compilação Incremental

O plugin do Vite (`@vizejs/vite-plugin`) faz cache em nível de arquivo, em duas camadas com chaves diferentes:

- **Em memória, para dev e HMR** — indexado pelo caminho resolvido do arquivo (`npm/builder/vite/src/plugin/compiled-module-cache.ts`). As entradas são removidas explicitamente em hot update, em vez de reindexadas, de modo que um arquivo alterado é recompilado e seus vizinhos não.
- **Detecção de mudança na pré-compilação** — indexada por `mtime` + tamanho, comparados em Rust (`crates/vize_atelier_sfc/src/vite_plugin/precompile.rs`). É esse portão que decide quais arquivos um lote recompila.
- **Em disco, entre processos** — `node_modules/.vize/vite-precompile`, indexado por um hash SHA-256 do código-fonte mais uma chave de manifesto que cobre a identidade do binário do compilador e as opções resolvidas (`npm/builder/vite/src/plugin/precompile-cache-key.ts`). O hash de conteúdo é usado aqui justamente porque `mtime` não é confiável entre máquinas e checkouts.

## Medido: trabalho de arena e de expressões

O trabalho interno do compilador descrito acima é medido por um harness de microbenchmarks por crate (`cargo bench --bench davinci`) sobre uma escada fixa de seis fixtures, `tools/benchmarks/crates/davinci_harness/fixtures/{small,medium,large,stress-deep,stress-wide,stress-interp}.vue`.

**Como ler estes números.** As contagens de alocação são determinísticas e independentes de máquina, portanto são fatos exatos e servem como catraca de regressão. Os tempos de parede foram tomados em uma máquina de desenvolvimento compartilhada com amostragem `--quick` e são **apenas indicativos** — as gravações no runner de referência (Blacksmith) ainda estão pendentes, e é por isso que cada entrada `wall_p50_ns` e `allocs` em `davinci-road/plan/budgets.toml` continua em `0`, significando "ainda não gravado, apenas informativo". Os arquivos de resultado de cada execução caem em `tools/benchmarks/results/davinci/` e são artefatos locais, não baselines versionadas.

Chamadas de alocação por compilação, antes e depois do trabalho de strings e arena (exatas, mesmas fixtures):

| Fixture         | Parse     | Compilação DOM | Compilação SSR | Compilação Vapor |
| --------------- | --------- | -------------- | -------------- | ---------------- |
| `small`         | 21 → 9    | 52 → 39        | 73 → 60        | 90 → 73          |
| `medium`        | 171 → 107 | 329 → 264      | 1.099 → 1.030  | 588 → 515        |
| `large`         | 350 → 272 | 656 → 573      | 1.106 → 983    | 1.136 → 1.003    |
| `stress-deep`   | 397 → 155 | 669 → 426      | 612 → 369      | 764 → 514        |
| `stress-wide`   | 213 → 204 | 255 → 245      | 416 → 405      | 280 → 261        |
| `stress-interp` | 616 → 105 | 1.048 → 536    | 3.149 → 2.637  | 1.495 → 974      |

Os tamanhos dos nós encolheram junto, e os novos tamanhos estão fixados por asserções `const`: `RootNode` 296 → 224 bytes, `DirectiveNode` 208 → 176, `ElementNode` 128 → 104, `SimpleExpressionNode` 120 → 88, `AttributeNode` 80 → 56, `TextNode` 32 → 24.

**Pico de memória residente.** A reutilização da arena entre arquivos é o maior ganho isolado, e é um resultado de memória, não de velocidade. Compilando todos os 36.541 SFCs do corpus versionado (`vize build "tests/_fixtures/_git/**/*.vue" --format stats`, binários `ci-opt`, tamanho máximo do conjunto residente via `/usr/bin/time -l`, mesma máquina antes e depois):

| Workers | Antes    | Depois   | Variação   | Execuções cada |
| ------- | -------- | -------- | ---------- | -------------- |
| 12      | 766,5 MB | 171,1 MB | **−77,7%** | 5              |
| 1       | 717,0 MB | 88,2 MB  | **−87,7%** | 3              |

O número com um único worker é o sinal de acumulação: ele independe de escalonamento, mostrando que o pico antigo vinha de vazamento por arquivo, e não das arenas por worker. O tempo de parede ficou inalterado dentro do ruído, e todos os 36.541 arquivos emitidos foram idênticos byte a byte (manifestos SHA-256 comparados).

**Reanálise de expressões.** As expressões de template agora são analisadas uma única vez, durante a análise do template, e retidas no nó. Os consumidores leem a AST retida em vez de reanalisar o texto. Na trilha SSR, a fixture `stress-interp` passou de 500 reanálises redundantes de expressão por compilação para zero, e essa trilha fundida está **−13,6%** líquidos em tempo de parede em relação à árvore anterior à retenção (346,8µs → 299,8µs) — a análise agora custa mais e os consumidores custam muito menos. As trilhas DOM e Vapor não tinham reanálises a excluir nessa fixture, então ainda carregam o custo de análise adicionado; fechar essa lacuna é rastreado como trabalho restante da fase, não como um ganho já entregue.

## Benchmark: Linter — patina vs eslint-plugin-vue

Analisando **15.000 arquivos Vue SFC**, estação de trabalho local:

|           | eslint-plugin-vue (ST) | Pátina Vize (ST) | Aceleração | eslint-plugin-vue (MT) | Pátina Vize (MT) | Aceleração | **eslint ST vs Vize MT** |
| --------- | ---------------------- | ---------------- | ---------- | ---------------------- | ---------------- | ---------- | ------------------------ |
| **Tempo** | 45,08s                 | 4,02s            | **11,2x**  | 16,38s                 | 784ms            | **20,9x**  | **57,5x**                |

Correr `vp run --workspace-root bench:lint` para se reproduzir.

### Perfil de fiapos consciente do tipo

O linting consciente de tipos é intencionalmente perfilado nas fases em que o custo tende a se agrupar: análise sintática SFC, análise
Croquis, geração virtual de TypeScript, coleta de consultas de templates e sondas Corsa. Quando
múltiplas regras de type-aware apoiadas por templates são ativadas, o Patina coleta a expressão do template e
consultas Promise template em uma única caminhada AST antes da fase de sonda Corsa. A coleção de consultas também compartilha
análise de expressões OXC para verificações unsafe-template e floating-Promise, de modo que uma expressão template
não paga custo de análise duplicado quando ambas as regras estão ativadas.

Faça `vize lint --profile --preset opinionated src` para ver essas linhas em um projeto local. O relatório de perfil
também inclui uma seção rigorosa de auditoria que verifica a cobertura de tempo de parede, o tempo acumulado
trabalhador, os acertos de limiar lento e os períodos internos capturados antes de listar arquivos quentes e operações internas de
. Linhas de arquivo quente mostram participação e throughput por estágio, e linhas de operação sinalizam spans dominantes de
ou picos máximos/médios.

## Benchmark: Formatter — glifo vs Pretier

Formatação de **15.000 arquivos Vue SFC**, estação de trabalho local:

|           | Mais bonita (CLI) | Glifo Vize (ST) | Aceleração | Glifo Vize (MT) | **Cli mais bonito vs Vize MT** |
| --------- | ----------------- | --------------- | ---------- | --------------- | ------------------------------ |
| **Tempo** | 101,20s           | 2,97s           | **34,1x**  | 835ms           | **121,2x**                     |

Correr `vp run --workspace-root bench:fmt` para se reproduzir.

## Benchmark: Type Checker — cânone vs vue-tsc

Verificação de tipos de **500 arquivos Vue SFC gerados** com o caminho de diagnóstico atual respaldado pela Corsa, estação de trabalho local:

|           | vue-tsc (ST)   | Cânone Vize (ST) | Aceleração         | vue-tsc (MT)   | Cânone Vize (MT) | Aceleração         | **vue-tsc ST vs Vize MT** |
| --------- | -------------- | ---------------- | ------------------ | -------------- | ---------------- | ------------------ | ------------------------- |
| **Tempo** | 4,38s          | 511ms            | n/a (cross-engine) | 4,41s          | 493ms            | n/a (cross-engine) | n/a (cross-engine)        |
| **Taxa**  | 114 arquivos/s | 979 arquivos/s   |                    | 113 arquivos/s | 1.0k arquivos/s  |                    |                           |

As linhas de verificação de tipo abrangem dois mecanismos TypeScript: o vue-tsc executa o compilador JavaScript enquanto o Vize check executa o tsgo nativo (Corsa). Por isso nenhuma razão única é publicada (`n/a (cross-engine)`); cada classe de mecanismo é classificada separadamente, já que um número único atribuiria a reescrita em Go do TypeScript à camada Vue. Os dois tempos são reais e foram medidos na mesma execução; veja o [snapshot de benchmark Blacksmith](./performance-blacksmith) para o ranking por classe de mecanismo.

> **Nota:** O canhão Vize ainda está em desenvolvimento inicial e o caminho de diagnóstico apoiado pela Corsa ainda está alcançando a fidelidade vue-tsc. Essas medições refletem a implementação nativa atual com CLI primeiro, com um recurso de reserva por sessão de projeto, e mudarão à medida que a cobertura e a paridade de diagnóstico melhoram.

Execute `node tools/benchmarks/scripts/check.ts 500` após `cargo build --release -p vize` para reproduzir esse benchmark rápido.

### Perfil do verificador de tipos

O dispositivo de perfil 500-SFC mantém a maior parte do tempo de parede dentro do comando CLI Corsa, enquanto o caminho rápido de importação remove o custo anterior de análise OXC para arquivos sem especificadores Vue:

| Métrica                         | Antes   | Atualidade |
| ------------------------------- | ------- | ---------- |
| `canon.import.rewrite.vue`      | 26,77ms | 2,45ms     |
| Maior TS Virtual gerado         | 15.401B | 14.414B    |
| Tempo total na parede do perfil | 1,88s   | 668ms      |
| Fase de diagnóstico da Corsa    | 1,67s   | 482ms      |
| Análise do CLI Corsa            | N/A     | 10,41ms    |

A fase de `virtual project` do lado Rust — análise SFC por arquivo, análise Croquis
geração de Virtual TS e reescrita de importação — é distribuída pelo thread
pool do rayon dentro de `VirtualProject::register_paths`. Cada arquivo `.vue` é independente
uma vez que as opções do workspace são resolvidas, então um único lote paraleliza
de forma limpa. Em um dispositivo de 1.000 SFC, a fase cai de ~71 ms para ~25 ms antes mesmo de
Corsa ser ativada.

### Luminária e2e com muitos diagnósticos

`tools/benchmarks/scripts/check.ts` também mede o aplicativo `tests/_fixtures/_git/npmx.dev` quando o aparelho está presente. Isso captura o caminho de mapeamento de diagnóstico em um dispositivo real de aplicação:

| Jogos               | Arquivos SFC fonte | Arquivos virtuais | Diagnósticos | Cânone Vize |
| ------------------- | ------------------ | ----------------- | ------------ | ----------- |
| npmx.dev aplicativo | 134                | 226               | 1,053        | 1,94s       |

O perfil atual desse aparelho mantém a análise diagnóstica do CLI em ~7ms. A maior parte do tempo agora está no comando CLI da Corsa. A autoimportação de stubs do framework em um único arquivo ambiente também reduziu o maior arquivo Virtual TS gerado de cerca de 275KB para 144KB.

## Benchmark: Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue

Build do Vite com **1.000 importações do SFC Vue** (todas importadas em uma única entrada):

|                         | @vitejs/plugin-vue | @vizejs/vite-plugin | Aceleração |
| ----------------------- | ------------------ | ------------------- | ---------- |
| **Tempo de Construção** | 1.71s              | 631.7ms             | **2.7x**   |

> Nota: `@vizejs/vite-plugin` substitui apenas a etapa de compilação do Vue SFC — a diferença de desempenho vem inteiramente dessa parte. A resolução de dependências, construção de grafos de módulos, agrupamento (Rolldown) e todos os outros internos do Vite são idênticos aos `@vitejs/plugin-vue`. Para performance pura em compilações, veja o [Compiler benchmark](#benchmark-15000-sfc-files) acima. `@vizejs/vite-plugin` pré-compila `.vue` arquivos com entusiasmo usando compilação multithreaded nativa, que também permite um HMR mais rápido.

Esta linha é a superfície `vite` do snapshot commitado `tools/benchmarks/results/tool-benchmark-latest.json` ([run 30557718030](https://github.com/ubugeeei-prod/vize/actions/runs/30557718030)) — o mesmo artefato que `README.md` e o [snapshot de benchmark Blacksmith](/architecture/performance-blacksmith) publicam. `tests/tooling/docs-vite-benchmark-row.test.ts` a fixa nesse artefato, em todos os idiomas.

O número publicado aqui até agora — `957ms` / `479ms` / `2.0x` — veio de `tools/benchmarks/scripts/vite.ts` antes de #3392, que media o Vize com um cache de pré-compilação persistente deixado quente pelo próprio warmup enquanto o `@vitejs/plugin-vue` compilava do zero. Esse harness agora reporta linhas separadas de cold e warm na máquina em que roda, então produz um diagnóstico local, não um speedup publicável. Use `vp run --workspace-root bench:vite` para comparar uma mudança consigo mesma.
