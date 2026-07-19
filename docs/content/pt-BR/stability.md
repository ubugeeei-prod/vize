---
title: Estabilidade
description: Níveis de suporte ao Vize v1 alpha, promessas de compatibilidade e superfícies experimentais.
---

<!-- Generated translation; source: stability.md -->

# Estabilidade

Vize está caminhando para uma versão alfa v1. O contrato alfa é intencionalmente mais restrito do que um contrato
v1 estável: ele nomeia as superfícies que devem ser utilizáveis pelos primeiros adotantes, enquanto mantém espaço para
mudanças internas e integrações experimentais rapidamente. O projeto completo ainda não é uma cadeia de ferramentas completamente
pronta para produção; As decisões de liberação devem usar o
[production-readiness checklist](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/production-readiness.md).
janelas de depreciação, regras do SemVer e suporte a linhas de lançamento estão detalhados no
[support policy](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/support-policy.md).

## Contrato de Versionamento

Antes da estabilidade da v1, qualquer pré-lançamento pode incluir mudanças que não se venham nem parar. O Vize ainda trata mudanças quebradas como material
nota de lançamento, especialmente quando afetam entradas de pacotes, flags de CLI, campos de configuração
códigos de diagnóstico ou saída gerada.

A linha alfa v1 usa estas regras:

| Superfície                                        | Expectativa Alpha                                                                                    |
| ------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Nomes de pacotes publicados                       | Deve permanecer disponível ou ser enviado com notas de migração                                      |
| Comandos e sinalizações CLI documentados          | Deve evitar mudanças de comportamento silenciosas                                                    |
| Campos de configuração documentados               | Deve manter nomes e formas de valor estáveis, a menos que as notas de lançamento indicem uma mudança |
| Códigos diagnósticos listados nos documentos      | Devem permanecer reconhecíveis para que supressões e relatórios de correção continuem úteis          |
| APIs publicadas do Rust crate                     | Siga abaixo o nível por caixa e o contrato de depreciação                                            |
| Componentes internos da caixa Rust não exportados | Pode mudar sem suporte à migração antes da v1 estável                                                |
| Código gerado e saída virtual de TS               | Pode mudar quando necessário para correção, compatibilidade, desempenho ou diagnóstico               |

## Suporte em Tempo de Execução

O Node.js padrão para pacotes públicos de runtime npm é o Node 22, incluindo
`oxlint-plugin-vize`. O plugin Oxlint declara `^22 || >= 24` então o Nó 22 e o Nó 24 ou mais recentes
são permitidos, enquanto o Nó 23 permanece fora da matriz de compatibilidade testada.

O fluxo de trabalho de lançamento cria pacotes nativos para macOS, Linux e Windows em x64 e arm64
onde o pacote declara suporte. Os trabalhos de compatibilidade de CI cobrem o chão declarado do Nó e a versão
atual do projeto do Nó.

A matriz de fumaça totalmente instalada de zero (`.github/workflows/native-smoke.yml`) roda em uma cadência semanal de
e sob demanda, não em toda campanha de PR. Ele exerce o caminho de instalação de pacotes publicado em
runners hospedados no GitHub para linux-x64-gnu, linux-arm64-gnu, darwin-arm64 e win32-x64-msvc; Os
alvos restantes Darwin-x64 e Win32-ARM64-MSVC permanecem em runners hospedados específicos da arquitetura.
A matriz corre contra o Nó 22 e o Nó 24. As tags de release permanecem bloqueadas pelo fluxo de trabalho de lançamento
a fumaça de instalar tarball antes da publicação dos pacotes npm. As verificações de fumaça em tempo de funcionamento `vize --version`,
`vize check`, `@vizejs/native` tanto `require` quanto `import`, e uma
`@vizejs/vite-plugin` `vite build` a partir de bolas de asfalto instaladas.

Dois alvos declarados de musl Linux não estão atualmente sendo exercidos por um runner de instalação nova hospedado.
Eles são cobertos por artefatos de build por plataforma, além do `@vizejs/native-*`
resolvedor de dependência opcional até que uma fumaça Alpine conteinerizada possa preparar o
tarball nativo correspondente:

