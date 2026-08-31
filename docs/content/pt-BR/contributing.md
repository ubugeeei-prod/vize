---
title: Contribuição
---

<!-- Generated translation; source: contributing.md -->

# Contribuição

Obrigado por ajudar a tornar o Vize mais afiado. O projeto está em sua fase **de Testes do Mundo** Real e caminhando
para a alpha v1, então pequenas mudanças focadas com verificação clara são as mais fáceis de revisar. Se você
está aqui para relatar descobertas em vez de abrir um PR, comece pelo guia
[Testing & Feedback](./guide/testing.md).

## Configuração

Use a versão Node.js da `.node-version` e a versão da Ferrugem da `rust-toolchain.toml`. O
workspace declara uma versão mínima suportada de Rust (MSRV) de `1.95.0` em `Cargo.toml`
(`[workspace.package].rust-version`); as contribuições devem ser compiladas sob essa versão.

O shell padrão do Nix contém a cadeia de ferramentas local reproduzível. O suporte ao Blacksmith Testbox
é opcional e fica em um shell separado com a CLI Blacksmith fixada, `rsync`e CLI do GitHub:

```sh
nix develop             # local development
nix develop .#testbox   # hosted Testbox workflows
```

Instalar dependências a partir da raiz do workspace:

```sh
vp install --frozen-lockfile --prefer-offline
```

Se `vp` ainda não estiver disponível, instale [Vite+](https://viteplus.dev/guide/install) primeiro.

## Cheques Comuns

Faça a verificação mais restrita que cubra sua mudança e depois amplie quando tocar em comportamentos compartilhados.

```sh
vp check <changed-files>
node --test tests/tooling/<test-file>.test.ts
cargo fmt --all -- --check
cargo test -p <crate>
```

Antes de abrir um PR que altere ferramentas compartilhadas, automação de releases, bindings nativos ou comportamento
compilador, execute a tarefa relevante do workspace a partir do CI localmente quando possível.

Os fluxos de trabalho de build, teste e lint são locais por padrão e não precisam de credenciais hospedadas:

```sh
vp run --workspace-root build
vp run --workspace-root test
vp run --workspace-root lint
```

Dentro do shell de desenvolvimento do Nix, `vp build`, `vp test`e `vp lint` são abreviações para essas tarefas
workspace.

Para paridade de CI Linux de um comando, entre em cena o shell dedicado do Testbox. A `nix develop` padrão
omite intencionalmente o Blacksmith e não precisa de seu artefato ou credenciais hospedadas:

```sh
nix develop .#testbox
```

Depois, execute o ciclo de vida protegido abaixo. Ele limpa qualquer ID antigo de caixa antes do aquecimento, pula tarefas remotas se
autenticação, push ou aquecimento falharem, e sempre tenta parar uma caixa que foi aquecida com sucesso mesmo
quando uma tarefa falha:

```sh
run_testbox_checks() {
  unset BLACKSMITH_TESTBOX_ID testbox_output
  "$VIZE_BLACKSMITH_BIN" auth login || return
  git push --set-upstream origin "$(git branch --show-current)" || return

  if testbox_output="$(vp run --workspace-root testbox:warmup)"; then
    BLACKSMITH_TESTBOX_ID="$(printf '%s\n' "$testbox_output" | tail -n1)"
  else
    warmup_status=$?
    unset testbox_output
    return "$warmup_status"
  fi
  if [ -z "$BLACKSMITH_TESTBOX_ID" ]; then
    printf '%s\n' "Testbox warmup returned no box id." >&2
    unset BLACKSMITH_TESTBOX_ID testbox_output
    return 1
  fi
  export BLACKSMITH_TESTBOX_ID

  if vp run --workspace-root build:testbox &&
    vp run --workspace-root test:testbox &&
    vp run --workspace-root lint:testbox; then
    testbox_status=0
  else
    testbox_status=$?
  fi
  if vp run --workspace-root testbox:stop; then
    stop_status=0
  else
    stop_status=$?
  fi
  unset BLACKSMITH_TESTBOX_ID testbox_output

  if [ "$testbox_status" -ne 0 ]; then
    return "$testbox_status"
  fi
  return "$stop_status"
}
run_testbox_checks
```

Para alterações de tarefas do Blacksmith Testbox, também valide a forma do fluxo de trabalho com
`node --test tests/tooling/github-workflows.test.ts`.

## Disciplina de Mudança do Processador de Linguagem

O Vize segue a prática de projetos de compilador de rustc, TypeScript, TypeScript-Go e Flow: classifica a
alteração, adiciona o menor fixture significativo, revisa a saída gerada como um contrato e depois amplia para
paridade, desempenho ou portas de lançamento quando a superfície tocada precisar. Veja
[Language Engineering Practices](./architecture/language-engineering-practices.md) para a matriz completa
.

Use uma dessas classes de mudança em PRs quando aplicável:

- Parser ou AST
- Compilador e geração de código
- Análise semântica, fiapos e análise cruzada
- Virtual TypeScript e verificação de tipos
- Forformatador e LSP
- Empacotamento, lançamento ou documentação em tempo de execução

Para mudanças voltadas para a linguagem, inclua o fixture ou snapshot diff que comprova o comportamento. Para
atualizações de snapshot, explique por que a nova saída está correta e evite um churn de base ampla, a menos que a
PR seja especificamente sobre essa família de saída.

Quando um descompasso do compilador começa a partir de um arquivo de reprodução externo ou de um arquivo de projeto local, use o
[Compiler Inspector](./guide/compiler-inspector.md) playground para inspecionar a saída oficial do Vue, a saída do Vize
Virtual TS, VIR e gráfico cross-file. Adicione o permalink do inspetor ao corpo de PR, depois finalize o
de luminária minimizada ou snapshot completo que transforme o resultado em um contrato revisado. Os lotes locais
podem ser empacotados com `vize inspector <file-or-glob>`, e a transferência de agentes pode usar
`vize inspector --format agent`.

## Pull Requests

- Use Commits Convencionais para mensagens de commit e títulos de PR, como
  `fix(vite-plugin): surface SFC compile errors`.
  - Mantenha os PRs focados em uma única mudança comportamental ou em uma mudança de documentação/governança.
- Inclua comandos de verificação no órgão de PR.
- Não atualize grandes linhas de base snapshot a menos que o PR seja especificamente sobre essas saídas.
- Não inclua segredos, tokens de registro, detalhes privados de vulnerabilidades ou caminhos locais de máquina em
  relatórios, compromissos ou PRs.

## Solicitações de Correção

Use o modelo de relatório de correção para regressões, travamentos, diagnósticos incorretos, problemas de instalação
pacotes e falhas de lançamento. Use o modelo de solicitação de recursos para novas integrações, mudanças na API,
ou melhorias no fluxo de trabalho. Uma reprodução mínima — idealmente um link de inspetor de playground — torna o relatório
muito mais rápido para ser executado.

Relatórios de segurança devem seguir
[`SECURITY.md`](https://github.com/ubugeeei-prod/vize/blob/main/SECURITY.md) em vez dos modelos públicos
corrigir.

## Código de Conduta e Governança

Ao participar, você concorda em cumprir o
[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). O modelo de governança
e o processo de tomada de decisão estão documentados em
[`GOVERNANCE.md`](https://github.com/ubugeeei-prod/vize/blob/main/GOVERNANCE.md). Para ajuda para encontrar
canal certo, veja [`SUPPORT.md`](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md).
