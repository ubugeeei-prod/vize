---
title: Práticas de Engenharia de Linguagem
---

<!-- Generated translation; source: architecture/language-engineering-practices.md -->

# Práticas de Engenharia de Linguagem

Vize é uma toolchain do Vue, mas tem os mesmos modos de falha de um compilador: pequenas mudanças na sintaxe podem
mover diagnósticos, geração de código, comportamento do editor, saída de pacotes e desempenho ao mesmo
tempo. Esta página registra as práticas de processamento de linguagem que o Vize adota dos repositórios maduros de compiladores e verificadores de tipo
, depois as mapeia para os próprios fixtures, snapshots, testes de paridade, benchmarks,
e portas de lançamento do Vize.

## Sinais de Origem

| Fonte                                                                                                                                 | Prática observada                                                                                                                                                                                                                                  | Tradução Vize                                                                                                                                                                                                                                 |
| ------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`rust-lang/rust`](https://github.com/rust-lang/rust) e o [`rustc-dev-guide`](https://rustc-dev-guide.rust-lang.org/tests/intro.html) | `compiletest` agrupa os testes de UI por suíte, armazena a saída esperada próxima aos casos de origem, usa `tidy` para invariantes de repositório e acompanha as regressões do ecossistema e desempenho separadamente.                             | Trate as mudanças voltadas para compiladores como mudanças de fixture primeiro. Mantenha as expectativas do analisador/compilador em `tests/fixtures` e `tests/expected`, e mantenha invariantes de repositório em `tests/tooling/*.test.ts`. |
| [`rustc` ecosystem and perf testing](https://rustc-dev-guide.rust-lang.org/tests/ecosystem.html)                                      | Crater, cargotest, constructores de grandes projetos e rustc-perf tornam explícita ampla compatibilidade e risco de desempenho antes ou depois da fusão das mudanças no compilador.                                                                | Escale semântica ampla do Vue, formas de código gerado ou mudanças de caminho quente para fixaturas do mundo real, a matriz de paridade do Vue e o orçamento de benchmark PR, em vez de depender apenas dos fixtures unitários.               |
| [`rust-fuzz/cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) e libFuzzer                                                         | Alvos fuzz guiados por cobertura executam entradas arbitrárias em bytes, persistem corpus e minimizam reprodutores de falha antes de transformá-los em regressões determinísticas.                                                                 | Fuzz parser, lexer, CSS, expressão e limites de compilação de templates a partir de `tests/fuzz` com `cargo +nightly fuzz run <target>` antes de tratar as correções de falha como completas.                                                 |
| [Linux kernel testing](https://www.kernel.org/doc/html/next/dev-tools/testing-overview.html)                                          | O KUnit cobre pequenas unidades white-box, o kselftest cobre interfaces de sistema visíveis pelo usuário, o KCOV alimenta fuzzing guiado por cobertura e `perf stat` captura o contador repetível e o status de temporização.                      | Separe pequenas verificações em nível de caixa das verificações de integração CLI/workspace, use coverage/fuzzing para entradas arbitrárias e anexe status de perfil ou benchmark quando os caminhos de fase se movem.                        |
| [Chromium testing and CQ](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/testing_in_chromium.md)                  | Camadas de cromo para unidades herméticas, navegador, web, telemetria e testes de fuzzer; CQ/trybots tornam explicitamente rotas caras ou instáveis, e o ClusterFuzz executa alvos fuzz descobertos em escala.                                     | Mantenha os controles Vize herméticos por padrão, escale o comportamento do navegador/app para fixes do mundo real, use o orçamento de benchmark de PR para status semelhante ao da Telemetria e mantenha reprodutores fuzz para triagem.     |
| [V8 testing](https://v8.dev/docs/test) e [feature launch](https://v8.dev/docs/feature-launch-process)                                 | O V8 roda suítes de motores como `mjsunit` e Test262, regenera arquivos esperados somente após revisão, utiliza fluxos de comparação de `tools/run_perf.py` e benchmark, e exige fuzzing antes de enviar recursos de linguagem.                    | Trate as mudanças de compatibilidade Vue/TS como características de linguagem: cite o comportamento de origem, adicione testes de cenários, compare desempenho quando relevante e execute ou programe fuzzing antes da promoção.              |
| [`microsoft/TypeScript`](https://github.com/microsoft/TypeScript)                                                                     | O grafo de tarefas Hereby separa as tarefas de build, formatação, lint, teste e baseline. A saída do compilador é revisada por meio de `tests/baselines/reference` versus a saída gerada localmente antes de `baseline-accept`.                    | Mantenha fotos como contratos revisados. Um `tests/snapshots/*` alterado ou snapshot de Rust `insta` deve ser explicado pelo PR e limitado ao comportamento alterado.                                                                         |
| [`TypeScript tests/cases/fourslash`](https://github.com/microsoft/TypeScript/tree/main/tests/cases/fourslash)                         | O comportamento de serviços de linguagem voltado para editores é capturado como milhares de arquivos de cenário, em vez de ser inferido apenas por testes de compilador.                                                                           | LSP, quick-fix, completion, hover e mudanças incrementais no editor devem ter cobertura de fumaça ou integração em nível de cenário, não apenas fixaturas de parser/compilador.                                                               |
| [`microsoft/typescript-go`](https://github.com/microsoft/typescript-go)                                                               | A porta nativa mantém o submódulo TypeScript como implementação de referência, adiciona testes mínimos do compilador, escreve a saída gerada em `testdata/baselines/local`e trata linhas de base `.diff` reduzidas como evidência de convergência. | Compare a saída do Vize com o comportamento oficial do Vue e TypeScript antes de introduzir uma regra específica do Vize. Se o Vize divergir intencionalmente, documente o motivo e o nível de compatibilidade.                               |
| [`facebook/flow`](https://github.com/facebook/flow)                                                                                   | O Flow mantém testes de integração em formato de diretório com `.exp` saída esperada, suporta a regravação de alterações intencionais de saída e utiliza `newtests` de estilo ação/asserção para fluxos de editor e servidor.                      | Prefiro fixos pequenos para diagnósticos e fluxos de trabalho do editor. Snapshots regravados só são aceitáveis após revisar o diferencial e manter o ruído gerado fora da linha de base.                                                     |

## Vize mudam de classe

Cada PR de processamento de linguagem deve nomear sua classe de mudança e incluir evidências da linha
correspondente. Use o comando mais restrito durante o desenvolvimento e depois amplie quando a mudança tocar o comportamento
compartilhado.

| Mudança de classe                                              | Evidências necessárias                                                                                                                                                      | Comandos comuns                                                                                                                                            |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Parser ou AST                                                  | Fixture mínimo do parser, AST esperado ou saída de erro, e nenhuma atualização geral de snapshot.                                                                           | `cargo test -p vize_armature`, `cargo test -p vize_test_runner`, `node tests/tooling/support/generate-expected.ts <fixture>`                               |
| Compilador e geração de código                                 | Fixture de fonte mínima, saída esperada de DOM/Vapor/SSR e paridade real quando a forma de execução emitida muda.                                                           | `cargo test -p vize_atelier_dom`, `cargo test -p vize_atelier_vapor`, `vp run --filter './tests' test:build`                                               |
| Análise semântica, fiapos e análise cruzada                    | Fixture de regras ou analisadores, snapshot de saída JSON ou agente, e documentação para diagnósticos alterados.                                                            | `cargo test -p vize_patina`, `vp run --filter './tests' test:lint`, `node --test tests/tooling/snapshot-baselines.test.ts`                                 |
| Virtual TypeScript e verificação de tipos                      | Fixture SFC mínimo, snapshot diagnóstico mapeado, revisão virtual de TS gerada e nota oficial de paridade Vue ou TypeScript.                                                | `vp run --filter './tests' test:check:fixtures`, `cargo test -p vize_canon`, `vize check --show-virtual-ts <file>`                                         |
| Forformatador e LSP                                            | Saída de formatação dourada ou cobertura de fumaça do protocolo, além de uma verificação focada de integração com editores quando o comportamento é visível para o usuário. | `cargo test -p vize_glyph`, `cargo test -p vize_maestro`, `node --test tests/tooling/lsp-smoke.test.ts`                                                    |
| Empacotamento, lançamento ou documentação em tempo de execução | Teste de governança, instalação de fumaça ou cobertura de fluxo de trabalho, e documentos de lançamento/prontidão quando a postura de produção muda.                        | `node --test tests/tooling/*.test.ts`, `rust-script tools/commands/release/npm/smoke-release-install.rs --prepare-manifests --runtime-checks`, `vp run --workspace-root check:ci` |

## Faixas de Garantia

Algumas mudanças precisam de uma segunda lente além da classe de mudança. Essas faixas tornam explícito o status de segurança, o status de desempenho
e as evidências de confusão no PR, em vez de deixá-las como revisores
memória.

| Lane       | Use quando a mudança toca                                                                                                                                        | Evidências a registrar                                                                                                                                                                                                                                                                           |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Segurança  | Gerenciamento de URLs, saída HTML ou SSR, carregamento de sistema de arquivos/configuração, carregamento nativo, publicação de pacotes, CI ou credenciais.       | `security-audit` em `.github/workflows/check.yml`, `vp exec pnpm audit --prod --audit-level moderate`, `cargo audit --deny warnings`, verificações de runtime de instalação de fumaça, checagens fixadas de Ações do GitHub e qualquer regressão focada que cubra a entrada ou limite arriscado. |
| Desempenho | Analisador, compilador, linter, formatador, verificador de tipos, cache, percorrimento de grafos de projeto, saída gerada ou I/O CLI.                            | `.github/workflows/benchmark.yml`, `bench/compare-pr.mjs`, `bench/enforce-pr-budget.mjs`, o status `pr-benchmark-budget`, tarefas locais de `bench:*` e `vize lint --profile`, `vize check --profile``vize fmt --profile` ou quando a regressão precisa de atribuição.                           |
| Fuzzing    | Análise sintáctica orientada a bytes, recuperação de sintaxe, análise sintática CSS, análise de expressões JS/TS, lexing de templates ou recuperação de codegen. | `.github/workflows/fuzz.yml`, `tests/fuzz/Cargo.toml`, `tools/commands/ci/fuzz/seed_corpus.rs`, `cargo +nightly fuzz run <target>`, artefatos `fuzz-reproducers-*` carregados e uma regressão determinística minimizada após o crash, timeout ou OOM foi compreendida.                                      |

## Política Básica

- Comece pelo menor caso de falha ou ilustrativo, depois aceite acessórios mais amplos apenas quando eles
  provam ser um comportamento transversal.
- Arquivos snapshot e baseline são contratos visíveis ao usuário. Se um diferencial muda o diagnóstico, gerado
  código, saída pública de CLI ou comportamento do editor, o PR deve explicar por que a nova saída está correta.
- Normalize dados voláteis antes que atinjam um ponto de base. Caminhos, tempos, hashes e ambiente
  detalhes não devem criar refluxo recorrente de snapshots.
- Mantenha os artefatos de paridade explícitos. `tests/snapshots/check`, `tests/snapshots/lint`, mundo real
  snapshots de fixture e a matriz de paridade do Vue são o registro de compatibilidade.
- Não atualize linhas de base grandes de snapshots a menos que o PR seja sobre essas saídas. Quando muitos arquivos se movem
  juntos, incluam uma breve explicação da causa compartilhada.

## Gatilhos de Escalada

Busque evidências mais amplas quando uma mudança tem uma destas formas:

- O comportamento sintaxe, transformado ou virtual do TypeScript pode afetar aplicações comuns do Vue:
  adicionar ou atualizar um dispositivo real e explicar a paridade em relação às ferramentas oficiais do Vue.
- Forma de código gerada, cache, percorrimento de grafos de projeto ou análise consciente de tipos podem ser movidos
  de rendimento: execute o benchmark local que corresponda à superfície e confie no orçamento do benchmark de PR.
- Gerenciamento de URLs, saída HTML/SSR, carregamento de configuração, publicação de pacotes, carregamento nativo, CI, ou
  mudanças no código adjacentes a credenciais: registre o status de auditoria de segurança e adicione a regressão focada que
  prove que a fronteira ainda está protegida.
- Recuperação de analisador, entrada arbitrária de bytes, análise CSS/template/expressão, ou correções de falha: run ou
  agendar o alvo de fuzz correspondente, manter o reprodutor e realizar uma regressão determinística
  minimizada antes de fechar a solicitação de correção.
- LSP, editor, correção rápida, conclusão, hover ou mudanças incrementais de comportamento: adicionar nível de cenário
  cobertura que exercita a sequência visível pelo usuário, não apenas o diagnóstico final.
- Um snapshot muda devido a caminhos, hashes, ordenação, tempo, ambiente ou plataforma host:
  normalizar primeiro, depois aceitar a linha de base apenas se o diferencial restante for significativo.

## Guarda-rails operacionais

Vize mantém essas práticas executáveis em vez de depender da memória:

- `CONTRIBUTING.md` nomeia a disciplina de classe de mudança para os colaboradores.
- `.github/PULL_REQUEST_TEMPLATE.md` pede referências comportamentais, riscos e evidências de verificação.
- `bench/test-inventory.mjs` reporta o inventário atual de ativos de teste no PR CI.
- `.github/workflows/benchmark.yml` compara o desempenho da CLI base e da chefe e aplica um orçamento de relações públicas.
- `.github/workflows/check.yml` executa o trabalho `security-audit` para npm de produção e Rust
  avisos de dependência.
- `.github/workflows/fuzz.yml` roda o espaço de trabalho `tests/fuzz` cargo-fuzz e os uploads travam
  reprodutores para triagem de analisadores/compiladores.
- `docs/release/production-readiness.md` e `docs/release/vue-parity-matrix.md` definem quando um
  comportamento pode ser chamado de pronto para produção ou compatível.
- `tests/tooling/language-engineering-practices.test.ts` mantém esta página, o guia de contribuições,
  e o modelo de PR conectados.