| Alvo             | Intervalo de corredores hospedados                                            | Cobertura compensatória                                                                           |
| ---------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Linux-x64-musl   | Nenhuma VM Alpine/musl hospedada no GitHub está disponível como runner nativo | O projeto de construção emite a bola de tar musl; Manual `node:alpine` fumaça.                    |
| Linux-ARM64-MUSL | Os runners hospedados em Arm64 são Ubuntu GNU, não hosts nativos Alpine/musl  | O projeto de construção emite a bola de bolas de bolas de musl arm64; manual Alpine arm64 fumaça. |

O fechamento desses intervalos é acompanhado junto com [#493](https://github.com/ubugeeei-prod/vize/issues/493).

A versão mínima suportada de Rust (MSRV) para o workspace é declarada em `Cargo.toml` sob
`[workspace.package].rust-version`. A cadeia de ferramentas de desenvolvimento fixada por `rust-toolchain.toml`
pode ser a mesma versão ou mais recente. Antes da v1 se estabilizar, o MSRV pode avançar em qualquer pré-lançamento;
a mudança é mencionada nas notas de lançamento quando muda. Empacotadores posteriores devem ler
`rust-version` do `Cargo.toml` de uma caixa, em vez de inferir a partir do arquivo da toolchain.

## Níveis de Suporte a Pacotes

| Tier                      | Pacotes                                                                                       | Contrato                                                                                                                 |
| ------------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Suporte Alpha             | `vize`, `@vizejs/native`, `@vizejs/vite-plugin`                                               | Destinado a testes iniciais de produção com alterações de quebra respaldadas por releasenote de lançamento.              |
| Prévia de compatibilidade | `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`, `@vizejs/musea-nuxt`             | Espera-se que funcione para configurações comuns de host, mas a compatibilidade host-framework pode avançar rapidamente. |
| Experimental              | `oxlint-plugin-vize`, `@vizejs/vite-plugin-musea`, `@vizejs/musea-mcp-server`, `@vizejs/wasm` | Pacotes públicos, mas APIs, comandos, saída e formato de fluxo de trabalho podem mudar durante o alfa.                   |
| Incubação                 | `@vizejs/fresco`, `@vizejs/fresco-native`, pacotes de extensão de editor                      | Útil para desenvolvimento e feedback, mas ainda não faz parte da meta de produção da alpha v1.                           |

## Níveis de Suporte de Caixas de Ferrugem

Esta tabela é o contrato de compatibilidade canônico para consumidores crates.io. Ele cobre todas as caixas
cujos metadados do Cargo permitem publicação, incluindo caixas temporariamente adiadas pelo lançamento
publicador enquanto o primeiro lançamento crates.io está preparado. Módulos privados e detalhes
implementação não são superfícies de compatibilidade.

<!-- rust-crate-support:start -->

| Caixa                | Tier                      | Público-alvo                                             | Ponto de entrada pública                        | Remoção / depreciação                      |
| -------------------- | ------------------------- | -------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------ |
| `vize_carton`        | Suporte Alpha             | Autores do compilador e biblioteca Vize                  | `vize_carton::{Allocator, Bump, FxHashMap}`     | Um minor com `#[deprecated]`               |
| `vize_relief`        | Suporte Alpha             | Autores do AST e integração de compiladores              | `vize_relief::{RootNode, CompilerOptions}`      | Um minor com `#[deprecated]`               |
| `vize_armature`      | Suporte Alpha             | Ferramentas que analisam templates do Vue                | `vize_armature::{parse, Parser, Tokenizer}`     | Uma menor com `#[deprecated]`              |
| `vize_croquis`       | Prévia de compatibilidade | Autores de ferramentas semânticas e com perfil de tipos  | `vize_croquis::{Croquis, Drawer}`               | Um minor com `#[deprecated]`               |
| `vize_croquis_cf`    | Experimental              | Experimentos de análise de projeto inteiro com opt-in    | `vize_croquis_cf::CrossFileAnalyzer`            | Sem mínimo; Quebras de nota quando prático |
| `vize_atelier_core`  | Suporte Alpha             | Autores de backend de compiladores personalizados do Vue | `vize_atelier_core::{transform, generate}`      | Um menor com `#[deprecated]`               |
| `vize_atelier_dom`   | Suporte Alpha             | Integrações com compiladores e bundlers VDOM             | `vize_atelier_dom::compile_template`            | Um minor com `#[deprecated]`               |
| `vize_atelier_vapor` | Experimental              | Integrações com compiladores Opt-in com Vapor            | `vize_atelier_vapor::compile_vapor`             | Sem mínimo; Quebras de nota quando prático |
| `vize_atelier_ssr`   | Prévia de compatibilidade | Autores de SSR e integração de frameworks                | `vize_atelier_ssr::compile_ssr`                 | Um menor com `#[deprecated]`               |
| `vize_atelier_sfc`   | Suporte Alpha             | Autores de ferramentas e bundlers SFC                    | `vize_atelier_sfc::{parse_sfc, compile_sfc}`    | Um minor com `#[deprecated]`               |
| `vize_atelier_jsx`   | Prévia de compatibilidade | Autores do compilador e ferramentas JSX/TSX              | `vize_atelier_jsx::{compile_jsx, lower_source}` | Um minor com `#[deprecated]`               |
| `vize_musea`         | Experimental              | Galeria de Musea e ferramentas de documentação           | `vize_musea::{parse_art, transform_to_csf}`     | Sem mínimo; Quebras de nota quando prático |
| `vize_fresco`        | Incubação                 | Experimentos TUI                                         | `vize_fresco::{RenderTree, LayoutEngine}`       | Sem mínimo                                 |
| `vize_canon`         | Prévia de compatibilidade | Integrações com verificador de tipos e editor            | `vize_canon::{type_check_sfc, TypeChecker}`     | Uma menor com `#[deprecated]`              |
| `vize_patina`        | Prévia de compatibilidade | Integrações Linter e Oxlint                              | `vize_patina::{lint, Linter}`                   | Um minor com `#[deprecated]`               |

<!-- rust-crate-support:end -->

Cada caixa também registra seu nível em `package.metadata.vize.stability`. O CI compara esses valores de metadados de
Cargo, essa tabela e o conjunto completo de caixas de publicação de lançamento, de modo que adicionar, remover ou reclassificar
uma caixa publicável não possa alterar silenciosamente o contrato.

### Interpretação do portão SemVer

`cargo-semver-checks` roda para as caixas do editor de lançamento que têm registros resolvíveis
baselines. Uma caixa aguardando sua primeira publicação, ou bloqueada em uma, se junta a essa matriz assim que sua linha de base
está disponível. Até lá, a verificação de metadados/tabela/lista de lançamento ainda se aplica.

| Tier                                                  | Interpretação de CI                                                                                                                            |
| ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Suporte a Alpha / Pré-visualização de compatibilidade | Uma quebra de API deve ser corrigida ou seguir a janela de decontinuação da política de suporte e carregar um marcador de quebra convencional. |
| Experimental                                          | O portão capta deriva acidental; uma quebra intencional pode usar um marcador de quebra sem janela de depreciação.                             |
| Incubação                                             | A mesma detecção se aplica, mas toda a API ou caixa pode ser substituída ou removida em qualquer versão.                                       |

Os marcadores de quebra reconhecidos pelo CI são uma `!` no título de mudança convencional ou um
`BREAKING CHANGE:` rodapés. Passar pelo portão com qualquer um dos marcadores não dispensa a janela de
de depreciação para caixas suportadas por alfa ou pré-compatibilidade.

## O que conta como estável o suficiente para o Alpha

Um pacote ou comando pode migrar para a camada suportada por alfa quando tiver:

- Caminhos de Instalação e Uso Documentados
- Cobertura de CI para build, instalação e runtime de Node suportado
- divulgue a cobertura de fumaça para os pontos de entrada publicados
- um proprietário claro para regressões e relatórios de compatibilidade
- Comportamento não suportado conhecido documentado no guia relevante

## O que ainda não foi prometido

A alpha não promete total compatibilidade com todos os casos de limite do compilador Vue, todos os layouts
gerenciadores, todas as capacidades de editor ou todas as integrações de frameworks. Quando o Vize discorda de
ferramenta oficial do Vue, trate a saída oficial como a linha base de compatibilidade, a menos que um guia do Vize
documente explicitamente um comportamento diferente. O compilador bloqueador de releases, verificação de tipos, runtime,
e superfícies de compilação Vite são nomeados na
[Vue parity matrix](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/vue-parity-matrix.md).

Para o gerenciamento de segurança, veja o repositório `SECURITY.md`. Para contribuição e fluxo de trabalho fixo, veja
`CONTRIBUTING.md`.
