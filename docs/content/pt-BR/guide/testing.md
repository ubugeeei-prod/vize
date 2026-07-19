---
title: Testes e Feedback
---

<!-- Generated translation; source: guide/testing.md -->

# Testes e Feedback

O Vize está em sua fase **de Testes no Mundo Real** : o foco é a correção, e projetos do mundo real estão
suíte de testes. Esta página é para testadores — como inspecionar o que o Vize faz, onde procurar, como
relatar resultados, como medir desempenho e como oferecer seu projeto como banco de testes.

## Inspecionar com o Playground

O playground traz um **inspetor** que mostra, lado a lado, a saída oficial do compilador SFC do Vue
, a saída do compilador do Vize, o Virtual TS gerado, o VIR e um gráfico cross-file para lotes locais de
. É a maneira mais rápida de ver exatamente onde o Vize concorda ou discorda do Vue para um determinado arquivo
`.vue`.

- Abra à <https://vizejs.dev/play/?tab=inspector>.
- Veja o guia [Compiler Inspector](./compiler-inspector.md) para saber o que cada superfície significa.

Um link de inspetor de playground é uma excelente reprodução de reparação.

## Leia os Casos de Teste

O Vize é testado intensamente e de várias maneiras diferentes — fixtures do compilador comparados ao compilador oficial do
Vue, paridade de verificação de tipos contra `vue-tsc`, snapshots de lint e formator, snapshots de código SSR
, harnesses de fuzz e fixtures de aplicações do mundo real. Antes de registrar um relatório, muitas vezes
ajuda dar uma olhada rápida nos casos existentes:

- Fixtures e snapshots de paridade do compilador e SFC sob `tests/` e cada caixa `src/snapshots/`.
- Fixaturas de aplicações do mundo real sob `tests/_fixtures/` (por exemplo, Elk, Misskey, Nuxt UI,
  Reka UI e VOICEVOX) que impulsionam E2E e VRT.

Se um caso estiver faltando ou um resultado parecer errado, esse é exatamente o tipo de feedback que essa fase deseja.

## Conclusões do Relatório

- **Texto simples é bom.** Uma descrição clara do que você fez, do que esperava e do que aconteceu
  já é valioso.
- **Se puder, anexe uma reprodução mínima** ao rastreador do GitHub - o menor arquivo `.vue` (ou
  projeto pequeno) que ainda mostra o problema. Um link para o Playground Inspector funciona muito bem.
- Relatórios de correção, reproduções, resultados de benchmarks e achados de compatibilidade ajudam. Veja o
  [Contributing](../contributing.md) guia e
  [Support](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md).

## Mede o desempenho

O Vize tem um **modo de perfilagem** embutido, então você pode medir para onde o tempo vai em vez de adivinhar.

- No desenvolvimento local, a cadeia de ferramentas expõe o perfilamento entre o parser, compilador, análise e
  fases de verificação de tipo.
- O CLI também tem: `vize check --profile` faz a checagem pelo **vize_curador** e imprime um
  relatório de perfil por fase. Use-o para capturar e compartilhar números de desempenho do seu próprio
  código.

## Ofereça seu projeto como plataforma de testes

Bases de código reais e grandes encontram as falhas que exemplos sintéticos nunca encontrarão. **Quando a licença
permitir, um projeto pode ser adicionado aos equipamentos da Vize e se tornar um**alvo E2E / VRT, para que regressões futuras de
sejam capturadas automaticamente.

Se você mantém (ou conhece) algum aplicativo, biblioteca, framework ou ferramenta do Vue que possa ser usado dessa
, por favor, nos avise – abra uma solicitação de correção ou entre em contato. Quanto maior e mais real a base de código,
mais útil o sinal.
