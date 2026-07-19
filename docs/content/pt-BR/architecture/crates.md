---
title: Caixas
---

<!-- Generated translation; source: architecture/crates.md -->

# Referência à Caixa

> **⚠️ Trabalho em andamento:** Vize está em desenvolvimento ativo. Veja o canônico
> [Rust crate support tiers](../stability.md#rust-crate-support-tiers) antes de depender de um público
> API.

O espaço de trabalho Rust da Vize é organizado em torno de 20 caixas principais. Cada caixa possui uma faixa reutilizável para que
análise sintática, análise semântica, geração de código, linting, formatação, verificação de tipos e ferramentas
editor possam compartilhar o mesmo modelo sintácico.

## Fundação

| Caixa             | Função                                                                                                       |
| ----------------- | ------------------------------------------------------------------------------------------------------------ |
| `vize_carton`     | Alocador compartilhado, strings, coleções de hash, flags, profiler, i18n e utilitários DOM/tag               |
| `vize_relief`     | AST de template compartilhado Vue, erros do compilador e opções do compilador                                |
| `vize_armature`   | Tokenizador e parser de templates Vue                                                                        |
| `vize_croquis`    | Análise semântica, rastreamento de escopo, metadados de vinculação, reatividade e assistentes virtuais de TS |
| `vize_croquis_cf` | Análise semântica entre arquivos e diagnóstico em todo o projeto                                             |

## Compilação

| Caixa                | Função                                                                      |
| -------------------- | --------------------------------------------------------------------------- |
| `vize_atelier_core`  | Rotas de transformação compartilhadas e infraestrutura de geração de código |
| `vize_atelier_dom`   | Compilação de templates orientada a VDOM                                    |
| `vize_atelier_vapor` | Compilação de modelos em modo vapor                                         |
| `vize_atelier_ssr`   | Compilação de templates de renderização no lado do servidor                 |
| `vize_atelier_sfc`   | `.vue` análise sintáctica mais orquestração de script, template e estilo    |
| `vize_atelier_jsx`   | Análise compartilhada, redução e integração de compiladores JSX/TSX         |

## Ferramentas para Desenvolvedores

| Caixa          | Função                                                                                     |
| -------------- | ------------------------------------------------------------------------------------------ |
| `vize_patina`  | Formatação do linter e diagnóstico do Vue SFC                                              |
| `vize_glyph`   | Formatador SFC Vue                                                                         |
| `vize_canon`   | Verificação de tipos consciente do Vue e geração virtual de TypeScript                     |
| `vize_maestro` | Implementação do Protocolo de Servidor de Linguagem                                        |
| `vize_musea`   | Análise sintática de arte Musea, documentação, geração de paleta, autogeração e núcleo VRT |
| `vize_curator` | Cargas úteis do inspetor local, metadados de grafos/diferenciais e relatórios de perfil    |
| `vize_fresco`  | Primitivas de UI terminal usadas por experimentos orientados a TUI                         |

## Camadas de distribuição

| Caixa          | Função                                                       |
| -------------- | ------------------------------------------------------------ |
| `vize_vitrine` | Vinculações NAPI e WASM compartilhadas para consumidores JS  |
| `vize`         | CLI nativo de ferrugem mais reexportações de caixa para docs |

## Notas

- `vize_musea` é o núcleo Rust para ferramentas artísticas de Musea. A interface da galeria e o fluxo de trabalho dev-server são
  fornecido por `@vizejs/vite-plugin-musea`.
- `vize_curator` não é publicado. Possui artefatos de desenvolvedores locais, como cargas úteis de inspetores,
  relatórios de agentes, metadados de grafos cruzados e renderização de relatórios de perfil CLI. O perfilador de
  de baixo nível permanece em `vize_carton` porque caixas compartilhadas instrumentam seus próprios caminhos quentes.
- `vize_vitrine` é a ponte de Rust para JS. Pacotes como `@vizejs/native` e
  `@vizejs/wasm` publicar suas encadernações.
  - `vize` é a caixa completa do Rust CLI no espaço de trabalho. Para a versão alfa da v1, seu canal binário público é
    GitHub Releases ou Nix, enquanto o pacote npm `vize` é o ponto de entrada suportado pelo script-package.

## Mapeamento de pacotes

| Pacote / Comando            | Caixa(s) principal(es) de ferrugem                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `vize build`                | `vize`, `vize_atelier_sfc`, `vize_atelier_dom`, `vize_atelier_vapor`, `vize_atelier_ssr` |
| `vize fmt`                  | `vize`, `vize_glyph`                                                                     |
| `vize lint`                 | `vize`, `vize_patina`                                                                    |
| `vize check`                | `vize`, `vize_canon`                                                                     |
| `vize inspector`            | `vize`, `vize_curator`                                                                   |
| `vize lsp`                  | `vize`, `vize_maestro`                                                                   |
| `@vizejs/vite-plugin`       | `vize_vitrine`, `vize_atelier_sfc`                                                       |
| `@vizejs/native`            | `vize_vitrine`                                                                           |
| `@vizejs/wasm`              | `vize_vitrine`                                                                           |
| `@vizejs/vite-plugin-musea` | `vize_musea`, `vize_vitrine`                                                             |
| `@vizejs/musea-mcp-server`  | `vize_musea`, `vize_vitrine`                                                             |
| `oxlint-plugin-vize`        | `vize_patina`, `vize_vitrine`                                                            |
